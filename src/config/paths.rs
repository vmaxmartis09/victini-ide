use std::path::{Path, PathBuf};

pub fn detect_workspace_root() -> PathBuf {
    if let Ok(c) = std::env::current_dir() {
        let mut cur = c.as_path();
        loop {
            if cur.join(".git").exists() || cur.join("Cargo.toml").exists() || cur.join("package.json").exists() {
                return cur.to_path_buf();
            }
            match cur.parent() {
                Some(p) => cur = p,
                None => break,
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn get_agy_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let p = PathBuf::from(&userprofile).join(".gemini\\bin\\agy.exe");
            if p.exists() { return p; }
            let p2 = PathBuf::from(&userprofile).join(".local\\bin\\agy.exe");
            if p2.exists() { return p2; }
        }
        PathBuf::from("agy.exe")
    } else {
        if let Ok(home) = std::env::var("HOME") {
            let p1 = PathBuf::from(&home).join(".local/bin/agy");
            if p1.exists() { return p1; }
            let p2 = PathBuf::from(&home).join(".gemini/bin/agy");
            if p2.exists() { return p2; }
            let p3 = PathBuf::from("/usr/local/bin/agy");
            if p3.exists() { return p3; }
        }
        PathBuf::from("agy")
    }
}

pub fn get_ego_browser_path() -> Option<PathBuf> {
    let candidates = [
        "/Applications/Ego Lite.app/Contents/MacOS/ego-browser",
        "/Applications/Ego.app/Contents/MacOS/ego-browser",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
        "C:\\Users\\dtthoai\\.local\\share\\ego\\active_version_dir\\ego-browser.exe",
        "D:\\Tools\\Browser\\Ego Lite\\ego lite\\Application\\ego.exe",
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    ];

    for c in &candidates {
        let p = Path::new(c);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

pub fn get_browser_profile_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(appdata).join("agy-vibe-ide").join("browser-profile")
        } else {
            PathBuf::from(".agy-browser-profile")
        }
    } else {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join("Library/Application Support/agy-vibe-ide/browser-profile")
        } else {
            PathBuf::from(".agy-browser-profile")
        }
    }
}
