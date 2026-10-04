# `vxl palette`

*Part of the [Vxl Command-Line Reference](../../README.md).*

Palette operations. Addressing is per command: [`list`](list.md) selects
palettes by positional index filters such as `1` or `1-5`, and [`show`](show.md)
selects with a repeatable `--property <palette> <property> <presentation>
<reading>` selector that defaults to the whole-document wildcard
`'*' '*' auto auto`. The mutating
[`edit`](../../../palette-edit/README.md), [`quantize`](quantize.md), and
[`remap`](remap.md) select palettes with `--index` under the
[palette selection](../../../palette-edit/README.md#palette-selection) rule.
`--index` defaults to every palette. `quantize` and `remap` compare `--property`
(default `baseColor`). Property keys are the glTF vocabulary names such as
`baseColor`.

- [`vxl palette list`](list.md): overview of every palette in a document.
- [`vxl palette show`](show.md): print one palette's selected properties.
- [`vxl palette edit`](../../../palette-edit/README.md): write palette
  properties from value-language expressions.
- [`vxl palette quantize`](quantize.md): reduce each palette's materials.
- [`vxl palette remap`](remap.md): remap voxels onto a target palette.
