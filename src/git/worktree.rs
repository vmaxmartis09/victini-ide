use crate::git::diff_card::{compute_diff, DiffHunk};
use anyhow::{Context, Result};
use git2::{build::CheckoutBuilder, Repository, StatusOptions};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct GitFileChange {
    pub path: String,
    pub status: String, // "M", "A", "D", "?"
    pub is_staged: bool,
    pub added_lines: usize,
    pub removed_lines: usize,
}

#[derive(Clone, Debug)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: PathBuf,
    pub is_head: bool,
}

pub struct GitManager {
    pub repo_path: PathBuf,
}

impl GitManager {
    pub fn new(path: PathBuf) -> Self {
        Self { repo_path: path }
    }

    pub fn set_repo_path(&mut self, path: PathBuf) {
        self.repo_path = path;
    }

    pub fn get_branch_name(&self) -> String {
        if let Ok(repo) = Repository::discover(&self.repo_path) {
            if let Ok(head) = repo.head() {
                if let Some(sh) = head.shorthand() {
                    return sh.to_string();
                }
            }
        }
        "main".to_string()
    }

    pub fn get_head_commit_short(&self) -> String {
        if let Ok(repo) = Repository::discover(&self.repo_path) {
            if let Ok(head) = repo.head() {
                if let Some(target) = head.target() {
                    let s = target.to_string();
                    if s.len() >= 7 {
                        return s[..7].to_string();
                    }
                }
            }
        }
        "HEAD".to_string()
    }

    pub fn list_branches(&self) -> Vec<String> {
        let mut branches = Vec::new();
        if let Ok(repo) = Repository::discover(&self.repo_path) {
            if let Ok(iter) = repo.branches(Some(git2::BranchType::Local)) {
                for b in iter.flatten() {
                    if let Ok(Some(name)) = b.0.name() {
                        branches.push(name.to_string());
                    }
                }
            }
        }
        if branches.is_empty() {
            branches.push(self.get_branch_name());
        }
        branches
    }

