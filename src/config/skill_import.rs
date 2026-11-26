use crate::config::{SkillDiscovery, SkillScope, SkillSourceMeta};
use anyhow::{anyhow, bail, Result};
use chrono::Local;
use git2::{Cred, FetchOptions, RemoteCallbacks};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use walkdir::WalkDir;

pub const MAX_IMPORT_FILE_BYTES: u64 = 2 * 1024 * 1024; // 2 MB safety cap

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictStrategy {
    Skip,
    Overwrite,
    Rename,
}

impl ConflictStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Skip => "Bỏ qua nếu trùng tên",
            Self::Overwrite => "Ghi đè skill cũ",
            Self::Rename => "Tự đổi tên (vd: skill-2)",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedGitHubSource {
    pub owner: String,
    pub repo: String,
    pub clone_url: String,
    pub git_ref: Option<String>,
    pub subpath: Option<String>,
}

impl ParsedGitHubSource {
    pub fn repo_slug(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

#[derive(Clone, Debug)]
pub struct RemoteSkillCandidate {
    pub name: String,
    pub description: String,
    pub preview_md: String,
    pub subpath: String,
    pub abs_source_dir: PathBuf,
    pub file_count: usize,
    pub has_scripts: bool,
    pub skipped_large_files: usize,
    pub already_exists_project: bool,
    pub already_exists_global: bool,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct ScanRepoResult {
    pub source: ParsedGitHubSource,
    pub resolved_ref: String,
    pub commit_sha: String,
    pub candidates: Vec<RemoteSkillCandidate>,
}

#[derive(Clone, Debug)]
pub struct InstallReport {
    pub installed: Vec<String>,
    pub skipped: Vec<String>,
    pub renamed: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub enum ImportEvent {
    Progress(String, f32),
    ScanCompleted(Result<ScanRepoResult, String>),
    InstallCompleted(Result<InstallReport, String>),
    UpdateCompleted(Result<String, String>),
}

/// Parses GitHub URLs and shorthands:
/// - `owner/repo`
/// - `https://github.com/owner/repo`
/// - `https://github.com/owner/repo.git`
/// - `https://github.com/owner/repo/tree/<branch>/<subpath...>`
/// - `https://github.com/owner/repo/blob/<branch>/<subpath...>/SKILL.md`
/// - `git@github.com:owner/repo.git`
pub fn parse_github_url(input: &str, branch_override: Option<&str>) -> Result<ParsedGitHubSource> {
    let raw = input.trim().trim_end_matches('/');
    if raw.is_empty() {
        bail!("Vui lòng nhập URL GitHub hoặc dạng owner/repo.");
    }

    let cleaned = if let Some(rest) = raw.strip_prefix("git@github.com:") {
        rest.to_string()
    } else if let Some(rest) = raw.strip_prefix("https://github.com/") {
        rest.to_string()
    } else if let Some(rest) = raw.strip_prefix("http://github.com/") {
        rest.to_string()
    } else if let Some(rest) = raw.strip_prefix("github.com/") {
        rest.to_string()
    } else if !raw.contains("://") && raw.contains('/') {
        raw.to_string()
    } else {
        bail!("URL chưa được hỗ trợ. Hãy dùng dạng https://github.com/owner/repo hoặc owner/repo.");
    };

    let parts: Vec<&str> = cleaned.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        bail!("Thiếu tên owner hoặc repository (dạng owner/repo).");
    }

    let owner = parts[0].to_string();
    let repo = parts[1].trim_end_matches(".git").to_string();
    if owner.is_empty() || repo.is_empty() {
        bail!("Tên owner/repo không hợp lệ.");
    }

    let mut git_ref: Option<String> = None;
    let mut subpath: Option<String> = None;

    if parts.len() >= 4 && (parts[2] == "tree" || parts[2] == "blob") {
        git_ref = Some(parts[3].to_string());
        if parts.len() > 4 {
            let mut sub_parts: Vec<&str> = parts[4..].to_vec();
            if parts[2] == "blob"
                && sub_parts
                    .last()
                    .map(|f| f.eq_ignore_ascii_case("SKILL.md"))
                    .unwrap_or(false)
            {
                sub_parts.pop();
            }
            if !sub_parts.is_empty() {
                subpath = Some(sub_parts.join("/"));
            }
        }
    }

    if let Some(ovr) = branch_override.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        git_ref = Some(ovr.to_string());
    }

    Ok(ParsedGitHubSource {
        clone_url: format!("https://github.com/{}/{}.git", owner, repo),
        owner,
        repo,
        git_ref,
        subpath,
    })
}

fn cache_dir_for(src: &ParsedGitHubSource) -> PathBuf {
    let dir = SkillDiscovery::victini_dir().join("cache").join("repos");
    let _ = fs::create_dir_all(&dir);
    let safe = format!(
        "{}__{}",
        SkillDiscovery::sanitize_skill_slug(&src.owner),
        SkillDiscovery::sanitize_skill_slug(&src.repo)
    );
    dir.join(safe)
}

fn build_fetch_options<'a>(token: Option<String>) -> FetchOptions<'a> {
    let mut callbacks = RemoteCallbacks::new();
    let effective_token = token
        .filter(|t| !t.trim().is_empty())
        .or_else(|| std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.trim().is_empty()));

    if let Some(tok) = effective_token {
        callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
            Cred::userpass_plaintext("x-access-token", &tok)
        });
    }

    let mut opts = FetchOptions::new();
    opts.remote_callbacks(callbacks);
    opts.depth(1);
    opts
}

