use crate::{
    ProfileSet, QuantizeProfile, ResolvePrefsPaths, Result, commands::PaletteConfig,
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `palette quantize` can apply: the `.vxlconfig` layers'
/// `palette.quantize.profiles`, the user's first and the working directory's
/// last.
pub(crate) fn load_palette_quantize_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<QuantizeProfile>> {
    load_profile_set(
        dependencies,
        "palette",
        BTreeMap::new(),
        |config: PaletteConfig| config.quantize.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{ResolvePrefsPaths, commands::load_palette_quantize_profile_set};
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
    fn the_layers_load_from_the_palette_quantize_section() {
        let cascade = Cascade {
            files: BTreeMap::from([
                (
                    PathBuf::from("/home/.vxlconfig"),
                    r#"{ "palette": { "quantize": { "profiles": {
                        "retro": { "maxMaterials": 16 },
                    } } } }"#,
                ),
                (
                    PathBuf::from("/repo/.vxlconfig"),
                    r#"{ "palette": { "quantize": { "profiles": {
                        "retro": { "maxMaterials": 8 },
                    } } } }"#,
                ),
            ]),
        };

        let profiles = load_palette_quantize_profile_set(&cascade).unwrap();
        let retro = profiles.get("the test", "retro").unwrap();
        assert_eq!(retro.max_materials.unwrap().get(), 8);
    }
}