    pub fn create_and_checkout_branch(&self, branch_name: &str) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let head = repo.head()?;
        let commit = repo.find_commit(head.target().context("HEAD has no target")?)?;
        let _ = repo.branch(branch_name, &commit, false);
        self.switch_branch(branch_name)
    }

    pub fn list_worktrees(&self) -> Vec<WorktreeInfo> {
        let mut list = Vec::new();
        let active_branch = self.get_branch_name();
        if let Ok(repo) = Repository::discover(&self.repo_path) {
            let root_dir = repo.workdir().unwrap_or(&self.repo_path).to_path_buf();
            list.push(WorktreeInfo {
                name: active_branch.clone(),
                path: root_dir,
                is_head: true,
            });

            if let Ok(wts) = repo.worktrees() {
                for name in wts.iter().flatten() {
                    if let Ok(wt) = repo.find_worktree(name) {
                        list.push(WorktreeInfo {
                            name: name.to_string(),
                            path: wt.path().to_path_buf(),
                            is_head: name == active_branch,
                        });
                    }
                }
            }
        } else {
            list.push(WorktreeInfo {
                name: active_branch,
                path: self.repo_path.clone(),
                is_head: true,
            });
        }
        list
    }

    pub fn switch_branch(&self, branch_name: &str) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let (object, reference) = repo.revparse_ext(branch_name)?;

        let mut checkout = CheckoutBuilder::new();
        checkout.safe();
        repo.checkout_tree(&object, Some(&mut checkout))?;

        if let Some(gref) = reference {
            repo.set_head(gref.name().context("Invalid reference name")?)?;
        } else {
            repo.set_head_detached(object.id())?;
        }
        Ok(())
    }

    pub fn setup_new_worktree(&self, branch_name: &str, path: &Path) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let head = repo.head()?;
        let commit = repo.find_commit(head.target().context("Head has no target")?)?;
        let branch = repo
            .find_branch(branch_name, git2::BranchType::Local)
            .or_else(|_| repo.branch(branch_name, &commit, false))?;
        repo.worktree(
            branch_name,
            path,
            Some(git2::WorktreeAddOptions::new().reference(Some(branch.get()))),
        )?;
        Ok(())
    }

    pub fn is_noisy_build_path(path: &str) -> bool {
        let norm = path.replace('\\', "/");
        let lower = norm.to_lowercase();
        let noisy_prefixes = [
            "target/",
            "node_modules/",
            ".git/",
            "dist/",
            "build/",
            ".next/",
            ".cargo/",
            ".gradle/",
            "__pycache__/",
            ".venv/",
            "venv/",
            "vendor/",
        ];
        if noisy_prefixes
            .iter()
            .any(|p| lower.starts_with(p) || lower.contains(&format!("/{}", p)))
        {
            return true;
        }
        let file_name = norm.rsplit('/').next().unwrap_or(&norm);
        if matches!(
            file_name,
            "CACHEDIR.TAG"
                | ".cargo-lock"
                | "rustc_info.json"
                | "invoked.timestamp"
                | ".DS_Store"
        ) || file_name.starts_with("dep-lib-")
            || file_name.starts_with("run-build-script-")
            || file_name.ends_with(".rlib")
            || file_name.ends_with(".rmeta")
            || file_name.ends_with(".d")
            || file_name.ends_with(".o")
        {
            return true;
        }
        false
    }

    pub fn get_status(&self) -> Vec<GitFileChange> {
        let mut changes = Vec::new();
        if let Ok(repo) = Repository::discover(&self.repo_path) {
            let mut opts = StatusOptions::new();
            opts.include_untracked(true).recurse_untracked_dirs(true);
            if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
                for entry in statuses.iter() {
                    let path = entry.path().unwrap_or("").to_string();
                    if path.is_empty() || Self::is_noisy_build_path(&path) {
                        continue;
                    }
                    let s = entry.status();
                    let is_staged = s.is_index_new()
                        || s.is_index_modified()
                        || s.is_index_deleted()
                        || s.is_index_renamed();
                    let status_str = if s.is_wt_new() || s.is_index_new() {
                        "A"
                    } else if s.is_wt_modified() || s.is_index_modified() {
                        "M"
                    } else if s.is_wt_deleted() || s.is_index_deleted() {
                        "D"
                    } else {
                        "?"
                    };

                    let (added_lines, removed_lines) = if changes.len() < 60 {
                        self.compute_file_diff(&path)
                            .map(|h| (h.added_count, h.removed_count))
                            .unwrap_or((0, 0))
                    } else {
                        (0, 0)
                    };

                    changes.push(GitFileChange {
                        path,
                        status: status_str.to_string(),
                        is_staged,
                        added_lines,
                        removed_lines,
                    });
                    if changes.len() >= 250 {
                        break;
                    }
                }
            }
        }
        changes
    }

    pub fn compute_file_diff(&self, rel_path: &str) -> Result<DiffHunk> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let workdir = repo.workdir().unwrap_or(&self.repo_path);
        let disk_path = workdir.join(rel_path);
        let new_content = fs::read_to_string(&disk_path).unwrap_or_default();

        let mut old_content = String::new();
        if let Ok(head) = repo.head() {
            if let Ok(tree) = head.peel_to_tree() {
                if let Ok(entry) = tree.get_path(Path::new(rel_path)) {
                    if let Ok(obj) = entry.to_object(&repo) {
                        if let Some(blob) = obj.as_blob() {
                            old_content = String::from_utf8_lossy(blob.content()).to_string();
                        }
                    }
                }
            }
        }
        Ok(compute_diff(rel_path, &old_content, &new_content))
    }

    pub fn stage_file(&self, rel_path: &str) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let mut index = repo.index()?;
        let workdir = repo.workdir().unwrap_or(&self.repo_path);
        if workdir.join(rel_path).exists() {
            index.add_path(Path::new(rel_path))?;
        } else {
            index.remove_path(Path::new(rel_path))?;
        }
        index.write()?;
        Ok(())
    }

    pub fn unstage_file(&self, rel_path: &str) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        if let Ok(head) = repo.head() {
            if let Ok(target) = head.target().context("No HEAD") {
                if let Ok(obj) = repo.find_object(target, None) {
                    repo.reset_default(Some(&obj), [Path::new(rel_path)])?;
                    return Ok(());
                }
            }
        }
        let mut index = repo.index()?;
        let _ = index.remove_path(Path::new(rel_path));
        index.write()?;
        Ok(())
    }

    pub fn discard_file(&self, rel_path: &str) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let workdir = repo.workdir().unwrap_or(&self.repo_path);
        let in_head = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_tree().ok())
            .and_then(|t| t.get_path(Path::new(rel_path)).ok())
            .is_some();

        if in_head {
            let mut checkout = CheckoutBuilder::new();
            checkout.force().path(rel_path);
            repo.checkout_head(Some(&mut checkout))?;
        } else {
            let target = workdir.join(rel_path);
            if target.is_file() {
                fs::remove_file(&target)?;
            } else if target.is_dir() {
                fs::remove_dir_all(&target)?;
            }
        }
        Ok(())
    }

    pub fn stage_all(&self) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    pub fn unstage_all(&self) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        if let Ok(head) = repo.head() {
            if let Some(target) = head.target() {
                if let Ok(obj) = repo.find_object(target, None) {
                    repo.reset(&obj, git2::ResetType::Mixed, None)?;
                    return Ok(());
                }
            }
        }
        let mut index = repo.index()?;
        index.clear()?;
        index.write()?;
        Ok(())
    }

    pub fn read_file_content(&self, rel_path: &str) -> Result<String> {
        let repo = Repository::discover(&self.repo_path).ok();
        let workdir = repo
            .as_ref()
            .and_then(|r| r.workdir())
            .unwrap_or(&self.repo_path);
        let full_path = workdir.join(rel_path);
        fs::read_to_string(&full_path)
            .with_context(|| format!("Không thể đọc file {:?}", full_path))
    }

    pub fn write_file_content(&self, rel_path: &str, content: &str) -> Result<()> {
        let repo = Repository::discover(&self.repo_path).ok();
        let workdir = repo
            .as_ref()
            .and_then(|r| r.workdir())
            .unwrap_or(&self.repo_path);
        let full_path = workdir.join(rel_path);
        if let Some(parent) = full_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&full_path, content)
            .with_context(|| format!("Không thể lưu file {:?}", full_path))
    }

    pub fn list_workspace_files(&self, max_files: usize) -> Vec<String> {
        let mut files = Vec::new();
        if let Ok(repo) = Repository::discover(&self.repo_path) {
            if let Ok(index) = repo.index() {
                for entry in index.iter() {
                    if let Ok(p) = std::str::from_utf8(&entry.path) {
                        if !p.starts_with("target/") && !p.starts_with(".git/") {
                            files.push(p.to_string());
                            if files.len() >= max_files {
                                break;
                            }
                        }
                    }
                }
            }
        }
        if files.is_empty() {
            Self::walk_dir_files(&self.repo_path, &self.repo_path, &mut files, max_files);
        }
        files
    }

    fn walk_dir_files(root: &Path, dir: &Path, out: &mut Vec<String>, limit: usize) {
        if out.len() >= limit {
            return;
        }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if out.len() >= limit {
                    break;
                }
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
                if p.is_dir() {
                    Self::walk_dir_files(root, &p, out, limit);
                } else if let Ok(rel) = p.strip_prefix(root) {
                    out.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
    }

    pub fn stash_save(&self, message: &str) -> Result<String> {
        let msg = if message.trim().is_empty() {
            "victini-wip-stash"
        } else {
            message.trim()
        };
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(["stash", "push", "-u", "-m", msg])
            .output()
            .context("Failed to execute git stash push")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            Ok(format!("✓ Stashed working changes: {}", stdout.trim()))
        } else {
            anyhow::bail!("Git stash error: {} {}", stdout.trim(), stderr.trim())
        }
    }

    pub fn stash_pop(&self) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(["stash", "pop"])
            .output()
            .context("Failed to execute git stash pop")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            Ok(format!("✓ Popped stash: {}", stdout.trim()))
        } else {
            anyhow::bail!("Git stash pop error: {} {}", stdout.trim(), stderr.trim())
        }
    }

    pub fn remove_worktree(&self, wt_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(["worktree", "remove", "--force", &wt_path.to_string_lossy()])
            .output()
            .context("Failed to remove worktree")?;
        let _ = Command::new("git")
            .current_dir(&self.repo_path)
            .args(["worktree", "prune"])
            .output();
        if output.status.success() {
            Ok(format!("✓ Đã xóa Worktree {:?}", wt_path))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Không thể xóa Worktree: {}", stderr.trim())
        }
    }

    pub fn open_in_external_editor(&self, rel_path: Option<&str>) -> Result<String> {
        let target = match rel_path {
            Some(p) if !p.is_empty() => self.repo_path.join(p),
            _ => self.repo_path.clone(),
        };
        for cmd_name in ["cursor", "code", "zed"] {
            if Command::new(cmd_name).arg(&target).spawn().is_ok() {
                return Ok(format!("✓ Đã mở {:?} bằng {}", target, cmd_name));
            }
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(&target).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("explorer").arg(&target).spawn();
        }
        Ok(format!("✓ Đã mở {:?}", target))
    }

    pub fn generate_ai_commit_message(&self, changes: &[GitFileChange]) -> String {
        if changes.is_empty() {
            return "chore(workspace): clean sync".to_string();
        }
        let staged: Vec<&GitFileChange> = changes.iter().filter(|c| c.is_staged).collect();
        let active_set: Vec<&GitFileChange> = if !staged.is_empty() {
            staged
        } else {
            changes.iter().collect()
        };

        let mut total_add = 0;
        let mut total_rem = 0;
        let mut scopes = Vec::new();
        let mut file_names = Vec::new();
        let mut has_new = false;
        let mut has_del = false;

        for c in &active_set {
            total_add += c.added_lines;
            total_rem += c.removed_lines;
            if c.status == "A" || c.status == "?" {
                has_new = true;
            } else if c.status == "D" {
                has_del = true;
            }
            let parts: Vec<&str> = c.path.split('/').collect();
            if parts.len() >= 2 {
                let scope = parts[parts.len() - 2];
                if !scopes.contains(&scope) && scope != "src" {
                    scopes.push(scope);
                }
            }
            if let Some(fname) = parts.last() {
                if file_names.len() < 3 {
                    file_names.push(*fname);
                }
            }
        }

        let prefix = if has_new && !has_del {
            "feat"
        } else if has_del && active_set.len() == 1 {
            "chore"
        } else {
            "fix"
        };

        let scope_str = if scopes.is_empty() {
            self.get_branch_name()
        } else {
            scopes.into_iter().take(2).collect::<Vec<_>>().join(",")
        };

        let more = if active_set.len() > 3 {
            format!(" (+{} files)", active_set.len() - 3)
        } else {
            String::new()
        };

        format!(
            "{}({}): update {}{} (+{} -{})",
            prefix,
            scope_str,
            file_names.join(", "),
            more,
            total_add,
            total_rem
        )
    }

    pub fn commit(&self, message: &str) -> Result<String> {
        let repo = Repository::discover(&self.repo_path).context("Not a git repository")?;
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let sig = repo.signature().unwrap_or_else(|_| {
            git2::Signature::now("Victini Vibe Station", "victini@vmax.ai").unwrap()
        });
        if let Ok(head) = repo.head() {
            if let Some(target) = head.target() {
                if let Ok(parent) = repo.find_commit(target) {
                    let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;
                    return Ok(oid.to_string());
                }
            }
        }
        let oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?;
        Ok(oid.to_string())
    }

    pub fn push_current_branch(&self) -> Result<String> {
        let branch = self.get_branch_name();
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(["push", "-u", "origin", &branch])
            .output()
            .context("Failed to execute git push")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            Ok(format!("✓ Pushed branch '{}' to origin ({})", branch, stderr.trim()))
        } else {
            anyhow::bail!("Git push warning/error: {} {}", stdout.trim(), stderr.trim())
        }
    }

    pub fn pull_rebase(&self) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(["pull", "--rebase", "--autostash"])
            .output()
            .context("Failed to execute git pull")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            Ok(format!("✓ Pulled latest changes: {}", stdout.trim()))
        } else {
            anyhow::bail!("Git pull error: {} {}", stdout.trim(), stderr.trim())
        }
    }

    pub fn get_gitlab_mr_url(&self) -> String {
        let branch = self.get_branch_name();
        if let Ok(repo) = Repository::discover(&self.repo_path) {
            if let Ok(remote) = repo.find_remote("origin") {
                if let Some(url) = remote.url() {
                    let clean = url
                        .trim_end_matches(".git")
                        .replace("git@gitlab.com:", "https://gitlab.com/")
                        .replace("git@github.com:", "https://github.com/");
                    if clean.starts_with("http") {
                        return format!(
                            "{}/-/merge_requests/new?merge_request[source_branch]={}",
                            clean, branch
                        );
                    }
                }
            }
        }
        format!(
            "https://gitlab.com/dashboard/merge_requests?scope=all&state=opened&source_branch={}",
            branch
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noisy_build_paths_and_ai_commit_message_work() {
        assert!(GitManager::is_noisy_build_path("target/debug/deps/app.d"));
        assert!(GitManager::is_noisy_build_path("web/node_modules/react/index.js"));
        assert!(GitManager::is_noisy_build_path(".DS_Store"));
        assert!(!GitManager::is_noisy_build_path("src/ui/kanban_view.rs"));

        let mgr = GitManager::new(PathBuf::from("."));
        let changes = vec![
            GitFileChange {
                path: "src/ui/header.rs".to_string(),
                status: "M".to_string(),
                is_staged: true,
                added_lines: 12,
                removed_lines: 4,
            },
            GitFileChange {
                path: "src/ai/kanban.rs".to_string(),
                status: "M".to_string(),
                is_staged: true,
                added_lines: 8,
                removed_lines: 2,
            },
        ];
        let msg = mgr.generate_ai_commit_message(&changes);
        assert!(msg.starts_with("fix(ui,ai): update header.rs, kanban.rs (+20 -6)"));
    }
}
