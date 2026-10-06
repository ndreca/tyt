use crate::{
    COMMON_PY, CameraArgs, Dependencies, Error, FBX_HIERARCHY_JSON_PY, Lighting, Projection,
    Renderer, Result, Script, embed_blender_script, extract_json, match_hierarchy_paths,
};
use clap::Parser;
use std::{
    env,
    ffi::{OsStr, OsString},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// The Blender script that renders an FBX file to an image.
const FBX_RENDER_PY: Script = embed_blender_script!("fbx_render.py");

/// Renders the meshes in an FBX file from a specified camera position. The
/// result is written to an image file, displayed inline in the terminal (Kitty,
/// iTerm2, Sixel, or ANSI fallback), or both.
#[derive(Clone, Debug, Parser)]
pub struct Render {
    /// The input FBX file.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// The output PNG file to write. When omitted, the image is rendered to
    /// the terminal only.
    #[arg(value_name = "output-image", conflicts_with = "output_image_flag")]
    output_image_arg: Option<PathBuf>,

    /// The output PNG file to write. When omitted, the image is rendered to
    /// the terminal only.
    #[arg(
        value_name = "output-image",
        short = 'o',
        long = "output-image",
        conflicts_with = "output_image_arg"
    )]
    output_image_flag: Option<PathBuf>,

    /// Also displays the rendered image in the terminal. Implied when
    /// `output-image` is omitted.
    #[arg(value_name = "terminal", long)]
    terminal: bool,

    /// Render width in pixels.
    #[arg(value_name = "resolution-x", long, default_value_t = 1920)]
    resolution_x: u32,

    /// Render height in pixels.
    #[arg(value_name = "resolution-y", long, default_value_t = 1080)]
    resolution_y: u32,

    /// Camera focal length in millimeters. Cannot be combined with
    /// `--projection orthographic` or `--fov`.
    #[arg(value_name = "focal-length", long, conflicts_with = "fov")]
    focal_length: Option<f64>,

    /// Horizontal field of view in degrees. Cannot be combined with
    /// `--projection orthographic` or `--focal-length`.
    #[arg(value_name = "fov", long)]
    fov: Option<f64>,

    /// Camera projection.
    #[arg(
        value_name = "projection",
        long,
        value_enum,
        default_value_t = Projection::Perspective,
    )]
    projection: Projection,

    /// Orthographic scale (world units visible across the frame). Requires
    /// `--projection orthographic`. Defaults to the scene-bounds diagonal.
    #[arg(value_name = "ortho-scale", long)]
    ortho_scale: Option<f64>,

    /// Near clipping plane distance.
    #[arg(value_name = "near", long, default_value_t = 0.1)]
    near: f64,

    /// Far clipping plane distance.
    #[arg(value_name = "far", long, default_value_t = 1000.0)]
    far: f64,

    /// Render engine.
    #[arg(value_name = "renderer", long, value_enum, default_value_t = Renderer::Eevee)]
    renderer: Renderer,

    /// Render samples (AA or path-tracing samples, depending on the renderer).
    #[arg(value_name = "samples", long, default_value_t = 64)]
    samples: u32,

    /// Lighting preset.
    #[arg(
        value_name = "lighting",
        long,
        value_enum,
        default_value_t = Lighting::Environment,
    )]
    lighting: Lighting,

    #[command(flatten)]
    camera: CameraArgs,
}

impl Render {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Render {
            input_fbx,
            output_image_arg,
            output_image_flag,
            terminal,
            resolution_x,
            resolution_y,
            focal_length,
            fov,
            projection,
            ortho_scale,
            near,
            far,
            renderer,
            samples,
            lighting,
            camera,
        } = self;

        camera.validate()?;

        let output_image = output_image_arg.or(output_image_flag);
        let display_in_terminal = terminal || output_image.is_none();

        let (render_path, temp_dir) = match &output_image {
            Some(path) => {
                let absolute = if path.is_absolute() {
                    path.clone()
                } else {
                    env::current_dir()?.join(path)
                };
                (absolute, None)
            }

            None => {
                let dir = dependencies.create_temp_dir()?;
                (dir.join("render.png"), Some(dir))
            }
        };

        match projection {
            Projection::Perspective => {
                if ortho_scale.is_some() {
                    return Err(Error::IO(IOError::new(
                        ErrorKind::InvalidInput,
                        "--ortho-scale is only valid with --projection orthographic",
                    )));
                }
            }

            Projection::Orthographic => {
                if focal_length.is_some() {
                    return Err(Error::IO(IOError::new(
                        ErrorKind::InvalidInput,
                        "--focal-length is only valid with --projection perspective",
                    )));
                }
                if fov.is_some() {
                    return Err(Error::IO(IOError::new(
                        ErrorKind::InvalidInput,
                        "--fov is only valid with --projection perspective",
                    )));
                }
            }
        }

        let (lens_mode, lens_value) = match (focal_length, fov) {
            (_, Some(fov)) => ("fov", fov),
            (Some(focal), _) => ("focal", focal),
            (None, None) => ("focal", 50.0),
        };
        let ortho_scale_value = ortho_scale.unwrap_or(0.0);

        let subject_names = if camera.subject.is_empty() {
            Vec::new()
        } else {
            resolve_subject_names(&dependencies, &input_fbx, &camera.subject)?
        };

        let result = (|| -> Result<()> {
            let mut args: Vec<OsString> = vec![
                input_fbx.clone().into_os_string(),
                render_path.clone().into_os_string(),
                resolution_x.to_string().into(),
                resolution_y.to_string().into(),
                projection.as_blender_type().into(),
                lens_mode.into(),
                lens_value.to_string().into(),
                ortho_scale_value.to_string().into(),
                near.to_string().into(),
                far.to_string().into(),
                renderer.as_blender_engine().into(),
                samples.to_string().into(),
                lighting.as_blender_mode().into(),
            ];
            args.extend(camera.to_python_args(&subject_names));

            let stdout =
                dependencies.exec_temp_blender_scripts(&FBX_RENDER_PY, [&COMMON_PY], &args)?;

            if display_in_terminal {
                dependencies.display_image_in_terminal(&render_path)?;
            }

            camera.emit_print_camera(&dependencies, &stdout)?;

            Ok(())
        })();

        if let Some(dir) = temp_dir {
            let _ = dependencies.remove_dir_all(&dir);
        }

        result
    }
}

fn resolve_subject_names(
    dependencies: &impl Dependencies,
    input_fbx: &PathBuf,
    select: &[String],
) -> Result<Vec<String>> {
    let args: [&OsStr; 1] = [input_fbx.as_ref()];
    let stdout = dependencies.exec_temp_blender_script(&FBX_HIERARCHY_JSON_PY, args)?;
    let json = extract_json(&stdout, b'[', b']')?;
    let entries = dependencies.parse_hierarchy_json(json)?;

    let candidate_paths: Vec<&str> = entries.iter().map(|(_, path, _)| path.as_str()).collect();
    let matched = match_hierarchy_paths(dependencies, select, &candidate_paths)?;

    let matched_names: Vec<String> = entries
        .iter()
        .zip(matched.iter())
        .filter(|&(_, &m)| m)
        .map(|((name, _, _), _)| name.clone())
        .collect();

    if matched_names.is_empty() {
        return Err(Error::IO(IOError::new(
            ErrorKind::NotFound,
            format!("no object matched any of: {}", select.join(", ")),
        )));
    }

    Ok(matched_names)
}
