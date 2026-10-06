use crate::{
    CreateDirAll, CreateDirLink, Dependencies, ListDir, PathKind, ReadPathKind, RemoveDir,
    RenamePath, Result, WriteStdout,
    commands::{
        AGENTS_DIR, CLAUDE_DIR, ClaudeSkillsLink, IntegrationRoot, SKILLS_DIR, link_claude_skills,
        read_claude_skills_link,
    },
};
use clap::Parser;
use std::{io::Error as IOError, path::Path};

const MOVE_HELP: &str = "\
Claude Code reads skills only from .claude/skills, so the link lets it load the
skills in .agents/skills. Codex reads .agents/skills itself and needs no link.

Everything in .claude/skills moves into .agents/skills before the link replaces
the directory. When a name sits in both directories, the move fails before
anything changes.";

/// Moves the skills in a `.claude/skills` directory into `.agents/skills`, then
/// links `.claude/skills` there.
#[derive(Clone, Debug, Parser)]
#[command(name = "move", after_help = MOVE_HELP)]
pub struct IntegrationAgentsLinkMove {
    #[command(flatten)]
    root: IntegrationRoot,
}

impl IntegrationAgentsLinkMove {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let root = self.root.resolve(&dependencies)?;

        link_move(&dependencies, &root)
    }
}

fn link_move(
    dependencies: &(
         impl CreateDirAll
         + CreateDirLink
         + ListDir
         + ReadPathKind
         + RemoveDir
         + RenamePath
         + WriteStdout
     ),
    root: &Path,
) -> Result<()> {
    let claude_skills = root.join(CLAUDE_DIR).join(SKILLS_DIR);

    let problem = match read_claude_skills_link(dependencies, root)? {
        ClaudeSkillsLink::Directory => None,

        ClaudeSkillsLink::Linked => {
            let line = format!(
                "{} already links to .agents/skills\n",
                claude_skills.display()
            );

            return Ok(dependencies.write_stdout(line.as_bytes())?);
        }

        ClaudeSkillsLink::Missing => Some(format!(
            "{} does not exist; `vxl integration agents-link new` links it",
            claude_skills.display()
        )),

        ClaudeSkillsLink::LinkedElsewhere { target } => Some(format!(
            "{} links to {}, not .agents/skills",
            claude_skills.display(),
            target.display()
        )),

        ClaudeSkillsLink::File => Some(format!("{} is a file", claude_skills.display())),
    };

    if let Some(problem) = problem {
        return Err(IOError::other(problem).into());
    }

    let agents_skills = root.join(AGENTS_DIR).join(SKILLS_DIR);

    let entries = dependencies.list_dir(&claude_skills)?;

    let mut clashes = Vec::new();

    for entry in &entries {
        let name = entry.path.file_name().expect("a listed entry has a name");

        if dependencies.read_path_kind(&agents_skills.join(name))? != PathKind::Missing {
            clashes.push(name.to_string_lossy().into_owned());
        }
    }

    if !clashes.is_empty() {
        clashes.sort();

        return Err(IOError::other(format!(
            "{} and {} both hold {}",
            claude_skills.display(),
            agents_skills.display(),
            clashes.join(", ")
        ))
        .into());
    }

    dependencies.create_dir_all(&agents_skills)?;

    for entry in &entries {
        let name = entry.path.file_name().expect("a listed entry has a name");

        dependencies.rename_path(&entry.path, &agents_skills.join(name))?;
    }

    dependencies.remove_dir(&claude_skills)?;

    let link = link_claude_skills(dependencies, root)?;

    let noun = if entries.len() == 1 {
        "entry"
    } else {
        "entries"
    };

    let line = format!(
        "moved {} {noun} into {}, then linked {} to .agents/skills\n",
        entries.len(),
        agents_skills.display(),
        link.display()
    );

    Ok(dependencies.write_stdout(line.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use crate::{
        CapturedStdout,
        commands::integration::integration_agents_link::integration_agents_link_move::link_move,
    };
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::{fs, path::Path};
    use tempfile::TempDir;

    fn link(root: &TempDir) -> Result<String, String> {
        let dependencies = CapturedStdout::default();

        link_move(&dependencies, root.path()).map_err(|e| e.to_string())?;

        Ok(dependencies.stdout())
    }

    fn write(root: &TempDir, path: &str, contents: &str) {
        let path = root.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn read(root: &TempDir, path: &str) -> String {
        fs::read_to_string(root.path().join(path)).unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn move_moves_every_entry_into_agents_skills_then_links() {
        let root = TempDir::new().unwrap();
        write(&root, ".claude/skills/a/SKILL.md", "a");
        write(&root, ".claude/skills/b/SKILL.md", "b");
        write(&root, ".agents/skills/c/SKILL.md", "c");

        let stdout = link(&root).unwrap();

        let claude_skills = root.path().join(".claude/skills");
        assert_eq!(
            fs::read_link(&claude_skills).unwrap(),
            Path::new("../.agents/skills")
        );
        assert_eq!(read(&root, ".agents/skills/a/SKILL.md"), "a");
        assert_eq!(read(&root, ".claude/skills/b/SKILL.md"), "b");
        assert_eq!(read(&root, ".claude/skills/c/SKILL.md"), "c");
        assert_eq!(
            stdout,
            format!(
                "moved 2 entries into {}, then linked {} to .agents/skills\n",
                root.path().join(".agents/skills").display(),
                claude_skills.display()
            )
        );
    }

    #[test]
    fn move_fails_on_names_in_both_directories_before_moving_anything() {
        let root = TempDir::new().unwrap();
        write(&root, ".claude/skills/a/SKILL.md", "claude a");
        write(&root, ".claude/skills/b/SKILL.md", "claude b");
        write(&root, ".claude/skills/c/SKILL.md", "claude c");
        write(&root, ".agents/skills/c/SKILL.md", "agents c");
        write(&root, ".agents/skills/a/SKILL.md", "agents a");

        let error = link(&root).unwrap_err();

        assert!(error.ends_with(".agents/skills both hold a, c"));
        assert_eq!(read(&root, ".claude/skills/a/SKILL.md"), "claude a");
        assert_eq!(read(&root, ".claude/skills/b/SKILL.md"), "claude b");
        assert!(!root.path().join(".agents/skills/b").exists());
    }

    #[test]
    fn move_fails_on_a_missing_directory_and_points_to_new() {
        let root = TempDir::new().unwrap();

        let error = link(&root).unwrap_err();

        assert!(error.ends_with(
            ".claude/skills does not exist; `vxl integration agents-link new` links it"
        ));
        assert!(!root.path().join(".agents").exists());
    }

    #[cfg(unix)]
    #[test]
    fn move_leaves_an_existing_link_and_says_so() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        symlink("../.agents/skills", root.path().join(".claude/skills")).unwrap();

        let stdout = link(&root).unwrap();

        assert!(stdout.ends_with(".claude/skills already links to .agents/skills\n"));
    }
}
