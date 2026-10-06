use crate::{
    AgentSkill, Dependencies, PathKind, ReadFile, ReadPathKind, Result, WriteFile, WriteStdout,
    commands::{
        AGENTS_DIR, CLAUDE_DIR, ClaudeSkillsLink, IntegrationRoot, SKILL_FILE, SKILLS_DIR,
        read_claude_skills_link,
    },
};
use clap::Parser;
use std::{io::Error as IOError, path::Path, str};

const INSTALL_HELP: &str = "\
The skill goes in .agents/skills/<skill>/SKILL.md at the git root, or under the
home directory with --user. Codex reads .agents/skills. Claude Code reads only
.claude/skills, which `vxl integration agents-link` links to .agents/skills.

Installing again replaces a SKILL.md vxl wrote, which upgrades the skill to this
vxl. A file vxl did not write stays, and the install fails.";

/// Installs a skill where coding agents load it.
#[derive(Clone, Debug, Parser)]
#[command(name = "install", after_help = INSTALL_HELP)]
pub struct IntegrationSkillInstall {
    /// The skill to install.
    #[arg(value_name = "skill")]
    skill: AgentSkill,

    #[command(flatten)]
    root: IntegrationRoot,
}

impl IntegrationSkillInstall {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let root = self.root.resolve(&dependencies)?;

        install_skill(&dependencies, self.skill, &root)
    }
}

fn install_skill(
    dependencies: &(impl ReadFile + ReadPathKind + WriteFile + WriteStdout),
    skill: AgentSkill,
    root: &Path,
) -> Result<()> {
    let claude_skills = root.join(CLAUDE_DIR).join(SKILLS_DIR);

    let problem = match read_claude_skills_link(dependencies, root)? {
        ClaudeSkillsLink::Missing | ClaudeSkillsLink::Linked => None,

        ClaudeSkillsLink::LinkedElsewhere { target } => Some(format!(
            "{} links to {}, not .agents/skills",
            claude_skills.display(),
            target.display()
        )),

        ClaudeSkillsLink::Directory => Some(format!(
            "{} is a directory, not a link to .agents/skills; \
             `vxl integration agents-link move` moves its skills there and links it",
            claude_skills.display()
        )),

        ClaudeSkillsLink::File => Some(format!(
            "{} is a file, not a link to .agents/skills",
            claude_skills.display()
        )),
    };

    if let Some(problem) = problem {
        return Err(IOError::other(problem).into());
    }

    let path = root
        .join(AGENTS_DIR)
        .join(SKILLS_DIR)
        .join(skill.name())
        .join(SKILL_FILE);

    match dependencies.read_path_kind(&path)? {
        PathKind::Missing => {}

        PathKind::File => {
            if !written_by_vxl(&dependencies.read_file(&path)?, skill) {
                return Err(IOError::other(format!(
                    "{} is a SKILL.md vxl did not write; vxl replaces only its own",
                    path.display()
                ))
                .into());
            }
        }

        PathKind::Directory | PathKind::Symlink { .. } => {
            return Err(IOError::other(format!("{} is not a file", path.display())).into());
        }
    }

    dependencies.write_file(&path, skill.skill_md().as_bytes())?;

    Ok(dependencies.write_stdout(format!("installed {}\n", path.display()).as_bytes())?)
}

/// Whether `bytes` open with the frontmatter vxl writes for `skill`.
fn written_by_vxl(bytes: &[u8], skill: AgentSkill) -> bool {
    let Ok(text) = str::from_utf8(bytes) else {
        return false;
    };

    let mut lines = text.lines();

    if lines.next() != Some("---") {
        return false;
    }

    let frontmatter: Vec<_> = lines.take_while(|line| *line != "---").collect();

    let name = format!("name: {}", skill.name());

    frontmatter.contains(&name.as_str())
        && frontmatter
            .iter()
            .any(|line| line.starts_with("  vxl-version: "))
}

#[cfg(test)]
mod tests {
    use crate::{
        AgentSkill, CapturedStdout,
        commands::integration::integration_skill::integration_skill_install::install_skill,
    };
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    use std::{fs, path::PathBuf};
    use tempfile::TempDir;

