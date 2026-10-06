use crate::{
    PathKind, ReadPathKind,
    commands::{AGENTS_DIR, CLAUDE_DIR, ClaudeSkillsLink, SKILLS_DIR},
};
use std::{
    io::Result as IOResult,
    path::{Component, Path, PathBuf},
};

/// What sits at `.claude/skills` under `root`. A symlink counts as linked when
/// its target points to `root`'s `.agents/skills`, relatively or absolutely.
pub fn read_claude_skills_link(
    dependencies: &impl ReadPathKind,
    root: &Path,
) -> IOResult<ClaudeSkillsLink> {
    let link = root.join(CLAUDE_DIR).join(SKILLS_DIR);

    let skills_link = match dependencies.read_path_kind(&link)? {
        PathKind::Missing => ClaudeSkillsLink::Missing,

        PathKind::File => ClaudeSkillsLink::File,

        PathKind::Directory => ClaudeSkillsLink::Directory,

        PathKind::Symlink { target } => {
            let agents_skills = root.join(AGENTS_DIR).join(SKILLS_DIR);

            if normalize(&root.join(CLAUDE_DIR).join(&target)) == normalize(&agents_skills) {
                ClaudeSkillsLink::Linked
            } else {
                ClaudeSkillsLink::LinkedElsewhere { target }
            }
        }
    };

    Ok(skills_link)
}

/// `path` with its `.` and `..` components resolved lexically.
fn normalize(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}

            Component::ParentDir => {
                normal.pop();
            }

            component => normal.push(component),
        }
    }

    normal
}
