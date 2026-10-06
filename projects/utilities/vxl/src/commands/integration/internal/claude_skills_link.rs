use std::path::PathBuf;

/// The state of `.claude/skills`, which Claude Code reads in place of
/// `.agents/skills`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaudeSkillsLink {
    /// Nothing.
    Missing,

    /// A symlink to `.agents/skills`.
    Linked,

    /// A symlink to anywhere else.
    LinkedElsewhere {
        /// The target as written.
        target: PathBuf,
    },

    /// A real directory.
    Directory,

    /// A regular file.
    File,
}
