use crate::{
    Dependencies, Result, VoxjEncoding, VoxjFormat, VoxjOptimize, VoxjPositionEncoding,
    VoxjSampleEncoding,
};
use clap::Parser;
use std::path::PathBuf;

/// Converts a `.vmax` package directory to a Voxel Json document, written to
/// stdout.
///
/// `--format` chooses the output container and printing form. The block
/// encodings come from `--position-encoding` and `--sample-encoding`, or from
/// `--optimize` (which picks them automatically and cannot be combined with
/// the explicit encoding flags).
#[derive(Clone, Debug, Parser)]
#[command(name = "to-voxj")]
pub struct ToVoxj {
    /// The input `.vmax` package directory.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,

    /// Output container and printing form.
    #[arg(value_name = "format", long, default_value = "json")]
    format: VoxjFormat,

    /// Position-block encoding. Cannot be combined with `--optimize`.
    #[arg(
        value_name = "position-encoding",
        long,
        default_value = "bitmap-base64",
        conflicts_with = "optimize"
    )]
    position_encoding: VoxjPositionEncoding,

    /// Sample-block encoding. Cannot be combined with `--optimize`.
    #[arg(
        value_name = "sample-encoding",
        long,
        default_value = "rle-json",
        conflicts_with = "optimize"
    )]
    sample_encoding: VoxjSampleEncoding,

    /// Chooses the encodings automatically. Cannot be combined with
    /// `--position-encoding` or `--sample-encoding`.
    #[arg(value_name = "optimize", long)]
    optimize: Option<VoxjOptimize>,
}

impl ToVoxj {
    /// Writes the `.vmax` package to stdout as a Voxel Json document.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let encoding = match self.optimize {
            Some(VoxjOptimize::Size) => VoxjEncoding::Smallest,

            Some(VoxjOptimize::Fast) => VoxjEncoding::Fixed {
                position: VoxjPositionEncoding::BitmapBase64,
                sample: VoxjSampleEncoding::PackedBase64,
            },

            Some(VoxjOptimize::Pretty) => VoxjEncoding::Fixed {
                position: VoxjPositionEncoding::RawJson,
                sample: VoxjSampleEncoding::RawJson,
            },

            None => VoxjEncoding::Fixed {
                position: self.position_encoding,
                sample: self.sample_encoding,
            },
        };
        dependencies.write_voxj(&self.input_vmax, encoding, self.format)
    }
}
