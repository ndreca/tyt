use crate::{
    CreateDirAll, CreateDirLink,
    commands::{AGENTS_DIR, CLAUDE_DIR, SKILLS_DIR},
};
use std::{
    io::Result as IOResult,
    path::{Path, PathBuf},
};

/// The relative target `../.agents/skills` survives a move or a fresh clone of
/// `root`.
pub fn link_claude_skills(
    dependencies: &(impl CreateDirAll + CreateDirLink),
    root: &Path,
) -> IOResult<PathBuf> {
    let link = root.join(CLAUDE_DIR).join(SKILLS_DIR);

    dependencies.create_dir_all(&root.join(AGENTS_DIR).join(SKILLS_DIR))?;

    let target = Path::new("..").join(AGENTS_DIR).join(SKILLS_DIR);

    dependencies.create_dir_link(&target, &link)?;

    Ok(link)
}
