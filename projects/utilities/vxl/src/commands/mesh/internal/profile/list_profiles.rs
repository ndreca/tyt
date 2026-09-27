use crate::commands::ProfileSet;

/// The profiles of `profiles` as a table, one line per name in name order
/// with the origin supplying it.
pub(crate) fn list_profiles(profiles: &ProfileSet) -> String {
    let width = profiles
        .origins()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);

    profiles
        .origins()
        .map(|(name, origin)| format!("{name:width$}  {origin}\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::list_profiles;
    use crate::commands::{Profile, ProfileSet};
    use std::{collections::BTreeMap, path::PathBuf};

    #[test]
    fn the_names_sort_and_the_origins_align() {
        let layer = |name: &str| BTreeMap::from([(name.to_owned(), Profile::default())]);
        let profiles = ProfileSet::layered([
            (PathBuf::from("/home/.vxlconfig"), layer("matte")),
            (PathBuf::from("/repo/.vxlconfig"), layer("orm")),
        ]);

        assert_eq!(
            list_profiles(&profiles),
            "albedo    built in\n\
             defaults  built in\n\
             emissive  built in\n\
             matte     /home/.vxlconfig\n\
             orm       /repo/.vxlconfig\n\
             pbr       built in\n"
        );
    }
}
