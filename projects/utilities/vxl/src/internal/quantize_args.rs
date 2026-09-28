use crate::{Error, Result, cli_value_parser};
use clap::Args;
use std::num::NonZeroUsize;
use voxcore::material::BASE_COLOR;
use voxsmith::utilities::{
    AlphaMode, ColorSpace, Dither, PartitionProperties, PropertyInterpretation, QuantizeOptions,
    ReductionMethod,
};

/// The flags `palette quantize` and `object voxels quantize` share, resolving
/// to voxsmith's [`QuantizeOptions`].
#[derive(Clone, Debug, Args)]
pub struct QuantizeArgs {
    /// The most materials each quantized layer may sample afterward.
    #[arg(value_name = "max-materials", long)]
    max_materials: NonZeroUsize,

    /// The property to cluster on.
    #[arg(value_name = "property", long, default_value = BASE_COLOR)]
    property: String,

    /// How the property's values read as points.
    #[arg(
        value_name = "interpret-property",
        long,
        default_value = "auto",
        value_parser = cli_value_parser::<PropertyInterpretation>()
    )]
    interpret_property: PropertyInterpretation,

    /// How a 4-component color's alpha takes part, `partition` when omitted.
    /// Any other reading errors on it.
    #[arg(value_name = "alpha", long, value_parser = cli_value_parser::<AlphaMode>())]
    alpha: Option<AlphaMode>,

    /// Merge materials only when they agree on this property. Repeatable; `*`
    /// stands for every property other than `--property`.
    #[arg(value_name = "partition", long)]
    partition: Vec<String>,

    /// Clustering algorithm.
    #[arg(
        value_name = "method",
        long,
        default_value = "median-cut",
        value_parser = cli_value_parser::<ReductionMethod>()
    )]
    method: ReductionMethod,

    /// The space a color reading measures distance in, `oklab` when omitted. A
    /// numeric reading errors on it.
    #[arg(value_name = "space", long, value_parser = cli_value_parser::<ColorSpace>())]
    space: Option<ColorSpace>,

    /// Error diffusion applied when snapping samples to representatives.
    #[arg(
        value_name = "dither",
        long,
        default_value = "none",
        value_parser = cli_value_parser::<Dither>()
    )]
    dither: Dither,
}

impl QuantizeArgs {
    /// The [`QuantizeOptions`] these flags set. Errors when `--partition '*'`
    /// comes with a named partition.
    pub fn resolve(&self) -> Result<QuantizeOptions> {
        let named: Vec<_> = self
            .partition
            .iter()
            .filter(|name| *name != "*")
            .cloned()
            .collect();

        let partition = match named.len() < self.partition.len() {
            true if !named.is_empty() => {
                return Err(Error::usage(
                    "`--partition '*'` covers every other property, so it takes no named \
                     partition beside it",
                ));
            }

            true => PartitionProperties::All,

            false => PartitionProperties::Named(named),
        };

        Ok(QuantizeOptions {
            max_materials: self.max_materials,
            property: self.property.clone(),
            interpret_property: self.interpret_property,
            alpha: self.alpha,
            partition,
            method: self.method,
            space: self.space,
            dither: self.dither,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::QuantizeArgs;
    use clap::Parser;
    use voxsmith::utilities::{PartitionProperties, QuantizeOptions};

    /// A command carrying only the quantize flags.
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        quantize: QuantizeArgs,
    }

    /// The options resolved from `args` after `--max-materials 4`.
    fn resolve(args: &[&str]) -> crate::Result<QuantizeOptions> {
        let mut argv = vec!["cli", "--max-materials", "4"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).unwrap().quantize.resolve()
    }

    #[test]
    fn max_materials_is_required_and_positive() {
        assert!(Cli::try_parse_from(["cli"]).is_err());
        assert!(Cli::try_parse_from(["cli", "--max-materials", "0"]).is_err());
    }

    #[test]
    fn defaults_leave_alpha_and_space_to_the_reading() {
        let options = resolve(&[]).unwrap();
        assert_eq!(options.max_materials.get(), 4);
        assert_eq!(options.property, "baseColor");
        assert_eq!(options.alpha, None);
        assert_eq!(options.space, None);
        assert_eq!(options.partition, PartitionProperties::Named(Vec::new()));
    }

    #[test]
    fn partitions_repeat_and_star_stands_for_all() {
        assert_eq!(
            resolve(&["--partition", "metallic", "--partition", "ior"])
                .unwrap()
                .partition,
            PartitionProperties::Named(vec!["metallic".to_owned(), "ior".to_owned()])
        );
        assert_eq!(
            resolve(&["--partition", "*"]).unwrap().partition,
            PartitionProperties::All
        );
        assert!(resolve(&["--partition", "*", "--partition", "metallic"]).is_err());
    }
}
