use crate::operations::mesh::{ProfileListGroup, ProfileListLayout};
use treegrid::{
    TreeGrid, TreeGridHierarchyOptions, TreeGridJsonValue, TreeGridJsonValueCells, TreeGridLabel,
    TreeGridNestedTableOptions, TreeGridRenderHierarchy, TreeGridRenderJson, TreeGridRenderRows,
    TreeGridRenderTables, TreeGridRowsOptions, TreeGridTableShape,
};

/// Renders `groups` in `layout`, one root per group in the given order over
/// its profiles.
pub fn profile_list(groups: &[ProfileListGroup], layout: ProfileListLayout) -> String {
    let grid = build_grid(groups);

    match layout {
        ProfileListLayout::Hierarchy => grid.render_hierarchy(
            &TreeGridHierarchyOptions::default()
                .with_bare_roots(true)
                .with_value_children(true),
        ),
        ProfileListLayout::Rows => grid.render_rows(&TreeGridRowsOptions::default()),
        ProfileListLayout::Tables => grid.render_tables(&TreeGridTableShape::Nested(
            TreeGridNestedTableOptions::default(),
        )),
        ProfileListLayout::JsonPretty => grid.render_json_pretty(),
        ProfileListLayout::JsonCompact => grid.render_json_compact(),
    }
}

/// The forest every layout renders: a bare root per group over its profiles
/// as values.
fn build_grid(groups: &[ProfileListGroup]) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);

    for group in groups {
        let root_id = grid.retain_root(TreeGridLabel::bare(group.origin.clone()));

        for profile in &group.profiles {
            grid.push_value(root_id, TreeGridJsonValue::new(profile.clone()));
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::profile_list;
    use crate::operations::mesh::{ProfileListGroup, ProfileListLayout};

    /// Four built-ins, then a user layer and a repository layer.
    fn groups() -> Vec<ProfileListGroup> {
        let group = |origin: &str, profiles: &[&str]| ProfileListGroup {
            origin: origin.to_owned(),
            profiles: profiles.iter().map(|name| (*name).to_owned()).collect(),
        };

        vec![
            group("built in", &["albedo", "defaults", "emissive", "pbr"]),
            group("/home/.vxlconfig", &["matte"]),
            group("/repo/.vxlconfig", &["orm"]),
        ]
    }

    #[test]
    fn hierarchy_branches_each_group_in_order() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::Hierarchy),
            "built in\n\
             ├ albedo\n\
             ├ defaults\n\
             ├ emissive\n\
             └ pbr\n\
             \n\
             /home/.vxlconfig\n\
             └ matte\n\
             \n\
             /repo/.vxlconfig\n\
             └ orm\n"
        );
    }

    #[test]
    fn rows_put_each_group_beside_its_profiles() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::Rows),
            "built in         albedo defaults emissive pbr\n\
             \n\
             /home/.vxlconfig matte\n\
             \n\
             /repo/.vxlconfig orm\n"
        );
    }

    #[test]
    fn tables_give_each_group_a_column() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::Tables),
            "| #   | built in | /home/.vxlconfig | /repo/.vxlconfig |\n\
             | --- | -------- | ---------------- | ---------------- |\n\
             | 0   | albedo   | matte            | orm              |\n\
             | 1   | defaults |                  |                  |\n\
             | 2   | emissive |                  |                  |\n\
             | 3   | pbr      |                  |                  |\n"
        );
    }

    #[test]
    fn json_records_each_group_with_its_profiles() {
        let compact = profile_list(&groups(), ProfileListLayout::JsonCompact);
        assert_eq!(
            compact,
            "[{\"label\":\"built in\",\"values\":[\"albedo\",\"defaults\",\"emissive\",\"pbr\"]},\
             {\"label\":\"/home/.vxlconfig\",\"values\":[\"matte\"]},\
             {\"label\":\"/repo/.vxlconfig\",\"values\":[\"orm\"]}]\n"
        );

        let pretty = profile_list(&groups(), ProfileListLayout::JsonPretty);
        assert!(pretty.lines().count() > 3, "{pretty}");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&pretty).unwrap(),
            serde_json::from_str::<serde_json::Value>(&compact).unwrap()
        );
    }

    #[test]
    fn no_groups_render_nothing() {
        assert_eq!(profile_list(&[], ProfileListLayout::Hierarchy), "");
        assert_eq!(profile_list(&[], ProfileListLayout::JsonCompact), "[]\n");
    }
}
