# Voxel rendering follow-ups

Status: **open.** The
[voxel rendering plan](../../closed/voxel-rendering/README.md) closed on
2026-10-01 with the contract, the CPU reference, and `vxl object render`
built. This plan lists what it left open. A part gets its design, its
checklist, and its decisions log when it starts.

## Transparency

Alpha, `transmission`, and `ior`. Built as the
[voxel transparency plan](../../closed/voxel-transparency/README.md), closed
2026-10-04 with the reference, the mesher, and the `glass` profile done.

## `voxrender-wgpu`

The realtime tiers over the same render scene, drawing into a texture the
caller owns. Opened 2026-10-04 as the
[voxrender-wgpu plan](../voxrender-wgpu/README.md). It revisits the
[Realtime tiers](../../closed/voxel-rendering/README.md#realtime-tiers) of the
closed rendering plan with the VR experience first.

## Colored glass on a transparent background

Under `--background transparent`, colored glass vanishes wherever nothing
in the scene lies behind it. The output takes one alpha from the peak
transmittance. Red glass passes 96% of red, so its pixel comes out nearly
transparent and the tint has no channel to live in. The red pane in
`glass.voxj` loses its top face this way. Over a background color it reads
correctly. Found on 2026-10-04 while reviewing the integer walk. The
options so far:

1. `object render` defaults to a solid background, and transparent output
   keeps today's rule
2. Alpha comes from the least-transmitted channel, with the color solved to
   be exact over white. Colored glass then reads too bright over dark
   backgrounds
3. The rule stays, and the docs say colored glass needs a background

Still open: whether drawing the glass's inner back faces would help.

## Waiting for a reason

1. Traced occlusion as a third `occlusion` value
2. Area lights
3. Tiling several views into one sheet
