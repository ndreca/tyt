use crate::{COMMON_PY, HierarchyEntry, MeshWithUvs, Result, Script};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use ty_math::{TySrgbaF32, TyVector3F64};

/// The side effects FBX commands perform.
pub trait Dependencies {
    /// Creates a fresh temporary directory and returns its path.
    fn create_temp_dir(&self) -> Result<PathBuf>;

    /// Runs Blender headless on `script_py_path` with `script_dir` on the
    /// Python path and returns its stdout.
    fn exec_blender_script<
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    >(
        &self,
        script_dir: P1,
        script_py_path: P2,
        args: I,
    ) -> Result<Vec<u8>>;

    /// Removes `path` and everything under it.
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Writes `contents` to `path`.
    fn write_file<P: AsRef<Path>>(&self, path: P, contents: &[u8]) -> Result<()>;

    /// Writes `contents` to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;

    /// Parses the mesh JSON the faces-and-vertices script prints.
    fn parse_mesh_with_uvs_json(&self, json: &[u8]) -> Result<MeshWithUvs>;

    /// Serializes points and their per-texture color layers to JSON.
    fn serialize_points_and_colors_json(
        &self,
        points: &[TyVector3F64],
        colors: &[Vec<TySrgbaF32>],
    ) -> Result<Vec<u8>>;

    /// Loads the image at `path` as RGBA8 pixels with its width and height.
    fn load_image_rgba(&self, path: &Path) -> Result<(Vec<u8>, u32, u32)>;

    /// Displays the image at `path` inline in the terminal.
    fn display_image_in_terminal(&self, path: &Path) -> Result<()>;

    /// Tests each `(path, is_dir)` candidate against gitignore `patterns`.
    fn match_paths(&self, patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>>;

    /// Parses the hierarchy JSON into `(name, path, type)` triples.
    fn parse_hierarchy_json(&self, json: &[u8]) -> Result<Vec<(String, String, String)>>;

    /// Parses the hierarchy JSON with its transform, bounds, and extents
    /// payloads.
    fn parse_hierarchy_payloads_json(&self, json: &[u8]) -> Result<Vec<HierarchyEntry>>;

    // --- Provided methods ---

    /// Writes `script_py` and `additional_scripts` to a temporary directory,
    /// runs `script_py` in Blender, and returns its stdout.
    fn exec_temp_blender_scripts<
        'a,
        I1: IntoIterator<Item = &'a Script<'a>>,
        I2: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    >(
        &self,
        script_py: &'a Script<'a>,
        additional_scripts: I1,
        args: I2,
    ) -> Result<Vec<u8>> {
        let temp_dir = self.create_temp_dir()?;

        let result = (|| {
            let script_py_path = temp_dir.join(script_py.relative_file_path);
            self.write_file(&script_py_path, script_py.content.as_bytes())?;

            for additional_script in additional_scripts.into_iter() {
                let additional_script_path = temp_dir.join(additional_script.relative_file_path);
                self.write_file(
                    &additional_script_path,
                    additional_script.content.as_bytes(),
                )?;
            }

            self.exec_blender_script(&temp_dir, script_py_path, args)
        })();

        let output = result?;
        self.remove_dir_all(&temp_dir)?;

        Ok(output)
    }

    /// Runs [`exec_temp_blender_scripts`](Self::exec_temp_blender_scripts) and
    /// forwards Blender's stdout.
    fn exec_temp_blender_scripts_with_stdout<
        'a,
        I: IntoIterator<Item = &'a Script<'a>>,
        S: AsRef<OsStr>,
    >(
        &self,
        script_py: &'a Script<'a>,
        additional_scripts: I,
        args: impl IntoIterator<Item = S>,
    ) -> Result<()> {
        let stdout = self.exec_temp_blender_scripts(script_py, additional_scripts, args)?;
        self.write_stdout(&stdout)?;
        Ok(())
    }

    /// Runs `script_py` beside the common helper module and returns its stdout.
    fn exec_temp_blender_script<'a, I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        script_py: &'a Script<'a>,
        args: I,
    ) -> Result<Vec<u8>> {
        self.exec_temp_blender_scripts(script_py, [&COMMON_PY], args)
    }

    /// Runs `script_py` beside the common helper module and forwards Blender's
    /// stdout.
    fn exec_temp_blender_script_with_stdout<'a, I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        script_py: &'a Script<'a>,
        args: I,
    ) -> Result<()> {
        self.exec_temp_blender_scripts_with_stdout(script_py, [&COMMON_PY], args)
    }
}
