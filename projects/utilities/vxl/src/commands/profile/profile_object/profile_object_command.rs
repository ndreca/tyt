use crate::{
    Dependencies, Result,
    commands::{ProfileObjectMesh, ProfileObjectVoxels},
};
use clap::Subcommand;

/// The `profile object` command group.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum ProfileObjectCommand {
    #[command(name = "mesh")]
    ProfileObjectMesh(ProfileObjectMesh),
    #[command(name = "voxels")]
    ProfileObjectVoxels(ProfileObjectVoxels),
}

impl ProfileObjectCommand {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            ProfileObjectCommand::ProfileObjectMesh(mesh) => mesh.execute(dependencies),
            ProfileObjectCommand::ProfileObjectVoxels(voxels) => voxels.execute(dependencies),
        }
    }
}
