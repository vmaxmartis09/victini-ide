use anyhow::{anyhow, Result};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

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
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_millis(1200))
            .build()
            .unwrap_or_default();
        Self { port, http_client }
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

    /// Captures a live PNG screenshot from the first active CDP tab via WebSocket (`Page.captureScreenshot`).
    pub async fn capture_first_tab_screenshot(&self) -> Result<Vec<u8>> {
        let tabs = self.list_tabs().await?;
        let ws_url = tabs
            .into_iter()
            .find_map(|t| t.ws_url)
            .ok_or_else(|| anyhow!("No debuggable CDP tab found on port {}", self.port))?;

        let (mut ws_stream, _) =
            tokio::time::timeout(Duration::from_millis(1500), connect_async(&ws_url)).await??;

        let req = json!({
            "id": 1,
            "method": "Page.captureScreenshot",
            "params": { "format": "png" }
        });
        ws_stream
            .send(Message::Text(req.to_string()))
            .await?;

        while let Ok(Some(Ok(msg))) =
            tokio::time::timeout(Duration::from_millis(1800), ws_stream.next()).await
        {
            if let Message::Text(txt) = msg {
                if let Ok(val) = serde_json::from_str::<Value>(&txt) {
                    if val.get("id").and_then(|i| i.as_i64()) == Some(1) {
                        if let Some(b64) = val
                            .get("result")
                            .and_then(|r| r.get("data"))
                            .and_then(|d| d.as_str())
                        {
                            let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
                            let _ = ws_stream.close(None).await;
                            return Ok(bytes);
                        }
                    }
                }
            }
        }
        Err(anyhow!("Timed out waiting for CDP Page.captureScreenshot"))
    }
}