fn clone_or_refresh_repo(
    src: &ParsedGitHubSource,
    token: Option<String>,
    tx: Option<&std_mpsc::Sender<ImportEvent>>,
) -> Result<(PathBuf, String, String)> {
    let repo_dir = cache_dir_for(src);
    if repo_dir.exists() {
        let _ = fs::remove_dir_all(&repo_dir);
    }

    if let Some(tx) = tx {
        let _ = tx.send(ImportEvent::Progress(
            format!("Đang tải repository {}...", src.repo_slug()),
            0.25,
        ));
    }

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(build_fetch_options(token.clone()));
    if let Some(ref branch) = src.git_ref {
        builder.branch(branch);
    }

    let repo = match builder.clone(&src.clone_url, &repo_dir) {
        Ok(r) => r,
        Err(first_err) => {
            // Fallback without shallow depth if server rejects depth=1 or ref is a tag/commit
            let _ = fs::remove_dir_all(&repo_dir);
            let mut callbacks = RemoteCallbacks::new();
            let effective_token = token
                .filter(|t| !t.trim().is_empty())
                .or_else(|| std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.trim().is_empty()));
            if let Some(tok) = effective_token {
                callbacks.credentials(move |_url, _user, _allowed| {
                    Cred::userpass_plaintext("x-access-token", &tok)
                });
            }
            let mut opts = FetchOptions::new();
            opts.remote_callbacks(callbacks);
            let mut fallback = git2::build::RepoBuilder::new();
            fallback.fetch_options(opts);
            if let Some(ref branch) = src.git_ref {
                fallback.branch(branch);
            }
            fallback.clone(&src.clone_url, &repo_dir).map_err(|e2| {
                anyhow!(
                    "Không thể clone {} ({}). Hãy kiểm tra URL, nhánh hoặc token repo private. Chi tiết: {}",
                    src.repo_slug(),
                    first_err.message(),
                    e2.message()
                )
            })?
        }
    };

    let head = repo.head()?;
    let resolved_ref = head
        .shorthand()
        .map(|s| s.to_string())
        .or_else(|| src.git_ref.clone())
        .unwrap_or_else(|| "main".to_string());
    let commit_sha = head
        .peel_to_commit()
        .map(|c| c.id().to_string())
        .unwrap_or_default();

    Ok((repo_dir, resolved_ref, commit_sha))
}

fn scan_repo_directory(
    repo_dir: &Path,
    src: &ParsedGitHubSource,
    project_root: &Path,
) -> Result<Vec<RemoteSkillCandidate>> {
    let scan_root = if let Some(ref sub) = src.subpath {
        let joined = repo_dir.join(sub);
        if !joined.exists() {
            bail!("Không tìm thấy thư mục con '{}' trong repository.", sub);
        }
        joined
    } else {
        repo_dir.to_path_buf()
    };

    let mut candidates = Vec::new();
    let proj_skills_dir = project_root.join("skills");
    let global_skills_dir = SkillDiscovery::global_skills_dir();

    for entry in WalkDir::new(&scan_root)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "node_modules" && name != "target"
        })
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if !entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case("SKILL.md")
        {
            continue;
        }

        let Some(skill_dir) = entry.path().parent() else {
            continue;
        };

        let raw_name = if skill_dir == repo_dir {
            src.repo.clone()
        } else {
            skill_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        };
        let clean_name = SkillDiscovery::sanitize_skill_slug(&raw_name);
        if clean_name.is_empty() {
            continue;
        }
        if candidates
            .iter()
            .any(|c: &RemoteSkillCandidate| c.name == clean_name)
        {
            continue;
        }

        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let (fields, body) = SkillDiscovery::parse_frontmatter(&content);
        let description = fields
            .get("description")
            .cloned()
            .unwrap_or_else(|| "Skill từ GitHub".to_string());
        let preview_md: String = body.lines().take(60).collect::<Vec<_>>().join("\n");

        let (files, has_scripts) = SkillDiscovery::inspect_skill_files(skill_dir);
        let mut skipped_large_files = 0usize;
        for f in &files {
            if let Ok(meta) = fs::metadata(skill_dir.join(f)) {
                if meta.len() > MAX_IMPORT_FILE_BYTES {
                    skipped_large_files += 1;
                }
            }
        }

        let subpath = skill_dir
            .strip_prefix(repo_dir)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        candidates.push(RemoteSkillCandidate {
            already_exists_project: proj_skills_dir.join(&clean_name).exists(),
            already_exists_global: global_skills_dir.join(&clean_name).exists(),
            name: clean_name,
            description,
            preview_md,
            subpath,
            abs_source_dir: skill_dir.to_path_buf(),
            file_count: files.len(),
            has_scripts,
            skipped_large_files,
            selected: true,
        });
    }

    candidates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(candidates)
}

