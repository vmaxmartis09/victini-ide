use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};

pub struct BrowserProfileManager {
    profile_dir: PathBuf,
    browser_bin: Option<PathBuf>,
    running_process: Option<Child>,
    pub cdp_port: u16,
}

impl BrowserProfileManager {
    pub fn new(cdp_port: u16) -> Self {
        let profile_dir = crate::config::paths::get_browser_profile_dir();
        let browser_bin = crate::config::paths::get_ego_browser_path();
        Self {
            profile_dir,
            browser_bin,
            running_process: None,
            cdp_port,
        }
    }

    pub fn ensure_profile_dir(&self) -> Result<()> {
        if !self.profile_dir.exists() {
            fs::create_dir_all(&self.profile_dir)
                .with_context(|| format!("Failed to create profile dir: {:?}", self.profile_dir))?;
        }
        Ok(())
    }

    /// Spawns Ego Lite or Chrome with Persistent Profile & CDP port
    /// Protects user 'Dad' space by keeping dedicated Victini Vibe workspace profile
    pub fn launch_or_connect(&mut self, initial_url: Option<&str>) -> Result<()> {
        self.ensure_profile_dir()?;
        let url = initial_url.unwrap_or("https://gitlab.com");

        // If browser is already running on CDP port, open a new tab via CdpClient
        if self.is_running() {
            let port = self.cdp_port;
            let url_owned = url.to_string();
            tokio::spawn(async move {
                let client = crate::browser_gitlab::cdp_client::CdpClient::new(port);
                let _ = client.open_url(&url_owned).await;
            });
            return Ok(());
        }

        let bin = self
            .browser_bin
            .as_ref()
            .context("No compatible browser (Ego Lite/Chrome) found on system")?;

        let child = Command::new(bin)
            .arg(format!("--remote-debugging-port={}", self.cdp_port))
            .arg(format!("--user-data-dir={}", self.profile_dir.to_string_lossy()))
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg(url)
            .spawn()
            .with_context(|| format!("Failed to launch browser: {:?}", bin))?;

        self.running_process = Some(child);
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.running_process {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    self.running_process = None;
                    false
                }
            }
        } else {
            false
        }
    }
}
