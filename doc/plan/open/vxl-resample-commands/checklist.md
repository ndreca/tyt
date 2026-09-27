# Checklist

The design is in the [README](README.md). Check steps off as they land.

## Ground rules

- The voxsmith operations are generic over `VoxExt` and live under
  `operations/object/`. vxl stays a thin clap layer shaped like the other
  `object` commands.
- Each voxsmith operation gets unit tests, including hook firing through the
  recording ext. Each vxl command gets a parse test.

## Steps

- [x] **S1. voxcore resample.** Add `VoxObject::resample_voxels`,
      `VoxMain::resample_object_voxels`, and the
      `object_voxels_did_resample` hook. Forward the hook in voxconv's
      `CompositeVoxExt`. Make the vmax and goxl exts refresh on it as they do
      on a remap.
- [x] **S2. voxsmith operations.** `downsample_objects` and
      `upsample_objects` with `ResampleFactor` and `KeepRule`.
- [x] **S3. vxl commands.** `object downsample` and `object upsample`.
- [x] **S4. Docs.** Extend the vxl README's Objects section and the command
      list in the [vxl-commands README](../vxl-commands/README.md#commands).
