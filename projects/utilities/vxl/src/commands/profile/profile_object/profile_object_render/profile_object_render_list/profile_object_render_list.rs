use crate::{
    Dependencies, Result, cli_value_parser,
    commands::{list_profiles, load_render_profile_set},
};
use clap::Parser;
use voxsmith::operations::profile::ProfileListLayout;

/// Lists the profiles `object render --profile` can apply, grouped by the
/// `.vxlconfig` that supplies each. Built-in profiles group under `built in`.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct ProfileObjectRenderList {
    /// How to lay out the listing.
    #[arg(
        value_name = "layout",
        long,
        default_value = "box-hierarchy",
        value_parser = cli_value_parser::<ProfileListLayout>()
    )]
    layout: ProfileListLayout,
}

impl ProfileObjectRenderList {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profiles = load_render_profile_set(&dependencies)?;

        Ok(dependencies.write_stdout(list_profiles(&profiles, self.layout).as_bytes())?)
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ProfileObjectRenderList;
    use clap::Parser;
    use voxsmith::operations::profile::ProfileListLayout;

    #[test]
    fn the_layout_defaults_to_a_box_hierarchy() {
        let list = ProfileObjectRenderList::try_parse_from(["list"]).unwrap();
        assert_eq!(list.layout, ProfileListLayout::BoxHierarchy);

        let list =
            ProfileObjectRenderList::try_parse_from(["list", "--layout", "text-rows"]).unwrap();
        assert_eq!(list.layout, ProfileListLayout::TextRows);

        assert!(ProfileObjectRenderList::try_parse_from(["list", "model.voxj"]).is_err());
    }
}
