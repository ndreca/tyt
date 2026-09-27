use crate::commands::ProfileSet;
use voxsmith::operations::profile::{ProfileListGroup, ProfileListLayout, profile_list};

/// The profiles of `profiles` in `layout`: one group per origin in cascade
/// order, each holding the names it supplies in name order.
pub(crate) fn list_profiles(profiles: &ProfileSet, layout: ProfileListLayout) -> String {
    let groups: Vec<_> = profiles
        .by_origin()
        .into_iter()
        .map(|(origin, names)| ProfileListGroup {
            origin: origin.to_string(),
            profiles: names.into_iter().map(str::to_owned).collect(),
        })
        .collect();

    profile_list(&groups, layout)
}

#[cfg(test)]
mod tests {
    use super::list_profiles;
    use crate::commands::{MeshProfile, ProfileSet};
    use std::{collections::BTreeMap, path::PathBuf};
    use voxsmith::operations::profile::ProfileListLayout;

    #[test]
    fn the_origins_group_in_cascade_order() {
        let layer = |name: &str| BTreeMap::from([(name.to_owned(), MeshProfile::default())]);
        let profiles = ProfileSet::layered([
            (PathBuf::from("/home/.vxlconfig"), layer("matte")),
            (PathBuf::from("/repo/.vxlconfig"), layer("orm")),
        ]);

        assert_eq!(
            list_profiles(&profiles, ProfileListLayout::TextRows),
            "built in         albedo defaults emissive pbr\n\
             \n\
             /home/.vxlconfig matte\n\
             \n\
             /repo/.vxlconfig orm\n"
        );
    }
}