    const SKILL_MD: &str = ".agents/skills/vxl-model/SKILL.md";

    fn install(root: &TempDir) -> Result<String, String> {
        let dependencies = CapturedStdout::default();

        install_skill(&dependencies, AgentSkill::VxlModel, root.path())
            .map_err(|e| e.to_string())?;

        Ok(dependencies.stdout())
    }

    fn read(root: &TempDir, path: &str) -> String {
        fs::read_to_string(root.path().join(path)).unwrap()
    }

    #[test]
    fn install_writes_the_skill_under_agents_skills_and_reports_the_path() {
        let root = TempDir::new().unwrap();

        let stdout = install(&root).unwrap();

        assert_eq!(read(&root, SKILL_MD), AgentSkill::VxlModel.skill_md());
        assert_eq!(
            stdout,
            format!("installed {}\n", root.path().join(SKILL_MD).display())
        );
    }

    #[test]
    fn install_replaces_a_skill_md_an_older_vxl_wrote() {
        let root = TempDir::new().unwrap();
        let older = AgentSkill::VxlModel
            .skill_md()
            .replace(env!("CARGO_PKG_VERSION"), "0.0.1");
        fs::create_dir_all(root.path().join(".agents/skills/vxl-model")).unwrap();
        fs::write(root.path().join(SKILL_MD), older).unwrap();

        install(&root).unwrap();

        assert_eq!(read(&root, SKILL_MD), AgentSkill::VxlModel.skill_md());
    }

    #[test]
    fn install_keeps_a_skill_md_vxl_did_not_write_and_fails() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".agents/skills/vxl-model")).unwrap();
        fs::write(root.path().join(SKILL_MD), "---\nname: vxl-model\n---\n").unwrap();

        let error = install(&root).unwrap_err();

        assert!(error.ends_with("is a SKILL.md vxl did not write; vxl replaces only its own"));
        assert_eq!(read(&root, SKILL_MD), "---\nname: vxl-model\n---\n");
    }

    #[test]
    fn install_fails_on_a_claude_skills_directory_and_writes_nothing() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude/skills")).unwrap();

        let error = install(&root).unwrap_err();

        assert_eq!(
            error,
            format!(
                "{} is a directory, not a link to .agents/skills; \
                 `vxl integration agents-link move` moves its skills there and links it",
                root.path().join(".claude/skills").display()
            )
        );
        assert!(!root.path().join(".agents").exists());
    }

    #[test]
    fn install_fails_on_a_claude_skills_file() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        fs::write(root.path().join(".claude/skills"), "").unwrap();

        let error = install(&root).unwrap_err();

        assert!(error.ends_with(".claude/skills is a file, not a link to .agents/skills"));
    }

    #[test]
    fn install_ignores_a_codex_skills_directory() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".codex/skills")).unwrap();

        install(&root).unwrap();

        assert_eq!(read(&root, SKILL_MD), AgentSkill::VxlModel.skill_md());
    }

    #[cfg(unix)]
    #[test]
    fn install_reaches_claude_through_a_relative_or_absolute_link() {
        for absolute in [false, true] {
            let root = TempDir::new().unwrap();
            let target = if absolute {
                root.path().join(".agents/skills")
            } else {
                PathBuf::from("../.agents/skills")
            };
            fs::create_dir_all(root.path().join(".claude")).unwrap();
            symlink(target, root.path().join(".claude/skills")).unwrap();

            install(&root).unwrap();

            assert_eq!(
                read(&root, ".claude/skills/vxl-model/SKILL.md"),
                AgentSkill::VxlModel.skill_md()
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn install_fails_on_a_claude_skills_link_elsewhere() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        symlink("../skills", root.path().join(".claude/skills")).unwrap();

        let error = install(&root).unwrap_err();

        assert!(error.ends_with(".claude/skills links to ../skills, not .agents/skills"));
        assert!(!root.path().join(".agents").exists());
    }
}
