#!/usr/bin/env bash
# Releases the named crates and only the workspace crates they need.
#
#   scripts/release.sh plan [--patch <crate>,...] <crate>...
#   scripts/release.sh bump [--patch <crate>,...] <crate>...
#   scripts/release.sh publish [--dry-run] <crate>...
#
# `plan` prints the version bumps without changing anything. It walks the
# named crates' workspace dependencies, skipping dev-dependencies, and picks:
#
#   1. Every named crate
#   2. Every crate whose packaged files changed since its `crate@version` tag
#   3. Every crate that depends on a crate taking a breaking bump, since its
#      requirement changes too
#
# A crate without a tag is new and ships at its current version. Bumps are
# breaking unless `--patch` lists the crate: `0.2.3` goes to `0.3.0`, or to
# `0.2.4` as a patch.
#
# `bump` applies the plan to the manifests and refreshes the lockfile.
# `publish` uploads the named crates and their workspace dependencies whose
# versions crates.io lacks, one `cargo publish` each in dependency order. It
# waits out rate limits and skips published versions, so it can rerun after a
# failure. `--dry-run` prints the order without uploading.
set -euo pipefail

usage() {
  sed -n '4,6s/^#   //p' "$0" >&2
  exit 2
}

[ $# -ge 2 ] || usage

command=$1
shift

dry_run=false
if [ "$command" = publish ] && [ "$1" = --dry-run ]; then
  [ $# -ge 2 ] || usage
  dry_run=true
  shift
fi

patch=
if [ "$1" = --patch ]; then
  [ "$command" != publish ] && [ $# -ge 3 ] || usage
  patch=$2
  shift 2
fi

root=$(git rev-parse --show-toplevel)
cd "$root"

metadata=$(cargo metadata --no-deps --format-version 1)

# The named crates and their workspace dependencies, one `name version dir`
# line each.
closure() {
  jq -r --args '
    (.packages | map(.name)) as $workspace
    | (.packages
      | map({
        key: .name,
        value: [.dependencies[] | select(.kind != "dev" and (.name | IN($workspace[]))) | .name] | unique
      })
      | from_entries) as $graph
    | def walk_deps($names):
      ($names + [$names[] | $graph[.][]] | unique) as $more
      | if ($more | length) == ($names | length) then $names else walk_deps($more) end;
    ($ARGS.positional - $workspace) as $unknown
    | if ($unknown | length) > 0 then error("not workspace crates: \($unknown | join(", "))") else . end
    | walk_deps($ARGS.positional) as $closure
    | .packages[]
    | select(.name | IN($closure[]))
    | "\(.name) \(.version) \(.manifest_path | rtrimstr("/Cargo.toml"))"
  ' "$@" <<<"$metadata"
}

# One `name state` line per crate in the closure: `new` without a tag,
# `changed` when a packaged file differs from the tag, `same` otherwise.
states() {
  closure "$@" | while read -r name version dir; do
    tag="$name@$version"

    if ! git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
      echo "$name new"
      continue
    fi

    rel=${dir#"$root"/}
    changed=$(git diff --name-only --relative="$rel" "$tag" HEAD -- "$rel")

    if [ -n "$changed" ] &&
      grep -qxFf <(cargo package --list --allow-dirty -p "$name" 2>/dev/null) <<<"$changed"; then
      echo "$name changed"
    else
      echo "$name same"
    fi
  done
}

# One `name old new kind reason` line per bump, plus `name version version new`
# per new crate.
plan() {
  states "$@" | jq -rRn --slurpfile metadata <(echo "$metadata") --arg patch "$patch" --args '
    ([inputs | split(" ") | {key: .[0], value: .[1]}] | from_entries) as $states
    | ($patch | split(",") | map(select(. != ""))) as $patch_list
    | ($ARGS.positional) as $named
    | $metadata[0].packages as $packages
    | ($packages | map({key: .name, value: .}) | from_entries) as $by_name
    | ($states | keys) as $closure
    | ($patch_list - $closure) as $stray
    | if ($stray | length) > 0 then error("--patch crates outside the plan: \($stray | join(", "))") else . end
    | def kind($name): if $name | IN($patch_list[]) then "patch" else "breaking" end;
    def needs($name; $bumped):
      [$by_name[$name].dependencies[]
        | select(.kind != "dev" and (.name | IN($bumped[])) and kind(.name) == "breaking")
        | .name]
      | unique;
    def grow($bumped):
      ($bumped + [$closure[]
        | select($states[.] != "new" and (IN($bumped[]) | not) and (needs(.; $bumped) | length) > 0)])
      as $more
      | if ($more | length) == ($bumped | length) then $bumped else grow($more) end;
    def bumped_version($version; $kind):
      ($version | split(".") | map(tonumber)) as [$major, $minor, $micro]
      | if $kind == "patch" then "\($major).\($minor).\($micro + 1)"
        elif $major == 0 then "0.\($minor + 1).0"
        else "\($major + 1).0.0" end;
    grow([$closure[] | select($states[.] == "changed" or (IN($named[]) and $states[.] != "new"))]) as $bumped
    | ($closure[] | select($states[.] == "new") | "\(.) \($by_name[.].version) \($by_name[.].version) new"),
      ($bumped | sort[]
        | . as $name
        | (if $states[$name] == "changed" then "changed"
          elif IN($named[]) then "named"
          else "needs " + (needs($name; $bumped) | join(",")) end) as $reason
        | "\($name) \($by_name[$name].version) \(bumped_version($by_name[$name].version; kind($name))) \(kind($name)) \($reason)")
  ' "$@"
}

# Sets `$1`'s requirement to `$2` in every dependency table of the manifest
# `$3`, single-line or multi-line.
set_requirement() {
  NAME=$1 VERSION=$2 perl -0777 -i -pe '
    my @sections = split /(?=^\[)/m;

    for (@sections) {
      next unless /^\[(?:[^\]]*\.)?(?:dev-|build-)?dependencies\]/;

      s/^(\s*\Q$ENV{NAME}\E\s*=\s*")[^"]*"/$1$ENV{VERSION}"/mg;

      s/^(\s*\Q$ENV{NAME}\E\s*=\s*\{[^}]*?\bversion\s*=\s*")[^"]*"/$1$ENV{VERSION}"/msg;
    }

    $_ = join "", @sections;
  ' "$3"
}

# Sets the `[package]` version of the manifest `$1` to `$2`.
set_version() {
  VERSION=$1 perl -0777 -i -pe '
    my @sections = split /(?=^\[)/m;

    for (@sections) {
      s/^(version\s*=\s*")[^"]*"/$1$ENV{VERSION}"/m if /^\[package\]/;
    }

    $_ = join "", @sections;
  ' "$2"
}

case $command in
plan)
  plan "$@" | column -t
  ;;

bump)
  lines=$(plan "$@")
  echo "$lines" | column -t

  manifests=$(jq -r '.packages[].manifest_path' <<<"$metadata")

  echo "$lines" | while read -r name old new kind _; do
    [ "$kind" = new ] && continue

    manifest=$(jq -r --arg name "$name" '.packages[] | select(.name == $name) | .manifest_path' <<<"$metadata")
    set_version "$new" "$manifest"

    if [ "$kind" = breaking ]; then
      for other in $manifests; do
        set_requirement "$name" "$new" "$other"
      done
    fi
  done

  cargo update -w

  stale=$(cargo metadata --format-version 1 | jq -r '
    (.packages | map(select(.source == null) | .name)) as $workspace
    | .packages[]
    | select(.source != null and (.name | IN($workspace[])))
    | "\(.name) \(.version)"')

  if [ -n "$stale" ]; then
    echo "error: the lockfile resolves workspace crates from crates.io:" >&2
    echo "$stale" >&2
    exit 1
  fi
  ;;

publish)
  unpublished=()

  while read -r name version _; do
    lower=$(tr '[:upper:]' '[:lower:]' <<<"$name")

    case ${#lower} in
    1) prefix=1 ;;
    2) prefix=2 ;;
    3) prefix=3/${lower:0:1} ;;
    *) prefix=${lower:0:2}/${lower:2:2} ;;
    esac

    index=$(mktemp)
    status=$(curl -s -o "$index" -w '%{http_code}' "https://index.crates.io/$prefix/$lower")

    case $status in
    200)
      jq -e --arg version "$version" 'select(.vers == $version)' "$index" >/dev/null ||
        unpublished+=("$name")
      ;;

    404)
      unpublished+=("$name")
      ;;

    *)
      echo "error: the crates.io index answered $status for $name" >&2
      exit 1
      ;;
    esac

    rm "$index"
  done < <(closure "$@")

  if [ ${#unpublished[@]} -eq 0 ]; then
    echo "every crate is published"
    exit 0
  fi

  # Dependencies first, dev-dependencies included, since packaging resolves
  # them against crates.io.
  order=$(jq -r --args '
    ($ARGS.positional) as $set
    | (.packages
      | map(select(.name | IN($set[])))
      | map({key: .name, value: [.dependencies[] | select(.name | IN($set[])) | .name] | unique})
      | from_entries) as $graph
    | def order($left; $done):
      if ($left | length) == 0 then $done
      else
        [$left[] | select(all($graph[.][]; IN($done[])))] as $ready
        | if ($ready | length) == 0 then error("dependency cycle among \($left | join(", "))")
          else order($left - $ready; $done + ($ready | sort)) end
      end;
    order($set; [])[]
  ' "${unpublished[@]}" <<<"$metadata")

  if $dry_run; then
    echo "$order"
    exit 0
  fi

  for name in $order; do
    log=$(mktemp)

    # crates.io answers 429 past a burst of uploads and says when to retry.
    until cargo publish -p "$name" 2>&1 | tee "$log"; [ "${PIPESTATUS[0]}" -eq 0 ]; do
      grep -q "429" "$log" || exit 1
      sleep 90
    done

    rm "$log"
  done
  ;;

*)
  usage
  ;;
esac