pub fn spawn_scan_github_repo(
    url: String,
    branch_override: String,
    token: Option<String>,
    project_root: PathBuf,
) -> std_mpsc::Receiver<ImportEvent> {
    let (tx, rx) = std_mpsc::channel();
    tokio::spawn(async move {
        let tx_inner = tx.clone();
        let res = tokio::task::spawn_blocking(move || -> Result<ScanRepoResult> {
            let br = if branch_override.trim().is_empty() {
                None
            } else {
                Some(branch_override.as_str())
            };
            let parsed = parse_github_url(&url, br)?;
            let (repo_dir, resolved_ref, commit_sha) =
                clone_or_refresh_repo(&parsed, token, Some(&tx_inner))?;
            let _ = tx_inner.send(ImportEvent::Progress(
                "Đang quét các thư mục SKILL.md...".to_string(),
                0.7,
            ));
            let candidates = scan_repo_directory(&repo_dir, &parsed, &project_root)?;
            if candidates.is_empty() {
                bail!("Không tìm thấy file SKILL.md nào trong repository này.");
            }
            let _ = tx_inner.send(ImportEvent::Progress("Quét hoàn tất.".to_string(), 1.0));
            Ok(ScanRepoResult {
                source: parsed,
                resolved_ref,
                commit_sha,
                candidates,
            })
        })
        .await;

        let final_res = match res {
            Ok(Ok(data)) => Ok(data),
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(format!("Task lỗi: {}", e)),
        };
        let _ = tx.send(ImportEvent::ScanCompleted(final_res));
    });
    rx
}

fn resolve_target_name(base_dir: &Path, desired: &str, strategy: ConflictStrategy) -> Option<String> {
    if !base_dir.join(desired).exists() {
        return Some(desired.to_string());
    }
    match strategy {
        ConflictStrategy::Skip => None,
        ConflictStrategy::Overwrite => Some(desired.to_string()),
        ConflictStrategy::Rename => {
            for idx in 2..=99 {
                let candidate = format!("{}-{}", desired, idx);
                if !base_dir.join(&candidate).exists() {
                    return Some(candidate);
                }
            }
            None
        }
    }
}

