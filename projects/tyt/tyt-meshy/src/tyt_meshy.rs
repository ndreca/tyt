use crate::{
    Dependencies, Result,
    commands::{Mesh, Poll, Texture},
};
use clap::Subcommand;

/// Works with the Meshy API.
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum TytMeshy {
    #[command(name = "mesh")]
    Mesh(Mesh),

    #[command(name = "poll")]
    Poll(Poll),

    #[command(name = "texture")]
    Texture(Texture),
}

impl TytMeshy {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        match self {
            TytMeshy::Mesh(mesh) => mesh.execute(dependencies),
            TytMeshy::Poll(poll) => poll.execute(dependencies),
            TytMeshy::Texture(texture) => texture.execute(dependencies),
        }
    }
}
