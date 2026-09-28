use crate::{
    ProfileSet, QuantizeProfile, ResolvePrefsPaths, Result, commands::ObjectConfig,
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `object voxels quantize` can apply: the `.vxlconfig` layers'
/// `object.voxels.quantize.profiles`, the user's first and the working
/// directory's last.
pub(crate) fn load_object_voxels_quantize_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<QuantizeProfile>> {
    load_profile_set(
        dependencies,
        "object",
        BTreeMap::new(),
        |config: ObjectConfig| config.voxels.quantize.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{ResolvePrefsPaths, commands::load_object_voxels_quantize_profile_set};
    use std::{
        collections::BTreeMap,
        io::Result as IOResult,
        path::{Path, PathBuf},
    };
    use ty_preferences::{Dependencies as PreferencesDependencies, PrefsPaths};

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
    fn the_layers_load_from_the_object_voxels_quantize_section() {
        let cascade = Cascade {
            files: BTreeMap::from([(
                PathBuf::from("/repo/.vxlconfig"),
                r#"{ "object": { "voxels": { "quantize": { "profiles": {
                    "roughness-8": { "maxMaterials": 8, "property": "roughness" },
                } } } } }"#,
            )]),
        };

        let profiles = load_object_voxels_quantize_profile_set(&cascade).unwrap();
        let profile = profiles.get("the test", "roughness-8").unwrap();
        assert_eq!(profile.property.as_deref(), Some("roughness"));
    }

    #[test]
    fn a_palette_only_element_errors_naming_the_file() {
        let cascade = Cascade {
            files: BTreeMap::from([(
                PathBuf::from("/repo/.vxlconfig"),
                r#"{ "object": { "voxels": { "quantize": { "profiles": {
                    "first": { "index": 0 },
                } } } } }"#,
            )]),
        };

        let error = load_object_voxels_quantize_profile_set(&cascade)
            .unwrap_err()
            .to_string();
        assert!(error.contains("`.vxlconfig`"), "{error}");
    }
}
