use crate::{
    Error, Result,
    commands::{Profile, ProfileOrigin, built_in_profiles},
};
use std::{collections::BTreeMap, path::PathBuf};

/// The profiles a run can apply, one namespace merged from the layers of the
/// cascade. Each name reads from the last layer supplying it, wholesale, and
/// remembers that layer as its origin.
#[derive(Clone, Debug)]
pub(crate) struct ProfileSet {
    /// The layers in cascade order, the built-ins first.
    cascade: Vec<ProfileOrigin>,

    profiles: BTreeMap<String, (ProfileOrigin, Profile)>,
}

impl ProfileSet {
    /// The built-ins alone, the bottom layer of the cascade.
    #[cfg(test)]
    pub(crate) fn built_in() -> Self {
        Self::layered([])
    }

    /// The built-ins under `layers`, each a `.vxlconfig` path with its
    /// profiles, each name reading from the last layer supplying it.
    pub(crate) fn layered(
        layers: impl IntoIterator<Item = (PathBuf, BTreeMap<String, Profile>)>,
    ) -> Self {
        let mut cascade = vec![ProfileOrigin::BuiltIn];

        let mut profiles: BTreeMap<_, _> = built_in_profiles()
            .into_iter()
            .map(|(name, profile)| (name, (ProfileOrigin::BuiltIn, profile)))
            .collect();

        for (path, layer) in layers {
            let origin = ProfileOrigin::File(path);

            profiles.extend(
                layer
                    .into_iter()
                    .map(|(name, profile)| (name, (origin.clone(), profile))),
            );

            cascade.push(origin);
        }

        ProfileSet { cascade, profiles }
    }

    /// A set holding `profiles` alone, as built-ins.
    #[cfg(test)]
    pub(crate) fn from_profiles(profiles: BTreeMap<String, Profile>) -> Self {
        ProfileSet {
            cascade: vec![ProfileOrigin::BuiltIn],
            profiles: profiles
                .into_iter()
                .map(|(name, profile)| (name, (ProfileOrigin::BuiltIn, profile)))
                .collect(),
        }
    }

    /// Each origin supplying a name, in cascade order, with the names it
    /// supplies in name order. An origin every later layer overrode is
    /// absent.
    pub(crate) fn by_origin(&self) -> Vec<(&ProfileOrigin, Vec<&str>)> {
        self.cascade
            .iter()
            .map(|origin| {
                let names = self
                    .profiles
                    .iter()
                    .filter(|(_, (supplier, _))| supplier == origin)
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>();

                (origin, names)
            })
            .filter(|(_, names)| !names.is_empty())
            .collect()
    }

    /// The profile `name`, which `origin` asks for.
    pub(crate) fn get(&self, origin: &str, name: &str) -> Result<&Profile> {
        self.profiles.get(name).map(|(_, profile)| profile).ok_or_else(|| {
            let names: Vec<_> = self
                .profiles
                .keys()
                .map(|name| format!("`{name}`"))
                .collect();

            Error::usage(format!(
                "{origin} asks for the profile `{name}`, which is not defined. The profiles are {}",
                names.join(", ")
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ProfileSet;
    use crate::commands::Profile;
    use std::{collections::BTreeMap, path::PathBuf};

    /// A layer at `path` holding the profile `name` with `values`.
    fn layer(path: &str, name: &str, values: &[&str]) -> (PathBuf, BTreeMap<String, Profile>) {
        let profile = Profile {
            values: values.iter().map(|value| (*value).to_owned()).collect(),
            ..Profile::default()
        };

        (
            PathBuf::from(path),
            BTreeMap::from([(name.to_owned(), profile)]),
        )
    }

    #[test]
    fn a_later_layer_replaces_a_name_wholesale() {
        let profiles = ProfileSet::layered([
            layer("/home/.vxlconfig", "orm", &["orm = 1"]),
            layer("/repo/.vxlconfig", "a", &["a = 1"]),
            layer("/repo/sub/.vxlconfig", "orm", &["orm = 2"]),
        ]);

        let orm = profiles.get("the test", "orm").unwrap();
        assert_eq!(orm.values, ["orm = 2"]);
        assert!(orm.materials.is_empty());
        assert!(profiles.get("the test", "a").is_ok());
        assert!(profiles.get("the test", "pbr").is_ok());
    }

    #[test]
    fn by_origin_runs_the_cascade_and_drops_an_overridden_layer() {
        let profiles = ProfileSet::layered([
            layer("/home/.vxlconfig", "orm", &["orm = 1"]),
            layer("/repo/.vxlconfig", "a", &["a = 1"]),
            layer("/repo/sub/.vxlconfig", "orm", &["orm = 2"]),
        ]);

        let groups: Vec<_> = profiles
            .by_origin()
            .into_iter()
            .map(|(origin, names)| (origin.to_string(), names))
            .collect();

        assert_eq!(
            groups,
            [
                (
                    "built in".to_owned(),
                    vec!["albedo", "defaults", "emissive", "pbr"]
                ),
                ("/repo/.vxlconfig".to_owned(), vec!["a"]),
                ("/repo/sub/.vxlconfig".to_owned(), vec!["orm"]),
            ]
        );
    }

    #[test]
    fn an_undefined_profile_errors_with_the_defined_ones() {
        let profiles = ProfileSet::built_in();

        assert!(profiles.get("--profile", "orm").is_ok());

        let error = profiles.get("--profile", "metal").unwrap_err().to_string();
        assert!(error.contains("`metal`"), "{error}");
        assert!(error.contains("`albedo`, `defaults`"), "{error}");
    }
}
