# Voxel rendering follow-ups

Status: **open.** The
[voxel rendering plan](../../closed/voxel-rendering/README.md) closed on
2026-10-01 with the contract, the CPU reference, and `vxl object render`
built. This plan lists what it left open. A part gets its design, its
checklist, and its decisions log when it starts.

## Transparency

Alpha, `transmission`, and `ior`. Started 2026-10-02 as the
[voxel transparency plan](../voxel-transparency/README.md), which settles
what a transparent voxel is and adds it to the reference.

## `voxrender-wgpu`

The realtime tiers over the same render scene, drawing into a texture the
caller owns. The closed plan's
[Realtime tiers](../../closed/voxel-rendering/README.md#realtime-tiers)
section holds the design. The standalone tier comes first. It rasterizes
`voxsurface`'s greedy mesh and lights each face from the grid rays the
reference casts, so `per-face` and `per-corner` shadows match the reference
exactly. The desktop tier adds volume marching later. A tier turns off the
contract features it cannot afford, and the reference renders that tier's
golden images with the same features off. The crate name was free on
crates.io as of 2026-09-27. The tier needs the transparency-aware cull the
[voxel transparency plan](../voxel-transparency/README.md)'s mesh step
gives `voxsurface` before it can show the glass the reference renders.

## Waiting for a reason

1. Traced occlusion as a third `occlusion` value
2. Area lights
3. Tiling several views into one sheet
