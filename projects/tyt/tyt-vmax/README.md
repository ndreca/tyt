# tyt-vmax

Works with [Voxel Max](https://www.voxelmax.com/) packages.

## Usage

```
vmax <command> [options]
```

Some examples:

```sh
vmax hierarchy my-scene.vmax                                   # Print the scene hierarchy
vmax hierarchy my-scene.vmax "Cube*"                           # Filter to matching subtrees
vmax pack my-scene.vmax                                        # Strip history files in-place
vmax pack my-scene.vmax --output-vmax packed.vmax              # Strip history into a copy
vmax rename-node my-scene.vmax "Cube*" "Box"                   # Rename matching nodes
vmax to-voxj my-scene.vmax > my-scene.voxj                     # Convert to Voxel Json
vmax to-voxj my-scene.vmax --format zip > my-scene.voxjz       # Convert to compressed Voxel Json
vmax to-voxj my-scene.vmax --optimize size > my-scene.voxj     # Pick the smallest encodings
vmax integration completion zsh install                        # Install shell completions
```

`to-voxj` writes the document to stdout. `--format` selects the form (`json`,
`zip`, or `pretty`); `--optimize` (`size`/`fast`/`pretty`) picks the block
encodings automatically, or set them explicitly with `--position-encoding` and
`--sample-encoding`.

Run `vmax <command> --help` for full details on any subcommand:

```
> vmax --help
Works with Voxel Max packages

Usage: vmax <command>

Commands:
  integration  Sets up other tools to work with vmax
  from-voxj    Converts a Voxel Json document into a `.vmax` package directory
  hierarchy    Prints the Voxel Max hierarchy as a box-glyph tree, optionally filtered to selected nodes and their subtrees
  pack         Packs a `.vmax` package directory by stripping history, renumbering the surviving contents, palettes, and thumbnails, and deleting files no longer referenced by `scene.json`
  rename-node  Renames nodes in the Voxel Max scene hierarchy matching a selection pattern
  to-voxj      Converts a `.vmax` package directory to a Voxel Json document, written to stdout
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Building from source

```sh
cargo check                              # Type-check the workspace
cargo build -p tyt-vmax --features bin   # Build the binary
```

## License

[MIT](LICENSE)
