use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{MeshDocConfig, MeshDocVoxelizeProfile},
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `mesh-doc voxelize` can apply: the `.vxlconfig` layers'
/// `meshDoc.voxelize.profiles`, the user's first and the working directory's
/// last.
pub(crate) fn load_mesh_doc_voxelize_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<MeshDocVoxelizeProfile>> {
    load_profile_set(
        dependencies,
        "meshDoc",
        BTreeMap::new(),
        |config: MeshDocConfig| config.voxelize.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{ResolvePrefsPaths, commands::load_mesh_doc_voxelize_profile_set};
    use std::{
        collections::BTreeMap,
        io::Result as IOResult,
        path::{Path, PathBuf},
    };
    use ty_preferences::{Dependencies as PreferencesDependencies, PrefsPaths};
    use voxsmith::operations::mesh_doc::GridResolution;

    /// A cascade over in-memory files, the working directory `/repo/sub` under
    /// the git root `/repo` and the user's home `/home`.
    struct Cascade {
        files: BTreeMap<PathBuf, &'static str>,
    }

    impl PreferencesDependencies for Cascade {
        fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
            Ok(self.files.get(path).map(|text| text.as_bytes().to_vec()))
        }

        fn write_file(&self, _: &Path, _: &[u8]) -> IOResult<()> {
            unreachable!("loading never writes")
        }
    }

    impl ResolvePrefsPaths for Cascade {
        fn resolve_prefs_paths(&self) -> IOResult<PrefsPaths> {
            Ok(PrefsPaths {
                cwd: PathBuf::from("/repo/sub"),
                git_root: Some(PathBuf::from("/repo")),
                user: Some(PathBuf::from("/home")),
            })
        }
    }

    #[test]
    fn the_layers_load_from_the_mesh_doc_voxelize_section() {
        let cascade = Cascade {
            files: BTreeMap::from([
                (
                    PathBuf::from("/home/.vxlconfig"),
                    r#"{ "meshDoc": { "voxelize": { "profiles": {
                        "fine": { "voxelSize": 0.1 },
                    } } } }"#,
                ),
                (
                    PathBuf::from("/repo/sub/.vxlconfig"),
                    r#"{ "meshDoc": { "voxelize": { "profiles": {
                        "fine": { "voxelSize": 0.05 },
                    } } } }"#,
                ),
            ]),
        };

        let profiles = load_mesh_doc_voxelize_profile_set(&cascade).unwrap();
        let fine = profiles.get("the test", "fine").unwrap();
        assert_eq!(
            fine.grid_resolution().unwrap(),
            Some(GridResolution::VoxelSize(0.05))
        );
    }
}
