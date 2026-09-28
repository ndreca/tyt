use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{MeshProfile, ObjectConfig, built_in_profiles},
    load_profile_set,
};
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `object mesh` can apply: the built-ins under the `.vxlconfig`
/// layers' `object.mesh.profiles`, the user's first and the working
/// directory's last.
pub(crate) fn load_mesh_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<MeshProfile>> {
    load_profile_set(
        dependencies,
        "object",
        built_in_profiles(),
        |config: ObjectConfig| config.mesh.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        ProfileSet, ResolvePrefsPaths,
        commands::{MeshProfile, load_mesh_profile_set},
    };
    use std::{
        collections::BTreeMap,
        io::Result as IOResult,
        path::{Path, PathBuf},
    };
    use ty_preferences::{Dependencies as PreferencesDependencies, PrefsPaths};

    /// A cascade over in-memory files, the working directory `/repo/sub` under
    /// the git root `/repo` and the user's home `/home`.
    struct Cascade {
        in_repository: bool,
        files: BTreeMap<PathBuf, &'static str>,
    }

    impl Cascade {
        fn new(in_repository: bool, files: &[(&str, &'static str)]) -> Self {
            Cascade {
                in_repository,
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
                git_root: self.in_repository.then(|| PathBuf::from("/repo")),
                user: Some(PathBuf::from("/home")),
            })
        }
    }

    /// The values of the profile `name`.
    fn values(profiles: &ProfileSet<MeshProfile>, name: &str) -> Vec<String> {
        profiles.get("the test", name).unwrap().values.clone()
    }

    #[test]
    fn the_layers_load_in_application_order_over_the_built_ins() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": {
                        "a": { "values": ["a = 1"] },
                        "orm": { "values": ["orm = 1"] },
                    } } } }"#,
                ),
                (
                    "/repo/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "a": { "values": ["a = 2"] } } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "b": { "values": ["b = a"] } } } } }"#,
                ),
            ],
        );

        let profiles = load_mesh_profile_set(&cascade).unwrap();

        assert_eq!(values(&profiles, "a"), ["a = 2"]);
        assert_eq!(values(&profiles, "b"), ["b = a"]);
        assert_eq!(values(&profiles, "orm"), ["orm = 1"]);
        assert!(profiles.get("the test", "pbr").is_ok());

        let groups: Vec<_> = profiles
            .by_origin()
            .into_iter()
            .map(|(origin, names)| format!("{origin}: {}", names.join(" ")))
            .collect();
        assert_eq!(
            groups,
            [
                "built in: albedo defaults emissive pbr",
                "/home/.vxlconfig: orm",
                "/repo/.vxlconfig: a",
                "/repo/sub/.vxlconfig: b",
            ]
        );
    }

    #[test]
    fn outside_a_repository_the_user_layer_alone_loads() {
        let cascade = Cascade::new(
            false,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "a": { "values": ["a = 1"] } } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "b": { "values": ["b = a"] } } } } }"#,
                ),
            ],
        );

        let profiles = load_mesh_profile_set(&cascade).unwrap();

        assert_eq!(values(&profiles, "a"), ["a = 1"]);
        assert!(profiles.get("the test", "b").is_err());
    }

    #[test]
    fn a_layer_without_the_section_supplies_nothing() {
        let cascade = Cascade::new(true, &[("/repo/.vxlconfig", r#"{ "fs": {} }"#)]);

        let profiles = load_mesh_profile_set(&cascade).unwrap();

        assert!(profiles.get("the test", "pbr").is_ok());
    }

    #[test]
    fn a_broken_layer_errors_naming_the_file() {
        for text in [
            r#"{ "object": { "mesh": { "profiles": { "a": { "slot": 1 } } } } }"#,
            r#"{ "object": { "mesh": { "profiles": { "a": } } } }"#,
        ] {
            let cascade = Cascade::new(true, &[("/repo/.vxlconfig", text)]);

            let error = load_mesh_profile_set(&cascade).unwrap_err().to_string();
            assert!(error.contains("`.vxlconfig`"), "{error}");
        }
    }
}
