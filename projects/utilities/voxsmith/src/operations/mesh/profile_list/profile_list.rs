use crate::operations::mesh::{ProfileListGroup, ProfileListLayout};
use treegrid::{
    TreeGrid, TreeGridBoxHierarchyOptions, TreeGridJsonValue, TreeGridJsonValueCells,
    TreeGridLabel, TreeGridMdListsOptions, TreeGridNestedTableOptions, TreeGridRenderBoxHierarchy,
    TreeGridRenderBoxTables, TreeGridRenderJson, TreeGridRenderMdLists, TreeGridRenderMdTables,
    TreeGridRenderTextRows, TreeGridTableShape, TreeGridTableShapeKind, TreeGridTextRowsOptions,
};

/// Renders `groups` in `layout`, one node per group in the given order over
/// its profiles. The markdown lists layout alone titles them `profiles`,
/// putting one top-level heading over a section per group.
pub fn profile_list(groups: &[ProfileListGroup], layout: ProfileListLayout) -> String {
    match layout {
        ProfileListLayout::BoxHierarchy => build_grid(groups, None).render_box_hierarchy(
            &TreeGridBoxHierarchyOptions::default()
                .with_bare_roots(true)
                .with_value_children(true),
        ),
        ProfileListLayout::BoxTables => {
            build_grid(groups, None).render_box_tables(TreeGridTableShapeKind::Nested)
        }
        ProfileListLayout::JsonCompact => build_grid(groups, None).render_json_compact(),
        ProfileListLayout::JsonPretty => build_grid(groups, None).render_json_pretty(),
        ProfileListLayout::MdLists => {
            build_grid(groups, Some("profiles")).render_md_lists(&TreeGridMdListsOptions::default())
        }
        ProfileListLayout::MdTables => build_grid(groups, None).render_md_tables(
            &TreeGridTableShape::Nested(TreeGridNestedTableOptions::default()),
        ),
        ProfileListLayout::TextRows => {
            build_grid(groups, None).render_text_rows(&TreeGridTextRowsOptions::default())
        }
    }
}

/// The grid the layouts render: a bare node per group over its profiles as
/// values. The nodes are roots, or children of a bare `title` root when one
/// is given.
fn build_grid(
    groups: &[ProfileListGroup],
    title: Option<&str>,
) -> TreeGrid<TreeGridJsonValueCells> {
    let mut grid = TreeGrid::with_cells(TreeGridJsonValueCells);

    let title_id = title.map(|title| grid.retain_root(TreeGridLabel::bare(title)));

    for group in groups {
        let label = TreeGridLabel::bare(group.origin.clone());

        let group_id = match title_id {
            Some(title_id) => grid.retain_child(title_id, label),
            None => grid.retain_root(label),
        };

        for profile in &group.profiles {
            grid.push_value(group_id, TreeGridJsonValue::new(profile.clone()));
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
            profile_list(&groups(), ProfileListLayout::BoxHierarchy),
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
    fn box_tables_give_each_group_a_boxed_column() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::BoxTables),
            "┌───┬──────────┬──────────────────┬──────────────────┐\n\
             │ # │ built in │ /home/.vxlconfig │ /repo/.vxlconfig │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 0 │ albedo   │ matte            │ orm              │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 1 │ defaults │                  │                  │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 2 │ emissive │                  │                  │\n\
             ├───┼──────────┼──────────────────┼──────────────────┤\n\
             │ 3 │ pbr      │                  │                  │\n\
             └───┴──────────┴──────────────────┴──────────────────┘\n"
        );
    }

    #[test]
    fn lists_head_each_group_over_its_numbered_profiles() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::MdLists),
            "# profiles\n\
             \n\
             ## built in\n\
             \n\
             1. albedo\n\
             2. defaults\n\
             3. emissive\n\
             4. pbr\n\
             \n\
             ## /home/.vxlconfig\n\
             \n\
             1. matte\n\
             \n\
             ## /repo/.vxlconfig\n\
             \n\
             1. orm\n"
        );
    }

    #[test]
    fn rows_put_each_group_beside_its_profiles() {
        assert_eq!(
            profile_list(&groups(), ProfileListLayout::TextRows),
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
            profile_list(&groups(), ProfileListLayout::MdTables),
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
        assert_eq!(profile_list(&[], ProfileListLayout::BoxHierarchy), "");
        assert_eq!(profile_list(&[], ProfileListLayout::MdLists), "");
        assert_eq!(profile_list(&[], ProfileListLayout::JsonCompact), "[]\n");
    }
}
