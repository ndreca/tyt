use crate::{Dependencies, QuantizeArgs, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxsmith::operations::palette::quantize_palette;

/// Reduces a palette to at most `--max-materials` materials and snaps every
/// layer referencing it. Materials no voxel samples drop, and the rest compact.
/// `object voxels quantize` reduces what objects sample without changing the
/// palette.
#[derive(Clone, Debug, Parser)]
#[command(name = "quantize")]
pub struct PaletteQuantize {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    /// Which palette to quantize.
    #[arg(value_name = "index", long, default_value_t = 0)]
    index: usize,

    #[command(flatten)]
    quantize: QuantizeArgs,
}

impl PaletteQuantize {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let options = self.quantize.resolve()?;

        edit_document(&dependencies, &self.input, self.output, |main| {
            Ok(quantize_palette(main, self.index, &options)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::PaletteQuantize;
    use clap::Parser;

    #[test]
    fn index_defaults_to_the_first_palette() {
        let parse = |args: &[&str]| {
            let mut argv = vec!["quantize", "scene.voxj", "--max-materials", "16"];
            argv.extend_from_slice(args);
            PaletteQuantize::try_parse_from(argv).unwrap()
        };

        assert_eq!(parse(&[]).index, 0);
        assert_eq!(parse(&["--index", "2"]).index, 2);
    }
}
