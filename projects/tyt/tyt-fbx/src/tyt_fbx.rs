use crate::{
    Dependencies, Result,
    commands::{CreatePointCloud, Extract, Hierarchy, Modify, Reduce, Rename, Render, Transform},
};
use clap::Subcommand;

/// Works with FBX files.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytFbx {
    #[command(name = "create-point-cloud")]
    CreatePointCloud(CreatePointCloud),

    #[command(name = "extract")]
    Extract(Extract),

    #[command(name = "hierarchy")]
    Hierarchy(Hierarchy),

    #[command(name = "modify")]
    Modify(Modify),

    #[command(name = "reduce")]
    Reduce(Reduce),

    #[command(name = "rename")]
    Rename(Rename),

    #[command(name = "render")]
    Render(Render),

    #[command(name = "transform")]
    Transform(Transform),
}

impl TytFbx {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            TytFbx::CreatePointCloud(create_point_cloud) => {
                create_point_cloud.execute(dependencies)
            }

            TytFbx::Extract(extract) => extract.execute(dependencies),

            TytFbx::Hierarchy(hierarchy) => hierarchy.execute(dependencies),

            TytFbx::Modify(modify) => modify.execute(dependencies),

            TytFbx::Reduce(reduce) => reduce.execute(dependencies),

            TytFbx::Rename(rename) => rename.execute(dependencies),

            TytFbx::Render(render) => render.execute(dependencies),

            TytFbx::Transform(transform) => transform.execute(dependencies),
        }
    }
}
