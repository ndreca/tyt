use crate::{
    COMMON_PY, Dependencies, Error, FBX_HIERARCHY_JSON_PY, Result, Script, embed_blender_script,
    extract_json, match_hierarchy_paths,
};
use clap::Parser;
use std::{
    ffi::OsStr,
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// The Blender script that extracts one mesh into its own FBX file.
const FBX_EXTRACT_MESH_PY: Script = embed_blender_script!("fbx_extract_mesh.py");

/// Extracts a single mesh matching a selection pattern from the input FBX file,
/// unparents it keeping the world transform, deletes everything else, and
/// renames the mesh object and its datablock to `output-mesh-name`. Exactly one
/// mesh must match.
#[derive(Clone, Debug, Parser)]
pub struct Extract {
    /// The input FBX file.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// Gitignore-style pattern selecting mesh hierarchy paths, with more passed
    /// via `--select`. A bare name matches at any depth, a slashed pattern
    /// anchors to a scene root, and `**/name/**` selects a whole subtree.
    /// Exactly one mesh must match.
    #[arg(value_name = "pattern")]
    pattern: String,

    /// Additional patterns unioned with the positional `pattern`. Repeatable.
    #[arg(value_name = "select", long)]
    select: Vec<String>,

    /// The output FBX file to write. When omitted, the input file is
    /// overwritten.
    #[arg(value_name = "output-fbx")]
    output_fbx: Option<PathBuf>,

    /// The name of the output mesh object and datablock. When omitted, the
    /// matched mesh's name is used.
    #[arg(
        value_name = "output-mesh-name",
        short = 'o',
        long = "output-mesh-name",
        conflicts_with = "output_mesh_name_arg"
    )]
    output_mesh_name_flag: Option<String>,

    /// The name of the output mesh object and datablock. When omitted, the
    /// matched mesh's name is used.
    #[arg(
        value_name = "output-mesh-name",
        conflicts_with = "output_mesh_name_flag"
    )]
    output_mesh_name_arg: Option<String>,
}

impl Extract {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Extract {
            input_fbx,
            pattern,
            select,
            output_fbx,
            output_mesh_name_flag,
            output_mesh_name_arg,
        } = self;

        let output_fbx = output_fbx.as_ref().unwrap_or(&input_fbx);

        // Phase 1: get hierarchy JSON from Blender.
        let args: [&OsStr; 1] = [input_fbx.as_ref()];
        let stdout = dependencies.exec_temp_blender_script(&FBX_HIERARCHY_JSON_PY, args)?;

        let json = extract_json(&stdout, b'[', b']')?;
        let entries = dependencies.parse_hierarchy_json(json)?;

        // Filter to MESH objects only.
        let meshes: Vec<&(String, String, String)> = entries
            .iter()
            .filter(|(_, _, obj_type)| obj_type == "MESH")
            .collect();

        let mut patterns = vec![pattern];
        patterns.extend(select);

        let candidate_paths: Vec<&str> = meshes.iter().map(|(_, path, _)| path.as_str()).collect();
        let matched = match_hierarchy_paths(&dependencies, &patterns, &candidate_paths)?;

        let matched_meshes: Vec<&&(String, String, String)> = meshes
            .iter()
            .zip(matched.iter())
            .filter(|&(_, &m)| m)
            .map(|(entry, _)| entry)
            .collect();

        if matched_meshes.is_empty() {
            return Err(Error::IO(IOError::new(
                ErrorKind::NotFound,
                format!("no mesh matched any of: {}", patterns.join(", ")),
            )));
        }

        if matched_meshes.len() > 1 {
            let names: Vec<&str> = matched_meshes
                .iter()
                .map(|(_, path, _)| path.as_str())
                .collect();
            return Err(Error::IO(IOError::new(
                ErrorKind::InvalidInput,
                format!(
                    "{} meshes matched (expected 1): {}",
                    matched_meshes.len(),
                    names.join(", "),
                ),
            )));
        }

        let (mesh_name, _, _) = matched_meshes[0];

        let output_mesh_name = output_mesh_name_flag
            .as_ref()
            .or(output_mesh_name_arg.as_ref())
            .unwrap_or(mesh_name);

        // Phase 2: extract the matched mesh.
        let args: [&OsStr; 4] = [
            input_fbx.as_ref(),
            mesh_name.as_ref(),
            output_fbx.as_ref(),
            output_mesh_name.as_ref(),
        ];

        dependencies.exec_temp_blender_scripts_with_stdout(
            &FBX_EXTRACT_MESH_PY,
            [&COMMON_PY],
            args,
        )?;

        Ok(())
    }
}
