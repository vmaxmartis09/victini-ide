use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct DiscoveredAgent {
    pub name: String,
    pub role: String,
    pub path: PathBuf,
    pub content: String,
}

pub struct SkillDiscovery;

impl SkillDiscovery {
    pub fn scan_workspace(root: &Path) -> (Vec<DiscoveredSkill>, Vec<DiscoveredAgent>) {
        let mut skills = Vec::new();
        let mut agents = Vec::new();

        // 1. Scan skills (.agents/skills, .claude/skills, ai/skills, skills/)
        let skill_dirs = [
            root.join(".agents/skills"),
            root.join(".claude/skills"),
            root.join("ai/skills"),
            root.join("skills"),
        ];

        for s_dir in &skill_dirs {
            if s_dir.exists() && s_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(s_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let name = path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            if skills.iter().any(|s: &DiscoveredSkill| s.name == name) {
                                continue;
                            }
                            let skill_md = path.join("SKILL.md");
                            let content = fs::read_to_string(&skill_md).unwrap_or_default();
                            let desc = Self::extract_frontmatter_field(&content, "description")
                                .unwrap_or_else(|| "Active project skill".to_string());
                            skills.push(DiscoveredSkill {
                                name,
                                path,
                                description: desc,
                                content,
                            });
                        }
                    }
                }
            }
        }

        // 2. Scan agents (.claude/agents, .agents/plugins, agents/)
        let agent_dirs = [
            root.join(".claude/agents"),
            root.join(".agents/plugins/fe-master-team/agents"),
            root.join("agents"),
        ];

        for a_dir in &agent_dirs {
            if a_dir.exists() && a_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(a_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md")
                        {
                            let name = path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            if agents.iter().any(|a: &DiscoveredAgent| a.name == name) {
                                continue;
                            }
                            let content = fs::read_to_string(&path).unwrap_or_default();
                            let role = Self::extract_frontmatter_field(&content, "role")
                                .unwrap_or_else(|| "Autonomous Specialist Agent".to_string());
                            agents.push(DiscoveredAgent {
                                name,
                                role,
                                path,
                                content,
                            });
                        }
                    }
                }
            }
        }

        (skills, agents)
    }

    fn extract_frontmatter_field(content: &str, key: &str) -> Option<String> {
        let prefix = format!("{}:", key);
        for line in content.lines().take(15) {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix(&prefix) {
                let val = rest.trim().trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
        None
    }

    pub fn create_or_update_skill(
        root: &Path,
        name: &str,
        description: &str,
        body: &str,
    ) -> Result<PathBuf> {
        let clean_name = name.trim().replace(' ', "-").to_lowercase();
        let skill_dir = root.join("skills").join(&clean_name);
        fs::create_dir_all(&skill_dir)?;
        let skill_md = if body.trim().starts_with("---") {
            body.to_string()
        } else {
            format!(
                "---\nname: {}\ndescription: {}\n---\n\n# {}\n\n{}\n",
                clean_name,
                description.trim(),
                clean_name,
                body.trim()
            )
        };
        fs::write(skill_dir.join("SKILL.md"), skill_md)?;
        Ok(skill_dir)
    }

    pub fn create_or_update_agent(
        root: &Path,
        name: &str,
        role: &str,
        system_prompt: &str,
    ) -> Result<PathBuf> {
        let clean_name = name.trim().replace(' ', "_").to_lowercase();
        let agent_dir = root.join(".claude/agents");
        fs::create_dir_all(&agent_dir)?;
        let agent_file = agent_dir.join(format!("{}.md", clean_name));
        let agent_md = if system_prompt.trim().starts_with("---") {
            system_prompt.to_string()
        } else {
            format!(
                "---\nname: {}\nrole: {}\n---\n\n# Subagent {}\n\n{}\n",
                clean_name,
                role.trim(),
                clean_name,
                system_prompt.trim()
            )
        };
        fs::write(&agent_file, agent_md)?;
        Ok(agent_file)
    }

    pub fn delete_skill(path: &Path) -> Result<()> {
        if path.exists() && path.is_dir() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    pub fn delete_agent(path: &Path) -> Result<()> {
        if path.exists() && path.is_file() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
