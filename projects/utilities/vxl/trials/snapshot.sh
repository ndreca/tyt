#!/bin/sh
# PostToolUse hook: after a Bash call that renders, copies the slot's new PNGs
# and model files into _runs/<run>/passes/<n>. A render with no voxelize since
# the last snapshot adds its PNGs to that pass.
command=$(jq -r '.tool_input.command // ""')
slot=${CLAUDE_PROJECT_DIR:-$PWD}
out=$(dirname "$slot")/_runs/$(basename "$slot")
[ -f "$out/snapshot-marker" ] || exit 0
case $command in *"sdf-doc voxelize"*) touch "$out/voxelized" ;; esac
case $command in *"object render"*) ;; *) exit 0 ;; esac
pngs=$(cd "$slot" && find . -path ./.claude -prune -o -name '*.png' -newer "$out/snapshot-marker" -print)
[ -n "$pngs" ] || exit 0
n=$(ls -d "$out"/passes/[0-9]* 2> /dev/null | wc -l | tr -d ' ')
[ -f "$out/voxelized" ] || [ "$n" = 0 ] && n=$((n + 1))
pass=$out/passes/$(printf '%02d' "$n")
mkdir -p "$pass"
(cd "$slot" && printf '%s\n' "$pngs" | while read -r f; do cp "$f" "$pass/"; done)
(cd "$slot" && find . -path ./.claude -prune -o -name '*.ts' -print | while read -r f; do cp "$f" "$pass/"; done)
rm -f "$out/voxelized"
touch "$out/snapshot-marker"
exit 0
