use crate::{COMMON_PY, Dependencies, Result, Script, embed_blender_script};
use clap::Parser;
use std::{ffi::OsStr, path::PathBuf};

/// The Blender script that joins every mesh into one.
const FBX_REDUCE_TO_SINGLE_MESH_PY: Script = embed_blender_script!("fbx_reduce_to_single_mesh.py");

/// Collapses all mesh objects in the input FBX into a single joined mesh.
/// Clears parenting while keeping world transforms, deletes now-unused empties,
/// joins all meshes, and renames the result to `output-mesh-name`.
#[derive(Clone, Debug, Parser)]
pub struct Reduce {
    /// The input FBX file.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// The name for the output mesh object and datablock.
    #[arg(value_name = "output-mesh-name")]
    output_mesh_name: String,

    /// The output FBX file to write. If not provided, the input file will be
    /// overwritten.
    #[arg(value_name = "output-fbx")]
    output_fbx: Option<PathBuf>,
}

impl Reduce {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Reduce {
            input_fbx,
            output_mesh_name,
            output_fbx,
        } = self;

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

        let args: [&OsStr; 3] = [
            input_fbx.as_ref(),
            output_fbx.as_ref(),
            output_mesh_name.as_ref(),
        ];

        dependencies.exec_temp_blender_scripts_with_stdout(
            &FBX_REDUCE_TO_SINGLE_MESH_PY,
            [&COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
