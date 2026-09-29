use crate::ProfileSet;
use voxsmith::operations::profile::{ProfileListGroup, ProfileListLayout, profile_list};

/// The profiles of `profiles` in `layout`: one group per origin in cascade
/// order, each holding the names it supplies in name order.
pub fn list_profiles<P>(profiles: &ProfileSet<P>, layout: ProfileListLayout) -> String {
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
    use crate::{ProfileSet, commands::list_profiles};
    use std::{collections::BTreeMap, path::PathBuf};
    use voxsmith::operations::profile::ProfileListLayout;

    #[test]
    fn the_origins_group_in_cascade_order() {
        let layer = |names: &[&str]| {
            names
                .iter()
                .map(|name| ((*name).to_owned(), ()))
                .collect::<BTreeMap<_, _>>()
        };
        let profiles = ProfileSet::layered(
            layer(&["albedo", "orm", "pbr"]),
            [
                (PathBuf::from("/home/.vxlconfig"), layer(&["matte"])),
                (PathBuf::from("/repo/.vxlconfig"), layer(&["orm"])),
            ],
        );

        assert_eq!(
            list_profiles(&profiles, ProfileListLayout::TextRows),
            "built in         albedo pbr\n\
             \n\
             /home/.vxlconfig matte\n\
             \n\
             /repo/.vxlconfig orm\n"
        );
    }
}
