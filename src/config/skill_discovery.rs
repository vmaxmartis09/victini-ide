use anyhow::{bail, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillScope {
    Project,
    Global,
}

impl SkillScope {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Project => "Project",
            Self::Global => "Global",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillSourceMeta {
    pub repo: String,
    pub url: String,
    pub git_ref: String,
    pub commit_sha: String,
    pub subpath: String,
    pub installed_at: String,
}

#[derive(Clone, Debug)]
pub struct DiscoveredSkill {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    /// Full raw SKILL.md content (including frontmatter)
    pub content: String,
    /// Markdown body with YAML frontmatter cleanly stripped
    pub body: String,
    pub scope: SkillScope,
    pub enabled: bool,
    pub source: Option<SkillSourceMeta>,
    pub files: Vec<String>,
    pub has_scripts: bool,
}

#[derive(Clone, Debug)]
pub struct DiscoveredAgent {
    pub name: String,
    pub role: String,
    pub path: PathBuf,
    /// Full raw markdown file content
    pub content: String,
    /// System prompt body with YAML frontmatter cleanly stripped
    pub body: String,
    pub scope: SkillScope,
    pub avatar_key: String,
    pub model: Option<String>,
    pub default_skills: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TrashedItem {
    pub kind_label: String,
    pub name: String,
    pub original_path: PathBuf,
    pub trash_path: PathBuf,
    pub is_dir: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SkillsStateFile {
    disabled_paths: HashMap<String, bool>,
}

pub struct SkillDiscovery;

impl SkillDiscovery {
    pub fn global_skills_dir() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".gemini/config/skills")
    }

    pub fn global_agents_dir() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".gemini/config/agents")
    }

    pub fn victini_dir() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(home).join(".victini");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn state_file_path() -> PathBuf {
        Self::victini_dir().join("skills_state.json")
    }

    fn load_disabled_map() -> HashMap<String, bool> {
        fs::read_to_string(Self::state_file_path())
            .ok()
            .and_then(|s| serde_json::from_str::<SkillsStateFile>(&s).ok())
            .map(|f| f.disabled_paths)
            .unwrap_or_default()
    }

    fn save_disabled_map(map: &HashMap<String, bool>) {
        let st = SkillsStateFile {
            disabled_paths: map.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&st) {
            let _ = fs::write(Self::state_file_path(), json);
        }
    }

    pub fn set_skill_enabled(path: &Path, enabled: bool) {
        let mut map = Self::load_disabled_map();
        let key = path.to_string_lossy().to_string();
        if enabled {
            map.remove(&key);
        } else {
            map.insert(key, true);
        }
        Self::save_disabled_map(&map);
    }

    pub fn scan_workspace(root: &Path) -> (Vec<DiscoveredSkill>, Vec<DiscoveredAgent>) {
        let mut skills = Vec::new();
        let mut agents = Vec::new();
        let disabled_map = Self::load_disabled_map();

        // 1. Scan Project skills first, then Global skills
        let project_skill_dirs = [
            root.join(".agents/skills"),
            root.join(".claude/skills"),
            root.join("ai/skills"),
            root.join("skills"),
        ];
        for s_dir in &project_skill_dirs {
            Self::scan_skills_in_dir(s_dir, SkillScope::Project, &disabled_map, &mut skills);
        }

        let global_skill_dirs = [
            Self::global_skills_dir(),
            Self::victini_dir().join("skills"),
        ];
        for s_dir in &global_skill_dirs {
            Self::scan_skills_in_dir(s_dir, SkillScope::Global, &disabled_map, &mut skills);
        }

        skills.sort_by(|a, b| {
            let scope_ord = (a.scope != SkillScope::Project).cmp(&(b.scope != SkillScope::Project));
            scope_ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        // 2. Scan Project agents generically (including .agents/plugins/*/agents)
        let mut project_agent_dirs = vec![
            root.join(".claude/agents"),
            root.join(".agents/agents"),
            root.join("agents"),
        ];
        let plugins_dir = root.join(".agents/plugins");
        if plugins_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&plugins_dir) {
                for entry in entries.flatten() {
                    let candidate = entry.path().join("agents");
                    if candidate.is_dir() {
                        project_agent_dirs.push(candidate);
                    }
                }
            }
        }
        for a_dir in &project_agent_dirs {
            Self::scan_agents_in_dir(a_dir, SkillScope::Project, &mut agents);
        }

        // Global agents
        let global_agent_dirs = [Self::global_agents_dir()];
        for a_dir in &global_agent_dirs {
            Self::scan_agents_in_dir(a_dir, SkillScope::Global, &mut agents);
        }

        agents.sort_by(|a, b| {
            let scope_ord = (a.scope != SkillScope::Project).cmp(&(b.scope != SkillScope::Project));
            scope_ord.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        (skills, agents)
    }

    fn scan_skills_in_dir(
        s_dir: &Path,
        scope: SkillScope,
        disabled_map: &HashMap<String, bool>,
        skills: &mut Vec<DiscoveredSkill>,
    ) {
        if !s_dir.is_dir() {
            return;
        }
        let Ok(entries) = fs::read_dir(s_dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dir_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if dir_name.starts_with('.') {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            if skills
                .iter()
                .any(|s| s.name == dir_name && s.scope == scope)
            {
                continue;
            }
            let content = fs::read_to_string(&skill_md).unwrap_or_default();
            let (fields, body) = Self::parse_frontmatter(&content);
            let desc = fields
                .get("description")
                .cloned()
                .unwrap_or_else(|| "Active skill".to_string());

            let source = fs::read_to_string(path.join(".victini-source.json"))
                .ok()
                .and_then(|s| serde_json::from_str::<SkillSourceMeta>(&s).ok());

            let (files, has_scripts) = Self::inspect_skill_files(&path);
            let enabled = !disabled_map
                .get(&path.to_string_lossy().to_string())
                .copied()
                .unwrap_or(false);

            skills.push(DiscoveredSkill {
                name: dir_name,
                path,
                description: desc,
                content,
                body,
                scope,
                enabled,
                source,
                files,
                has_scripts,
            });
        }
    }

    fn scan_agents_in_dir(a_dir: &Path, scope: SkillScope, agents: &mut Vec<DiscoveredAgent>) {
        if !a_dir.is_dir() {
            return;
        }
        let Ok(entries) = fs::read_dir(a_dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if agents.iter().any(|a| a.name == name && a.scope == scope) {
                continue;
            }
            let content = fs::read_to_string(&path).unwrap_or_default();
            let (fields, body) = Self::parse_frontmatter(&content);
            let role = fields
                .get("role")
                .or_else(|| fields.get("description"))
                .cloned()
                .unwrap_or_else(|| "Autonomous Specialist Agent".to_string());
            let avatar_key = fields
                .get("avatar")
                .cloned()
                .unwrap_or_else(|| "fe_senior_dev".to_string());
            let model = fields.get("model").cloned().filter(|m| !m.is_empty());
            let default_skills = fields
                .get("skills")
                .map(|raw| {
                    raw.trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();

            agents.push(DiscoveredAgent {
                name,
                role,
                path,
                content,
                body,
                scope,
                avatar_key,
                model,
                default_skills,
            });
        }
    }

    pub fn inspect_skill_files(dir: &Path) -> (Vec<String>, bool) {
        let mut files = Vec::new();
        let mut has_scripts = false;
        for entry in WalkDir::new(dir).max_depth(4).into_iter().flatten() {
            if entry.file_type().is_file() {
                if let Ok(rel) = entry.path().strip_prefix(dir) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if rel_str == ".victini-source.json" {
                        continue;
                    }
                    let lower = rel_str.to_lowercase();
                    if lower.starts_with("scripts/")
                        || lower.ends_with(".sh")
                        || lower.ends_with(".py")
                        || lower.ends_with(".js")
                        || lower.ends_with(".ts")
                        || lower.ends_with(".bat")
                        || lower.ends_with(".ps1")
                    {
                        has_scripts = true;
                    }
                    files.push(rel_str);
                }
            }
        }
        files.sort();
        (files, has_scripts)
    }

    /// Parses YAML frontmatter (including multiline `>` / `|` block scalars and `- item` lists)
    /// and returns `(fields, clean_markdown_body)`.
    pub fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
        let mut fields = HashMap::new();
        let normalized = content.replace("\r\n", "\n");
        let Some(after_open) = normalized.strip_prefix("---\n") else {
            return (fields, normalized.trim().to_string());
        };

        let lines: Vec<&str> = after_open.lines().collect();
        let Some(end_idx) = lines.iter().position(|l| l.trim() == "---") else {
            return (fields, normalized.trim().to_string());
        };

        let fm_lines = &lines[..end_idx];
        let body = lines[end_idx + 1..].join("\n").trim().to_string();

        let mut i = 0;
        while i < fm_lines.len() {
            let line = fm_lines[i];
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                i += 1;
                continue;
            }
            if let Some((k_raw, v_raw)) = line.split_once(':') {
                let key = k_raw.trim().to_lowercase();
                let val_trim = v_raw.trim();

                if val_trim == ">" || val_trim == "|" || val_trim == ">-" || val_trim == "|-" {
                    let folded = val_trim.starts_with('>');
                    let mut block_lines = Vec::new();
                    i += 1;
                    while i < fm_lines.len() {
                        let sub = fm_lines[i];
                        if sub.starts_with(' ') || sub.starts_with('\t') || sub.trim().is_empty() {
                            block_lines.push(sub.trim());
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    let joined = if folded {
                        block_lines
                            .into_iter()
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ")
                    } else {
                        block_lines.join("\n").trim().to_string()
                    };
                    if !joined.is_empty() {
                        fields.insert(key, joined);
                    }
                    continue;
                } else if val_trim.is_empty() {
                    // Check for YAML list items (`  - item`) or indented continuation
                    let mut list_items = Vec::new();
                    i += 1;
                    while i < fm_lines.len() {
                        let sub = fm_lines[i].trim();
                        if let Some(item) = sub.strip_prefix("- ") {
                            list_items
                                .push(item.trim().trim_matches('"').trim_matches('\'').to_string());
                            i += 1;
                        } else if fm_lines[i].starts_with(' ') || fm_lines[i].starts_with('\t') {
                            if !sub.is_empty() {
                                list_items.push(sub.to_string());
                            }
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    if !list_items.is_empty() {
                        fields.insert(key, list_items.join(", "));
                    }
                    continue;
                } else {
                    let clean_val = val_trim
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim()
                        .to_string();
                    if !clean_val.is_empty() {
                        fields.insert(key, clean_val);
                    }
                }
            }
            i += 1;
        }

        (fields, body)
    }

    pub fn sanitize_skill_slug(name: &str) -> String {
        let mut out = String::new();
        let mut prev_dash = false;
        for ch in name.trim().to_lowercase().chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch);
                prev_dash = false;
            } else if (ch == '-' || ch == '_' || ch == ' ') && !prev_dash && !out.is_empty() {
                out.push('-');
                prev_dash = true;
            }
        }
        out.trim_matches('-').to_string()
    }

    pub fn sanitize_agent_slug(name: &str) -> String {
        let mut out = String::new();
        let mut prev_sep = false;
        for ch in name.trim().to_lowercase().chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch);
                prev_sep = false;
            } else if (ch == '_' || ch == '-' || ch == ' ') && !prev_sep && !out.is_empty() {
                out.push('_');
                prev_sep = true;
            }
        }
        out.trim_matches('_').to_string()
    }

    fn yaml_quote(s: &str) -> String {
        let single_line = s.replace('\r', "").replace('\n', " ");
        let trimmed = single_line.trim();
        if trimmed.contains(':')
            || trimmed.contains('#')
            || trimmed.contains('"')
            || trimmed.starts_with('[')
            || trimmed.starts_with('{')
        {
            format!("\"{}\"", trimmed.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            trimmed.to_string()
        }
    }

    /// Saves a skill with cleanly reconstructed frontmatter + body.
    /// If `original_path` is provided and the name/scope changed, migrates the folder cleanly
    /// (preserving scripts/, references/, .victini-source.json) and removes the old folder.
    #[allow(clippy::too_many_arguments)]
    pub fn save_skill(
        project_root: &Path,
        scope: SkillScope,
        name: &str,
        description: &str,
        body: &str,
        original_path: Option<&Path>,
        allow_overwrite: bool,
    ) -> Result<PathBuf> {
        let clean_name = Self::sanitize_skill_slug(name);
        if clean_name.is_empty() {
            bail!("Tên skill không hợp lệ (chỉ dùng chữ cái, số và dấu gạch ngang).");
        }

        let base_dir = match scope {
            SkillScope::Project => project_root.join("skills"),
            SkillScope::Global => Self::global_skills_dir(),
        };
        let target_dir = base_dir.join(&clean_name);

        let is_same_dir = original_path
            .map(|orig| orig == target_dir.as_path())
            .unwrap_or(false);

        if target_dir.exists() && !is_same_dir && !allow_overwrite {
            bail!(
                "Skill '{}' đã tồn tại trong phạm vi {}.",
                clean_name,
                scope.label()
            );
        }

        // If renaming / moving across scopes, copy existing extra files first
        if let Some(orig) = original_path {
            if orig.is_dir() && orig != target_dir.as_path() {
                if target_dir.exists() {
                    fs::remove_dir_all(&target_dir)?;
                }
                Self::copy_dir_recursive(orig, &target_dir)?;
            }
        }

        fs::create_dir_all(&target_dir)?;

        // Strip accidental frontmatter from body so header fields always win
        let (_, clean_body) = if body.trim_start().starts_with("---\n") {
            Self::parse_frontmatter(body)
        } else {
            (HashMap::new(), body.trim().to_string())
        };

        let desc_val = if description.trim().is_empty() {
            "Active skill".to_string()
        } else {
            Self::yaml_quote(description)
        };

        let skill_md = format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}\n",
            clean_name, desc_val, clean_body
        );
        fs::write(target_dir.join("SKILL.md"), skill_md)?;

        // Remove old directory after successful write on rename/move
        if let Some(orig) = original_path {
            if orig.is_dir() && orig != target_dir.as_path() {
                let _ = fs::remove_dir_all(orig);
            }
        }

        Ok(target_dir)
    }

    /// Backwards-compatible helper
    pub fn create_or_update_skill(
        root: &Path,
        name: &str,
        description: &str,
        body: &str,
    ) -> Result<PathBuf> {
        Self::save_skill(root, SkillScope::Project, name, description, body, None, true)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_agent(
        project_root: &Path,
        scope: SkillScope,
        name: &str,
        role: &str,
        avatar_key: &str,
        model: Option<&str>,
        default_skills: &[String],
        system_prompt: &str,
        original_path: Option<&Path>,
        allow_overwrite: bool,
    ) -> Result<PathBuf> {
        let clean_name = Self::sanitize_agent_slug(name);
        if clean_name.is_empty() {
            bail!("Tên agent không hợp lệ.");
        }

        let agent_dir = match scope {
            SkillScope::Project => project_root.join(".claude/agents"),
            SkillScope::Global => Self::global_agents_dir(),
        };
        fs::create_dir_all(&agent_dir)?;
        let target_file = agent_dir.join(format!("{}.md", clean_name));

        let is_same_file = original_path
            .map(|orig| orig == target_file.as_path())
            .unwrap_or(false);
        if target_file.exists() && !is_same_file && !allow_overwrite {
            bail!("Agent '{}' đã tồn tại.", clean_name);
        }

        let (_, clean_body) = if system_prompt.trim_start().starts_with("---\n") {
            Self::parse_frontmatter(system_prompt)
        } else {
            (HashMap::new(), system_prompt.trim().to_string())
        };

        let mut fm = format!(
            "---\nname: {}\nrole: {}\navatar: {}\n",
            clean_name,
            Self::yaml_quote(if role.trim().is_empty() {
                "Autonomous Specialist Agent"
            } else {
                role
            }),
            if avatar_key.trim().is_empty() {
                "fe_senior_dev"
            } else {
                avatar_key.trim()
            }
        );
        if let Some(m) = model.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            fm.push_str(&format!("model: {}\n", Self::yaml_quote(m)));
        }
        if !default_skills.is_empty() {
            fm.push_str(&format!("skills: [{}]\n", default_skills.join(", ")));
        }
        fm.push_str("---\n\n");
        fm.push_str(&clean_body);
        fm.push('\n');

        fs::write(&target_file, fm)?;

        if let Some(orig) = original_path {
            if orig.is_file() && orig != target_file.as_path() {
                let _ = fs::remove_file(orig);
            }
        }

        Ok(target_file)
    }

    pub fn create_or_update_agent(
        root: &Path,
        name: &str,
        role: &str,
        system_prompt: &str,
    ) -> Result<PathBuf> {
        Self::save_agent(
            root,
            SkillScope::Project,
            name,
            role,
            "fe_senior_dev",
            None,
            &[],
            system_prompt,
            None,
            true,
        )
    }

    /// Moves a skill directory into ~/.victini/trash/ so the user can Undo.
    pub fn trash_skill(path: &Path) -> Result<TrashedItem> {
        let trash_root = Self::victini_dir().join("trash");
        fs::create_dir_all(&trash_root)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let stamp = Local::now().format("%Y%m%d_%H%M%S");
        let dest = trash_root.join(format!("skill_{}_{}", stamp, name));

        if fs::rename(path, &dest).is_err() {
            Self::copy_dir_recursive(path, &dest)?;
            fs::remove_dir_all(path)?;
        }

        Ok(TrashedItem {
            kind_label: "skill".to_string(),
            name,
            original_path: path.to_path_buf(),
            trash_path: dest,
            is_dir: true,
        })
    }

    /// Moves an agent markdown file into ~/.victini/trash/ so the user can Undo.
    pub fn trash_agent(path: &Path) -> Result<TrashedItem> {
        let trash_root = Self::victini_dir().join("trash");
        fs::create_dir_all(&trash_root)?;
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let stamp = Local::now().format("%Y%m%d_%H%M%S");
        let dest = trash_root.join(format!("agent_{}_{}.md", stamp, name));

        if fs::rename(path, &dest).is_err() {
            fs::copy(path, &dest)?;
            fs::remove_file(path)?;
        }

        Ok(TrashedItem {
            kind_label: "agent".to_string(),
            name,
            original_path: path.to_path_buf(),
            trash_path: dest,
            is_dir: false,
        })
    }

    pub fn restore_trashed(item: &TrashedItem) -> Result<()> {
        if let Some(parent) = item.original_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if fs::rename(&item.trash_path, &item.original_path).is_err() {
            if item.is_dir {
                Self::copy_dir_recursive(&item.trash_path, &item.original_path)?;
                fs::remove_dir_all(&item.trash_path)?;
            } else {
                fs::copy(&item.trash_path, &item.original_path)?;
                fs::remove_file(&item.trash_path)?;
            }
        }
        Ok(())
    }

    pub fn delete_skill(path: &Path) -> Result<()> {
        let _ = Self::trash_skill(path)?;
        Ok(())
    }

    pub fn delete_agent(path: &Path) -> Result<()> {
        let _ = Self::trash_agent(path)?;
        Ok(())
    }

    pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in WalkDir::new(src).into_iter().flatten() {
            let rel = entry.path().strip_prefix(src)?;
            if rel.as_os_str().is_empty() {
                continue;
            }
            let target = dst.join(rel);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&target)?;
            } else if entry.file_type().is_file() {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), &target)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frontmatter_handles_multiline_description_and_strips_header() {
        let raw = "---\nname: my-skill\ndescription: >\n  Line one of desc\n  and line two.\n---\n\n# Heading\nBody text here.";
        let (fields, body) = SkillDiscovery::parse_frontmatter(raw);
        assert_eq!(fields.get("name").map(|s| s.as_str()), Some("my-skill"));
        assert_eq!(
            fields.get("description").map(|s| s.as_str()),
            Some("Line one of desc and line two.")
        );
        assert_eq!(body, "# Heading\nBody text here.");
        assert!(!body.contains("---"));
    }

    #[test]
    fn save_skill_updates_description_and_rename_cleans_old_dir() {
        let root = std::env::temp_dir().join(format!("victini_skill_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let p1 = SkillDiscovery::save_skill(
            &root,
            SkillScope::Project,
            "Alpha Skill",
            "First desc",
            "# Alpha\nHello",
            None,
            false,
        )
        .unwrap();
        assert!(p1.join("SKILL.md").exists());
        fs::write(p1.join("extra.txt"), "keep me").unwrap();

        // Update description even if user pasted frontmatter in body
        let p2 = SkillDiscovery::save_skill(
            &root,
            SkillScope::Project,
            "beta-skill",
            "Updated desc",
            "---\nname: old\ndescription: stale\n---\n\n# Beta\nUpdated body",
            Some(&p1),
            false,
        )
        .unwrap();
        assert!(!p1.exists(), "Old directory must be removed on rename");
        assert!(p2.join("extra.txt").exists(), "Extra files must be preserved");
        let saved = fs::read_to_string(p2.join("SKILL.md")).unwrap();
        let (fields, body) = SkillDiscovery::parse_frontmatter(&saved);
        assert_eq!(fields.get("description").map(|s| s.as_str()), Some("Updated desc"));
        assert_eq!(body, "# Beta\nUpdated body");

        // Trash & restore
        let trashed = SkillDiscovery::trash_skill(&p2).unwrap();
        assert!(!p2.exists());
        SkillDiscovery::restore_trashed(&trashed).unwrap();
        assert!(p2.join("SKILL.md").exists());

        let _ = fs::remove_dir_all(&root);
    }
}
