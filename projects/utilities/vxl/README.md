# vxl

A command-line tool for working with voxels.

## Install

### Rust

[rustup](https://rustup.rs) installs Rust and Cargo. On Windows, the rustup site
offers `rustup-init.exe` in place of this command:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### vxl

vxl installs from [crates.io](https://crates.io/crates/vxl):

```sh
cargo install vxl --features bin
```

The `bin` feature builds the `vxl` binary. The same command upgrades vxl.

### Node

`sdf-doc build` runs model files under [Node](https://nodejs.org) 24 or later.
`--runtime bun` and `--runtime deno` run the model files under
[Bun](https://bun.sh) or [Deno](https://deno.com) instead. The skill's passes
run under Node.

### Shell completions

`vxl integration completion print <shell>` prints completions to stdout:

```sh
# Bash (bash-completion v2 user-local)
mkdir -p ~/.local/share/bash-completion/completions
vxl integration completion print bash > ~/.local/share/bash-completion/completions/vxl

# Zsh
mkdir -p ~/.zsh/completions
vxl integration completion print zsh > ~/.zsh/completions/_vxl
# Then ensure this is in your .zshrc *before* compinit:
#   fpath=("$HOME/.zsh/completions" $fpath)

# Zsh (Oh My Zsh)
mkdir -p ~/.oh-my-zsh/custom/completions
vxl integration completion print zsh > ~/.oh-my-zsh/custom/completions/_vxl
# If completions don't show up, ensure this is in your .zshrc *before* compinit:
#   fpath=("$HOME/.oh-my-zsh/custom/completions" $fpath)

# Fish
mkdir -p ~/.config/fish/completions
vxl integration completion print fish > ~/.config/fish/completions/vxl.fish

# PowerShell
# recommended: keep completions in a separate file and dot-source it from your $PROFILE
$dir = Join-Path $HOME ".config\powershell"
New-Item -ItemType Directory -Force -Path $dir | Out-Null

vxl integration completion print powershell | Set-Content -Encoding UTF8 (Join-Path $dir "vxl-completions.ps1")

if (!(Test-Path $PROFILE)) { New-Item -ItemType File -Force -Path $PROFILE | Out-Null }
$line = ". `"$dir\vxl-completions.ps1`""
if (-not (Select-String -Quiet -Path $PROFILE -Pattern [regex]::Escape($line))) {
  Add-Content -Path $PROFILE -Value $line
}
```

### Claude Code skill

The `voxel-modeling` skill teaches [Claude Code](https://claude.com/claude-code)
to build voxel models with vxl:

```sh
# Installs the skill for every project.
mkdir -p ~/.claude/skills/voxel-modeling
vxl integration skill print voxel-modeling > ~/.claude/skills/voxel-modeling/SKILL.md
```

A project's `.claude/skills` folder holds the skill for that project alone. The
skill teaches the API of the vxl that printed it. Upgrading vxl takes a reprint.

## Modeling with Claude Code

A session with the skill writes a model file, voxelizes it, reviews the renders,
and revises the model until it matches the prompt. The session can also export
the model as a glTF mesh. A prompt such as "Make a voxel treasure chest about 60
cm wide and export it as a glb" leaves `chest.ts`, `chest.sdfj`, `chest.voxj`,
the review PNGs, and `chest.glb` in the working folder.

A permission rule in the project's `.claude/settings.json` lets the session run
vxl without asking each time:

```json
{
  "permissions": {
    "allow": ["Bash(vxl:*)"]
  }
}
```

`sdf-doc build` runs the model file as a program, so the rule also lets the
session run the code it writes.

## Editing

The `object`, `node`, `palette edit`, and `palette quantize` commands edit a
document. Each reads any format voxconv reads and writes Voxel JSON beside the
input by default. `object mesh` writes a mesh instead. `node list` only prints
the scene graph. `--select` takes a hierarchy-path glob. `--select-index` takes
an id, an `a-b` range, or `*`. Both repeat and pick what the command acts on.
`--select-parent` and `--select-parent-index` pick the one node at the parent
end of an edge.

```sh
# Reads Voxel Max and writes scene.voxj beside it.
vxl object remove scene.vmax --select 'debris/**'
```

## Objects

`object` commands write object properties and move the voxels along when needed.
`set origin` moves an object's grid within its node. `set edit-bounds` and
`trim` resize the grid without moving a voxel in the scene. `link` and `unlink`
add or drop one placement. `duplicate` copies objects within the document. `add`
copies objects from another file along with their palettes. `downsample` and
`upsample` change the grid's resolution by a whole-number factor per axis,
scaling `origin` and `bounds` with it, so `node set scale` keeps the object's
size in the scene. `mesh` writes the selected objects as a glTF mesh. The
[mesh reference](../../../doc/ref/mesh/mesh.md) covers its flags.

`render` draws the selected objects into one image per view, placed as `mesh`
places them. The views and lights come from profiles in `.vxlconfig` and the
flags that mirror them. With neither, the `hero` view renders under the
`studio` lights. `--profile glow` adds a bloom over the emissive
materials. The image shows inline in the terminal, or `--to png` writes
one PNG per view beside the input. The
[render reference](../../../doc/ref/render/render.md) covers the flags and the
profiles.

```sh
# Copies the first three props under the one node matching house.
vxl object add scene.voxj --source props.voxj --select-index 0-2 --select-parent house

# Grows the crate's edit box from [-2 -6 0]..[2 2 4] by 2 voxels on -x and -z.
vxl object set edit-bounds scene.voxj --select crate --min -4 -6 -2 --max 2 2 4

# Merges the crate's voxels two per axis, keeping a block at least half live.
vxl object downsample scene.voxj --select crate --factor 2

# Splits each of the crate's voxels into ten per axis.
vxl object upsample scene.voxj --select crate --factor 10

# Shows the crate from the front-right-top under the studio lights.
vxl object render scene.voxj --select crate

# Writes scene-front.png and scene-top.png beside the input.
vxl object render scene.voxj --profile front --profile top --to png
```

## Object Voxels

`object voxels` commands edit voxels within the grid and never change `origin`
or `bounds`. `translate` shifts the voxels, `flip` mirrors them, and `rotate`
turns them in quarter turns that follow the right-hand rule. One or three turns
need the two turned dimensions to be equal. `node set rotation` turns any
object.

`quantize` rewrites the materials voxels sample so each quantized layer samples
at most `--max-materials` materials of its palette. The palettes stay as they
are. Each object clusters apart unless `--shared` clusters the whole selection
together. The clustering flags match `palette quantize`'s.

```sh
# Turns the 6 x 8 x 6 crate a quarter turn about y.
vxl object voxels rotate scene.voxj --select crate --axis y --turns 1

# Brings the crate down to 16 materials, keeping metals and dielectrics apart.
vxl object voxels quantize scene.voxj --select crate --max-materials 16 --partition metallic
```

## Palettes

`palette list` and `palette show` print palettes. `palette edit` and
`palette quantize` act on each palette `--index` selects. `--index` takes an id,
an `a-b` range, or `*`. The flag repeats and defaults to `*`.

`palette edit` runs a [value-language](../../../doc/ref/mesh/value-language.md)
program over each palette. Each property the program reads enters as a swatch
array holding one entry per material. `--value` adds bindings to the program.
`--write-property` writes an expression's result into a property and adds the
property when the palette lacks it. A bare whole number written to a float
property reads as `f64`. Every selected palette evaluates before any write
lands, so an error leaves the document untouched.

`palette quantize` reduces each palette to at most `--max-materials` materials
and snaps every voxel sampling it. Each cluster collapses onto its most-sampled
material, and a merged voxel takes that whole material. Clustering runs on
`--property`, `baseColor` by default. `--partition` keeps materials apart unless
they agree on a property. Materials no voxel samples drop. A palette no voxel
samples errors.

`palette edit` and both quantize commands take `--profile`, which applies flags
saved in a `.vxlconfig` under `palette.edit.profiles`,
`palette.quantize.profiles`, or `object.voxels.quantize.profiles`.
`vxl profile palette edit list`, `vxl profile palette quantize list`, and
`vxl profile object voxels quantize list` print the profiles.

```sh
# Gives rusty materials roughness 0.9 in every palette.
vxl palette edit scene.voxj \
  --value 'rust = tag == "rust"' \
  --write-property roughness 'mix(roughness, 0.9, rust)'

# Reduces every palette to Voxel Max's 255 colors.
vxl palette quantize scene.voxj --max-materials 255
```

## Nodes

`node list` prints the scene graph. The other `node` commands write nodes and
hierarchy edges. `set` writes a node's name, position, rotation, or scale.
`set rotation` takes Euler angles in the order `node list --show-transforms`
prints them, in degrees by default. `add` creates an empty node. `link` and
`unlink` add or drop one child edge. Without a parent selector, `add`, `link`,
and `unlink` act on the root list. `remove` releases nodes along with each
descendant left without a parent.

```sh
# Swings the door node 37 degrees about y.
vxl node set rotation scene.voxj --select house/door --rotation 0 37 0

# Places the same door node under garage too.
vxl node link scene.voxj --select house/door --select-parent garage
```

## Models

`sdf-doc build` records a voxel model written in TypeScript as an `.sdfj`
document. `sdf-doc voxelize` samples the document into Voxel JSON. The
[modeling API](docs/modeling-api.md) lists every call a model file can make.
[Modeling with Claude Code](#modeling-with-claude-code) sets up the skill that
runs these commands for an agent.

```sh
# Records chair.ts as chair.sdfj with the built-in materials.
vxl sdf-doc build chair.ts --library materials

# Writes chair.voxj at 2.5 cm per voxel and prints the report.
vxl sdf-doc voxelize chair.sdfj --voxel-size 0.025 --report

# Writes chair.glb with the materials baked into textures.
vxl object mesh chair.voxj --profile pbr
```
