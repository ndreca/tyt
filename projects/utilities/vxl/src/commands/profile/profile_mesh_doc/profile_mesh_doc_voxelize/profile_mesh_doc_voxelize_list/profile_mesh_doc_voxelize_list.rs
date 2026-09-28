use crate::{
    Dependencies, Result, cli_value_parser,
    commands::{list_profiles, load_mesh_doc_voxelize_profile_set},
};
use clap::Parser;
use voxsmith::operations::profile::ProfileListLayout;

/// Lists the profiles `mesh-doc voxelize --profile` can apply, grouped by the
/// `.vxlconfig` that supplies each.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct ProfileMeshDocVoxelizeList {
    /// How to lay out the listing.
    #[arg(
        value_name = "layout",
        long,
        default_value = "box-hierarchy",
        value_parser = cli_value_parser::<ProfileListLayout>()
    )]
    layout: ProfileListLayout,
}

impl ProfileMeshDocVoxelizeList {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profiles = load_mesh_doc_voxelize_profile_set(&dependencies)?;
        Ok(dependencies.write_stdout(list_profiles(&profiles, self.layout).as_bytes())?)
    }
}
