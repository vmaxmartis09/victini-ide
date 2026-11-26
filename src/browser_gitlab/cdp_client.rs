use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetTab {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub ws_url: Option<String>,
}

pub struct CdpClient {
    port: u16,
    http_client: reqwest::Client,
}

impl CdpClient {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn list_tabs(&self) -> Result<Vec<TargetTab>> {
        let endpoint = format!("http://127.0.0.1:{}/json/list", self.port);
        let resp = self.http_client.get(&endpoint).send().await?;
        let tabs: Vec<TargetTab> = resp.json().await?;
        Ok(tabs)
    }

    pub async fn open_url(&self, url: &str) -> Result<TargetTab> {
        let endpoint = format!("http://127.0.0.1:{}/json/new?{}", self.port, url);
        let resp = self.http_client.put(&endpoint).send().await?;
        let tab: TargetTab = resp.json().await?;
        Ok(tab)
    }

    pub async fn close_tab(&self, target_id: &str) -> Result<()> {
        let endpoint = format!("http://127.0.0.1:{}/json/close/{}", self.port, target_id);
        self.http_client.get(&endpoint).send().await?;
        Ok(())
    }
}
