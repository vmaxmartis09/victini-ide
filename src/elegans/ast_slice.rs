use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

static RE_IMPORT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:import|export)\s+(?:type\s+)?(?:[\w*\s{},]+from\s+)?['"]([^'"]+)['"]|\b(?:use|mod)\s+([a-zA-Z0-9_:]+)"#)
        .unwrap()
});

#[derive(Debug, Clone)]
pub struct AstBlastSummary {
    pub files: usize,
    pub edges: usize,
    pub god_hubs: Vec<String>,
    pub impacted: Vec<String>,
}

pub fn analyze_ast_blast(dir: &Path, target: &str) -> AstBlastSummary {
    let mut file_count = 0usize;
    let mut edge_count = 0usize;
    let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::new();
    let mut fan_in: HashMap<String, usize> = HashMap::new();

    for entry in WalkDir::new(dir).max_depth(6).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let p_str = p.to_string_lossy().replace('\\', "/");
        if p_str.contains("node_modules")
            || p_str.contains("/.next")
            || p_str.contains("/dist/")
            || p_str.contains("/target/")
            || p_str.contains("/.git/")
        {
            continue;
        }
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(
            ext,
            "rs" | "tsx" | "ts" | "jsx" | "js" | "vue" | "svelte" | "astro"
        ) {
            continue;
        }
        file_count += 1;
        if let Ok(raw) = fs::read_to_string(p) {
            for cap in RE_IMPORT.captures_iter(&raw) {
                if let Some(m) = cap.get(1).or_else(|| cap.get(2)) {
                    let imp = m.as_str();
                    if imp.starts_with('.')
                        || imp.starts_with("@/")
                        || imp.starts_with("~/")
                        || imp.starts_with("crate::")
                        || !imp.contains("::")
                    {
                        edge_count += 1;
                        let key = Path::new(imp)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(imp)
                            .rsplit("::")
                            .find(|seg| !seg.is_empty() && *seg != "*" && *seg != "self" && *seg != "super")
                            .unwrap_or("")
                            .to_string();
                        if !key.is_empty() && key != "tests" {
                            *fan_in.entry(key.clone()).or_insert(0) += 1;
                            reverse_deps
                                .entry(key)
                                .or_default()
                                .insert(p_str.clone());
                        }
                    }
                }
            }
        }
    }

    let mut top_hubs: Vec<(&String, &usize)> = fan_in.iter().collect();
    top_hubs.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    let hubs: Vec<String> = top_hubs
        .into_iter()
        .take(8)
        .map(|(k, v)| format!("{k}({v})"))
        .collect();

    let impacted: Vec<String> = if !target.is_empty() {
        let stem = Path::new(target)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(target);
        reverse_deps
            .get(stem)
            .map(|set| set.iter().take(15).cloned().collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    AstBlastSummary {
        files: file_count,
        edges: edge_count,
        god_hubs: hubs,
        impacted,
    }
}

pub fn run_slice(args: &[String]) -> i32 {
    if args.len() < 2 {
        eprintln!("Usage: fe-core slice <file> <start:end | --symbol name> [--ctx 20]");
        return 1;
    }
    let file_path = &args[0];
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("@fe:err{{\"file\":\"{file_path}\",\"msg\":\"{e}\"}}");
            return 1;
        }
    };
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();

    let ctx: usize = args
        .iter()
        .position(|a| a == "--ctx")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let (start, end) = if args[1] == "--symbol" {
        let sym = args.get(2).map(|s| s.as_str()).unwrap_or("");
        let hit = lines
            .iter()
            .position(|l| l.contains(sym))
            .unwrap_or(0);
        let s = hit.saturating_sub(4) + 1;
        let e = (hit + ctx).min(total);
        (s, e)
    } else if let Some((s_str, e_str)) = args[1].split_once(':') {
        let s = s_str.parse::<usize>().unwrap_or(1).max(1);
        let e = e_str.parse::<usize>().unwrap_or(s + 30).min(total);
        (s, e)
    } else {
        let center = args[1].parse::<usize>().unwrap_or(1).max(1);
        let s = center.saturating_sub(ctx / 2).max(1);
        let e = (center + ctx / 2).min(total);
        (s, e)
    };

    println!(
        "@fe:slice{{\"file\":\"{}\",\"range\":\"{}:{}\",\"total\":{}}}",
        file_path, start, end, total
    );
    let start_idx = start.saturating_sub(1);
    let end_idx = end.min(total);
    for (offset, line) in lines[start_idx..end_idx].iter().enumerate() {
        println!("{:>4} | {}", start_idx + offset + 1, line);
    }
    0
}

pub fn run_ast_blast(args: &[String]) -> i32 {
    let dir = args
        .first()
        .filter(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .unwrap_or(".");
    let target = args
        .iter()
        .position(|a| a == "--target")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");

    let summary = analyze_ast_blast(Path::new(dir), target);

    let out = json!({
        "files": summary.files,
        "edges": summary.edges,
        "god_hubs": summary.god_hubs,
        "target": target,
        "blast_radius_count": summary.impacted.len(),
        "impacted": summary.impacted
    });
    println!("@fe:ast{}", out);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_ast_blast_scans_workspace_modules_and_hubs() {
        let summary = analyze_ast_blast(Path::new("src"), "CanhTheme");
        assert!(summary.files >= 20);
        assert!(summary.edges > 10);
        assert!(summary.god_hubs.iter().all(|h| !h.starts_with('(')));
    }
}
