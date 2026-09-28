use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{PaletteConfig, PaletteShowProfile},
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `palette show` can apply: the `.vxlconfig` layers'
/// `palette.show.profiles`, the user's first and the working directory's last.
pub(crate) fn load_palette_show_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<PaletteShowProfile>> {
    load_profile_set(
        dependencies,
        "palette",
        BTreeMap::new(),
        |config: PaletteConfig| config.show.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{ResolvePrefsPaths, commands::load_palette_show_profile_set};
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

    impl Cascade {
        fn new(files: &[(&str, &'static str)]) -> Self {
            Cascade {
                files: files
                    .iter()
                    .map(|&(path, text)| (PathBuf::from(path), text))
                    .collect(),
            }
        }
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
    fn the_layers_load_in_application_order_with_no_built_ins() {
        let cascade = Cascade::new(&[
            (
                "/home/.vxlconfig",
                r#"{ "palette": { "show": { "profiles": {
                    "pbr": { "properties": [{ "property": "baseColor" }] },
                } } } }"#,
            ),
            (
                "/repo/.vxlconfig",
                r#"{ "palette": { "show": { "profiles": {
                    "pbr": { "properties": [{ "property": "metallic" }, { "property": "roughness" }] },
                } } } }"#,
            ),
        ]);

        let profiles = load_palette_show_profile_set(&cascade).unwrap();

        assert_eq!(profiles.get("the test", "pbr").unwrap().properties.len(), 2);

        let groups: Vec<_> = profiles
            .by_origin()
            .into_iter()
            .map(|(origin, names)| format!("{origin}: {}", names.join(" ")))
            .collect();
        assert_eq!(groups, ["/repo/.vxlconfig: pbr"]);
    }

    #[test]
    fn a_broken_selector_errors_naming_the_file() {
        let cascade = Cascade::new(&[(
            "/repo/.vxlconfig",
            r#"{ "palette": { "show": { "profiles": {
                "pbr": { "properties": [{ "property": "baseColor", "reading": "rainbow" }] },
            } } } }"#,
        )]);

        let error = load_palette_show_profile_set(&cascade)
            .unwrap_err()
            .to_string();
        assert!(error.contains("`.vxlconfig`"), "{error}");
    }
}
