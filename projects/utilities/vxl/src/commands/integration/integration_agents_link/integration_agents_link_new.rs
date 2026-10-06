use crate::{
    CreateDirAll, CreateDirLink, Dependencies, ReadPathKind, Result, WriteStdout,
    commands::{
        CLAUDE_DIR, ClaudeSkillsLink, IntegrationRoot, SKILLS_DIR, link_claude_skills,
        read_claude_skills_link,
    },
};
use clap::Parser;
use std::{io::Error as IOError, path::Path};

const NEW_HELP: &str = "\
Claude Code reads skills only from .claude/skills, so the link lets it load the
skills in .agents/skills. Codex reads .agents/skills itself and needs no link.
On a real .claude/skills directory the link fails and the directory stays.
`vxl integration agents-link move` moves its skills into .agents/skills first.";

/// Links a missing `.claude/skills` to `.agents/skills`.
#[derive(Clone, Debug, Parser)]
#[command(name = "new", after_help = NEW_HELP)]
pub struct IntegrationAgentsLinkNew {
    #[command(flatten)]
    root: IntegrationRoot,
}

impl IntegrationAgentsLinkNew {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let root = self.root.resolve(&dependencies)?;

        link_new(&dependencies, &root)
    }
}

fn link_new(
    dependencies: &(impl CreateDirAll + CreateDirLink + ReadPathKind + WriteStdout),
    root: &Path,
) -> Result<()> {
    let claude_skills = root.join(CLAUDE_DIR).join(SKILLS_DIR);

    let problem = match read_claude_skills_link(dependencies, root)? {
        ClaudeSkillsLink::Missing => None,

        ClaudeSkillsLink::Linked => {
            let line = format!(
                "{} already links to .agents/skills\n",
                claude_skills.display()
            );

            return Ok(dependencies.write_stdout(line.as_bytes())?);
        }

        ClaudeSkillsLink::LinkedElsewhere { target } => Some(format!(
            "{} links to {}, not .agents/skills",
            claude_skills.display(),
            target.display()
        )),

        ClaudeSkillsLink::Directory => Some(format!(
            "{} is a directory; `vxl integration agents-link move` moves its skills into \
             .agents/skills and links it",
            claude_skills.display()
        )),

        ClaudeSkillsLink::File => Some(format!("{} is a file", claude_skills.display())),
    };

    if let Some(problem) = problem {
        return Err(IOError::other(problem).into());
    }

    let link = link_claude_skills(dependencies, root)?;

    let line = format!("linked {} to .agents/skills\n", link.display());

    Ok(dependencies.write_stdout(line.as_bytes())?)
}

#[cfg(test)]
mod tests {
    use crate::{
        CapturedStdout,
        commands::integration::integration_agents_link::integration_agents_link_new::link_new,
    };
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::{fs, path::Path};
    use tempfile::TempDir;

    fn new(root: &TempDir) -> Result<String, String> {
        let dependencies = CapturedStdout::default();

        link_new(&dependencies, root.path()).map_err(|e| e.to_string())?;

        Ok(dependencies.stdout())
    }

    #[cfg(unix)]
    #[test]
    fn new_creates_agents_skills_and_a_relative_link_to_it() {
        let root = TempDir::new().unwrap();

        let stdout = new(&root).unwrap();

        let link = root.path().join(".claude/skills");
        assert_eq!(
            fs::read_link(&link).unwrap(),
            Path::new("../.agents/skills")
        );
        assert!(root.path().join(".agents/skills").is_dir());
        assert_eq!(
            stdout,
            format!("linked {} to .agents/skills\n", link.display())
        );
    }

    #[cfg(unix)]
    #[test]
    fn new_leaves_an_existing_link_and_says_so() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        symlink("../.agents/skills", root.path().join(".claude/skills")).unwrap();

        let stdout = new(&root).unwrap();

        assert!(stdout.ends_with(".claude/skills already links to .agents/skills\n"));
        assert!(!root.path().join(".agents").exists());
    }

    #[test]
    fn new_fails_on_a_directory_and_points_to_move() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude/skills/mine")).unwrap();

        let error = new(&root).unwrap_err();

        assert!(error.ends_with(
            ".claude/skills is a directory; `vxl integration agents-link move` moves its \
             skills into .agents/skills and links it"
        ));
        assert!(root.path().join(".claude/skills/mine").is_dir());
        assert!(!root.path().join(".agents").exists());
    }

    #[cfg(unix)]
    #[test]
    fn new_fails_on_a_link_elsewhere() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        symlink("../skills", root.path().join(".claude/skills")).unwrap();

        let error = new(&root).unwrap_err();

        assert!(error.ends_with(".claude/skills links to ../skills, not .agents/skills"));
    }

    #[test]
    fn new_fails_on_a_file() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        fs::write(root.path().join(".claude/skills"), "").unwrap();

        let error = new(&root).unwrap_err();

        assert!(error.ends_with(".claude/skills is a file"));
    }
}
