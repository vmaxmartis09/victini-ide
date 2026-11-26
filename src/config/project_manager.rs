use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

fn default_expanded() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub path: PathBuf,
    pub last_branch: String,
    pub pinned: bool,
    #[serde(default = "default_expanded")]
    pub expanded: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectManager {
    pub active_project: PathBuf,
    pub projects: Vec<ProjectEntry>,
}

impl ProjectManager {
    pub fn load_or_init(initial_root: PathBuf) -> Self {
        let store_path = Self::registry_path();
        if let Ok(raw) = fs::read_to_string(&store_path) {
            if let Ok(mut mgr) = serde_json::from_str::<ProjectManager>(&raw) {
                mgr.ensure_project_in_list(&initial_root);
                return mgr;
            }
        }

        let mut mgr = Self {
            active_project: initial_root.clone(),
            projects: Vec::new(),
        };
        mgr.ensure_project_in_list(&initial_root);
        mgr.discover_sibling_workspaces(&initial_root);
        let _ = mgr.save();
        mgr
    }

    fn registry_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(home).join(".victini");
        let _ = fs::create_dir_all(&dir);
        dir.join("projects.json")
    }

    pub fn ensure_project_in_list(&mut self, path: &Path) {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if !self.projects.iter().any(|p| p.path == canonical) {
            let name = canonical
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            self.projects.insert(
                0,
                ProjectEntry {
                    name: if name.is_empty() { "workspace".to_string() } else { name },
                    path: canonical,
                    last_branch: "main".to_string(),
                    pinned: true,
                    expanded: true,
                },
            );
        }
    }

    pub fn discover_sibling_workspaces(&mut self, root: &Path) {
        if let Some(parent) = root.parent() {
            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries.flatten().take(16) {
                    let p = entry.path();
                    if p.is_dir() && (p.join(".git").exists() || p.join("Cargo.toml").exists() || p.join("package.json").exists()) {
                        let canonical = p.canonicalize().unwrap_or(p);
                        if !self.projects.iter().any(|existing| existing.path == canonical) {
                            let name = canonical
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            self.projects.push(ProjectEntry {
                                name,
                                path: canonical,
                                last_branch: "main".to_string(),
                                pinned: false,
                                expanded: true,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn add_or_switch_project(&mut self, path_str: &str) -> Result<PathBuf> {
        let p = PathBuf::from(path_str.trim());
        if !p.exists() {
            fs::create_dir_all(&p)?;
        }
        let canonical = p.canonicalize().unwrap_or(p);
        self.ensure_project_in_list(&canonical);
        self.active_project = canonical.clone();
        let _ = self.save();
        Ok(canonical)
    }

    pub fn toggle_pin(&mut self, path: &Path) {
        if let Some(proj) = self.projects.iter_mut().find(|p| p.path == path) {
            proj.pinned = !proj.pinned;
            let _ = self.save();
        }
    }

    pub fn toggle_expanded(&mut self, path: &Path) {
        if let Some(proj) = self.projects.iter_mut().find(|p| p.path == path) {
            proj.expanded = !proj.expanded;
            let _ = self.save();
        }
    }

    pub fn update_branch(&mut self, path: &Path, branch: &str) {
        if let Some(proj) = self.projects.iter_mut().find(|p| p.path == path) {
            proj.last_branch = branch.to_string();
            let _ = self.save();
        }
    }

    pub fn active_project_name(&self) -> String {
        self.active_project
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(Self::registry_path(), json)?;
        Ok(())
    }
}
