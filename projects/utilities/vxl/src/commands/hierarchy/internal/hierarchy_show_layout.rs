use crate::CliValue;
use voxsmith::operations::hierarchy::HierarchyShowLayout;

impl CliValue for HierarchyShowLayout {
    const VARIANTS: &'static [Self] = &[
        HierarchyShowLayout::BoxHierarchy,
        HierarchyShowLayout::JsonCompact,
        HierarchyShowLayout::JsonPretty,
    ];

    fn name(self) -> &'static str {
        match self {
            HierarchyShowLayout::BoxHierarchy => "box-hierarchy",
            HierarchyShowLayout::JsonCompact => "json-compact",
            HierarchyShowLayout::JsonPretty => "json-pretty",
        }
    }

    fn help(self) -> &'static str {
        match self {
            HierarchyShowLayout::BoxHierarchy => {
                "The scene graph as a box-glyph tree, each entity's tag and view rows inline on \
                 its nodes"
            }
            HierarchyShowLayout::JsonCompact => "The scene graph as single-line JSON records",
            HierarchyShowLayout::JsonPretty => "The scene graph as indented JSON records",
        }
    }
}
