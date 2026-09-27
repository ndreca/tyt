use crate::{
    TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind, TreeGridListsLabelMode,
    TreeGridListsOptions, TreeGridOptions,
};

impl TreeGridOptions {
    /// The lists render's options, rejecting every option it does not
    /// consume. An unset label means `header`.
    pub fn resolve_lists(&self) -> Result<TreeGridListsOptions, TreeGridError> {
        self.no_hierarchy_options()?;
        self.no_width()?;
        self.no_table_shape()?;
        let label = match self.label.unwrap_or(TreeGridLabelKind::Header) {
            TreeGridLabelKind::None => {
                self.no_header_level()?;
                TreeGridListsLabelMode::None
            }
            TreeGridLabelKind::Concat => return Err(TreeGridError::LabelConcatWithLists),
            TreeGridLabelKind::Header => TreeGridListsLabelMode::Header(TreeGridHeaderOptions {
                level: self.level(),
            }),
        };
        Ok(TreeGridListsOptions { label })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridError, TreeGridHeaderOptions, TreeGridLabelKind, TreeGridListsLabelMode,
        TreeGridListsOptions, TreeGridOptions, TreeGridTableShapeKind,
    };
    use std::num::NonZeroU8;

    #[test]
    fn default_options_resolve_to_header_lists() {
        assert_eq!(
            TreeGridOptions::default().resolve_lists(),
            Ok(TreeGridListsOptions::default())
        );
    }

    #[test]
    fn a_header_label_carries_the_level() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::Header)
            .with_header_level(NonZeroU8::new(3).unwrap());

        assert_eq!(
            options.resolve_lists(),
            Ok(
                TreeGridListsOptions::default().with_label(TreeGridListsLabelMode::Header(
                    TreeGridHeaderOptions::default().with_level(NonZeroU8::new(3).unwrap())
                ))
            )
        );
    }

    #[test]
    fn a_none_label_drops_the_headings() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::None);

        assert_eq!(
            options.resolve_lists(),
            Ok(TreeGridListsOptions::default().with_label(TreeGridListsLabelMode::None))
        );
    }

    #[test]
    fn a_concat_label_is_rejected() {
        let options = TreeGridOptions::default().with_label(TreeGridLabelKind::Concat);

        assert_eq!(
            options.resolve_lists(),
            Err(TreeGridError::LabelConcatWithLists)
        );
    }

    #[test]
    fn a_level_without_headings_is_rejected() {
        let options = TreeGridOptions::default()
            .with_label(TreeGridLabelKind::None)
            .with_header_level(NonZeroU8::new(2).unwrap());

        assert_eq!(
            options.resolve_lists(),
            Err(TreeGridError::HeaderLevelWithoutHeaders)
        );
    }

    #[test]
    fn options_of_other_layouts_are_rejected() {
        assert_eq!(
            TreeGridOptions::default().with_width(72).resolve_lists(),
            Err(TreeGridError::WidthWithoutRows)
        );
        assert_eq!(
            TreeGridOptions::default()
                .with_table_shape(TreeGridTableShapeKind::Nested)
                .resolve_lists(),
            Err(TreeGridError::TableShapeWithoutTables)
        );
        assert_eq!(
            TreeGridOptions::default()
                .with_value_children(true)
                .resolve_lists(),
            Err(TreeGridError::ValueChildrenWithoutHierarchy)
        );
    }
}