fn copy_skill_safe(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_entry(|e| e.file_name().to_string_lossy() != ".git")
        .flatten()
    {
        let rel = entry.path().strip_prefix(src)?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                if meta.len() > MAX_IMPORT_FILE_BYTES {
                    continue;
                }
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

pub fn spawn_install_candidates(
    scan: ScanRepoResult,
    scope: SkillScope,
    strategy: ConflictStrategy,
    project_root: PathBuf,
) -> std_mpsc::Receiver<ImportEvent> {
    let (tx, rx) = std_mpsc::channel();
    tokio::spawn(async move {
        let tx_bg = tx.clone();
        let res = tokio::task::spawn_blocking(move || -> Result<InstallReport> {
            let base_dir = match scope {
                SkillScope::Project => project_root.join("skills"),
                SkillScope::Global => SkillDiscovery::global_skills_dir(),
            };
            fs::create_dir_all(&base_dir)?;

            let selected: Vec<&RemoteSkillCandidate> =
                scan.candidates.iter().filter(|c| c.selected).collect();
            let total = selected.len().max(1);
            let mut report = InstallReport {
                installed: Vec::new(),
                skipped: Vec::new(),
                renamed: Vec::new(),
            };

            for (idx, cand) in selected.iter().enumerate() {
                let _ = tx_bg.send(ImportEvent::Progress(
                    format!("Đang cài {} ({}/{})...", cand.name, idx + 1, total),
                    (idx as f32 + 0.5) / total as f32,
                ));

                let Some(final_name) = resolve_target_name(&base_dir, &cand.name, strategy) else {
                    report.skipped.push(cand.name.clone());
                    continue;
                };

                let dst = base_dir.join(&final_name);
                copy_skill_safe(&cand.abs_source_dir, &dst)?;

                let meta = SkillSourceMeta {
                    repo: scan.source.repo_slug(),
                    url: scan.source.clone_url.clone(),
                    git_ref: scan.resolved_ref.clone(),
                    commit_sha: scan.commit_sha.clone(),
                    subpath: cand.subpath.clone(),
                    installed_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
                };
                if let Ok(json) = serde_json::to_string_pretty(&meta) {
                    let _ = fs::write(dst.join(".victini-source.json"), json);
                }

                if final_name != cand.name {
                    report.renamed.push((cand.name.clone(), final_name.clone()));
                }
                report.installed.push(final_name);
            }

            let _ = tx_bg.send(ImportEvent::Progress("Cài đặt hoàn tất.".to_string(), 1.0));
            Ok(report)
        })
        .await;

        let final_res = match res {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(format!("Task lỗi: {}", e)),
        };
        let _ = tx.send(ImportEvent::InstallCompleted(final_res));
    });
    rx
}

pub fn spawn_update_skill_from_source(
    skill_path: PathBuf,
    source: SkillSourceMeta,
    token: Option<String>,
) -> std_mpsc::Receiver<ImportEvent> {
    let (tx, rx) = std_mpsc::channel();
    tokio::spawn(async move {
        let tx_bg = tx.clone();
        let res = tokio::task::spawn_blocking(move || -> Result<String> {
            let parsed = parse_github_url(&source.url, Some(&source.git_ref))?;
            let (repo_dir, resolved_ref, commit_sha) =
                clone_or_refresh_repo(&parsed, token, Some(&tx_bg))?;

            if !commit_sha.is_empty() && commit_sha == source.commit_sha {
                let short = &commit_sha[..7.min(commit_sha.len())];
                return Ok(format!("Skill đã ở bản mới nhất (@{}).", short));
            }

            let src_dir = if source.subpath.trim().is_empty() {
                repo_dir.clone()
            } else {
                repo_dir.join(&source.subpath)
            };
            if !src_dir.join("SKILL.md").exists() {
                bail!(
                    "Không còn tìm thấy SKILL.md tại '{}' trong repo nguồn.",
                    source.subpath
                );
            }

            copy_skill_safe(&src_dir, &skill_path)?;
            let updated_meta = SkillSourceMeta {
                repo: source.repo,
                url: source.url,
                git_ref: resolved_ref,
                commit_sha: commit_sha.clone(),
                subpath: source.subpath,
                installed_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&updated_meta) {
                let _ = fs::write(skill_path.join(".victini-source.json"), json);
            }
            let short = &commit_sha[..7.min(commit_sha.len())];
            Ok(format!("Đã cập nhật skill lên commit @{}.", short))
        })
        .await;

        let final_res = match res {
            Ok(Ok(msg)) => Ok(msg),
            Ok(Err(e)) => Err(e.to_string()),
            Err(e) => Err(format!("Lỗi cập nhật: {}", e)),
        };
        let _ = tx.send(ImportEvent::UpdateCompleted(final_res));
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_url_supports_shorthand_tree_and_blob() {
        let p1 = parse_github_url("anthropics/skills", None).unwrap();
        assert_eq!(p1.owner, "anthropics");
        assert_eq!(p1.repo, "skills");
        assert_eq!(p1.clone_url, "https://github.com/anthropics/skills.git");
        assert_eq!(p1.git_ref, None);
        assert_eq!(p1.subpath, None);

        let p2 = parse_github_url(
            "https://github.com/anthropics/skills/tree/main/skills/pdf",
            None,
        )
        .unwrap();
        assert_eq!(p2.git_ref.as_deref(), Some("main"));
        assert_eq!(p2.subpath.as_deref(), Some("skills/pdf"));

        let p3 = parse_github_url(
            "https://github.com/acme/repo/blob/dev/my-skill/SKILL.md",
            None,
        )
        .unwrap();
        assert_eq!(p3.git_ref.as_deref(), Some("dev"));
        assert_eq!(p3.subpath.as_deref(), Some("my-skill"));

        let p4 = parse_github_url("git@github.com:acme/cool.git", Some("v2")).unwrap();
        assert_eq!(p4.owner, "acme");
        assert_eq!(p4.repo, "cool");
        assert_eq!(p4.git_ref.as_deref(), Some("v2"));
    }

    #[test]
    fn resolve_target_name_handles_conflict_strategies() {
        let base = std::env::temp_dir().join(format!("victini_conflict_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("pdf")).unwrap();

        assert_eq!(
            resolve_target_name(&base, "pdf", ConflictStrategy::Skip),
            None
        );
        assert_eq!(
            resolve_target_name(&base, "pdf", ConflictStrategy::Overwrite),
            Some("pdf".to_string())
        );
        assert_eq!(
            resolve_target_name(&base, "pdf", ConflictStrategy::Rename),
            Some("pdf-2".to_string())
        );

        let _ = fs::remove_dir_all(&base);
    }
}
