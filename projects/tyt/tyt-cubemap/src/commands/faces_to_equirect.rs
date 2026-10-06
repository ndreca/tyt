use crate::{C6X1_FACES, Dependencies, Result, pixelate_faces};
use clap::Parser;
use std::path::Path;

/// Converts six cube face images into a single equirectangular panorama.
#[derive(Clone, Debug, Parser)]
pub struct FacesToEquirect {
    /// Base name for the input face files (`<base>-left.png`, etc.).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `<base>-equirect`.
    #[arg(value_name = "output-base")]
    out_base: Option<String>,

    /// Uses nearest-neighbor filtering on the final `--output-size` resize.
    #[arg(value_name = "point", long)]
    point: bool,

    /// Uses nearest-neighbor filtering (`interp=near`) on the `v360` reprojection
    /// itself.
    #[arg(value_name = "point-reprojection", long)]
    point_reprojection: bool,

    /// Pixelates (point-resizes) the faces to the given height in pixels before
    /// converting. Implies `--point-reprojection` so the downscaled pixel edges
    /// survive the reprojection.
    #[arg(value_name = "pixelate", long)]
    pixelate: Option<u32>,

    /// Final output height in pixels. When set, the equirectangular image is
    /// resized to this height. Combine with `--point` for nearest-neighbor
    /// filtering that preserves hard edges.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl FacesToEquirect {
    /// Reprojects the faces in a temp directory and reports the written path.
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self
            .out_base
            .unwrap_or_else(|| format!("{}-equirect", self.base));
        let point_reprojection = self.point_reprojection || self.pixelate.is_some();
        let tmp_dir = deps.create_temp_dir()?;

        let result: Result<String> = (|| {
            let equirect_base = if let Some(size) = self.pixelate {
                let tmp_base = tmp_dir.join("face");
                let tmp_base_str = tmp_base.to_string_lossy().into_owned();
                pixelate_faces(&deps, &self.base, &tmp_base_str, size, None)?;
                tmp_base_str
            } else {
                self.base.clone()
            };
            let out_path = faces_to_equirect(
                &deps,
                &equirect_base,
                &out_base,
                &tmp_dir,
                point_reprojection,
            )?;
            if let Some(output_size) = self.output_size {
                let resize = format!("x{output_size}");
                if self.point {
                    deps.exec_magick([
                        out_path.as_str(),
                        "-filter",
                        "point",
                        "-resize",
                        &resize,
                        &out_path,
                    ])?;
                } else {
                    deps.exec_magick([out_path.as_str(), "-resize", &resize, &out_path])?;
                }
            }
            Ok(out_path)
        })();

        deps.remove_dir_all(&tmp_dir)?;
        let out_path = result?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}

/// Appends six cube face images into a horizontal strip and converts it to
/// equirectangular.
fn faces_to_equirect(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    tmp_dir: &Path,
    point_reprojection: bool,
) -> Result<String> {
    let strip_path = tmp_dir.join("strip.png");
    let strip_str = strip_path.to_string_lossy().into_owned();
    let out_path = format!("{out_base}.png");

    let mut magick_args: Vec<String> = C6X1_FACES
        .iter()
        .map(|face| format!("{base}-{face}.png"))
        .collect();
    magick_args.push("+append".into());
    magick_args.push(strip_str.clone());
    deps.exec_magick(magick_args)?;

    let vf = if point_reprojection {
        "v360=c6x1:e:interp=near"
    } else {
        "v360=c6x1:e"
    };
    deps.exec_ffmpeg([
        "-y",
        "-loglevel",
        "error",
        "-i",
        &strip_str,
        "-vf",
        vf,
        &out_path,
    ])?;

    Ok(out_path)
}
