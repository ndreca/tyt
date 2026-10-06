use clap::ValueEnum;

/// How `from-voxj` stores an object's colors in the rebuilt `.vmax` package.
/// Voxel Max accepts either a color image or a color table inside the material
/// settings sidecar, but uses only one at a time.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ColorFormat {
    /// Store colors as a `256 x 1` `palette*.png` image and leave the
    /// `palette*.settings.vmaxpsb` sidecar without a `colors` table.
    #[value(name = "png")]
    Png,

    /// Store colors in the `palette*.settings.vmaxpsb` sidecar's `colors` table
    /// and write no `palette*.png`. The `pal` reference still names the absent
    /// image.
    #[value(name = "plist")]
    Plist,

    /// Store colors in both the image and the sidecar.
    #[value(name = "all")]
    All,
}
