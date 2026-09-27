use crate::{
    Dependencies, Result,
    commands::{Hierarchy, MeshDoc, Node, Object, ObjectVoxels, Palette, Profile, VoxDoc},
};
use clap::Subcommand;

/// A command-line tool for working with voxels.
// The command line parses once, so the variants' sizes never matter.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Vxl {
    #[command(name = "hierarchy")]
    Hierarchy(Hierarchy),
    #[command(name = "mesh-doc")]
    MeshDoc(MeshDoc),
    #[command(name = "node")]
    Node(Node),
    #[command(name = "object")]
    Object(Object),
    #[command(name = "object-voxels")]
    ObjectVoxels(ObjectVoxels),
    #[command(name = "palette")]
    Palette(Palette),
    #[command(name = "profile")]
    Profile(Profile),
    #[command(name = "vox-doc")]
    VoxDoc(VoxDoc),
}

impl Vxl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            Vxl::Hierarchy(hierarchy) => hierarchy.execute(dependencies),
            Vxl::MeshDoc(mesh_doc) => mesh_doc.execute(dependencies),
            Vxl::Node(node) => node.execute(dependencies),
            Vxl::Object(object) => object.execute(dependencies),
            Vxl::ObjectVoxels(object_voxels) => object_voxels.execute(dependencies),
            Vxl::Palette(palette) => palette.execute(dependencies),
            Vxl::Profile(profile) => profile.execute(dependencies),
            Vxl::VoxDoc(vox_doc) => vox_doc.execute(dependencies),
        }
    }
}
