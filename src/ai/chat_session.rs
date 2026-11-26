use crate::ai::ElegansAgent;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuotedContext {
    pub source_msg_id: String,
    pub author_label: String,
    pub snippet: String,
    pub is_quote_block: bool, // true = ❝ Quote block, false = ↩ Reply reference
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionStepItem {
    pub icon: String,
    pub label: String,
    pub detail: String,
    pub is_done: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub agent: ElegansAgent,
    pub timestamp: String,
    pub content: String,
    pub quoted: Option<QuotedContext>,
    pub steps: Vec<ExecutionStepItem>,
    pub duration_secs: f32,
    pub is_streaming: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatThread {
    pub id: String,
    pub title: String,
    pub branch: String,
    pub updated_at: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatSessionManager {
    pub threads: Vec<ChatThread>,
    pub active_thread_idx: usize,
    #[serde(skip)]
    pub active_quote: Option<QuotedContext>,
}

impl ChatSessionManager {
    pub fn load_for_project(project_root: &Path, branch: &str) -> Self {
        let path = Self::storage_path(project_root);
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(mut mgr) = serde_json::from_str::<ChatSessionManager>(&raw) {
                if mgr.threads.is_empty() {
                    mgr.create_default_thread(branch);
                }
                if mgr.active_thread_idx >= mgr.threads.len() {
                    mgr.active_thread_idx = 0;
                }
                return mgr;
            }
        }

        let mut mgr = Self {
            threads: Vec::new(),
            active_thread_idx: 0,
            active_quote: None,
        };
        mgr.create_default_thread(branch);
        mgr
    }

    fn storage_path(project_root: &Path) -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let proj_name = project_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .replace(['/', '\\', ' '], "_");
        let dir = PathBuf::from(home).join(".victini");
        let _ = fs::create_dir_all(&dir);
        dir.join(format!("chats_{}.json", proj_name))
    }

    pub fn save(&self, project_root: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::storage_path(project_root), json);
        }
    }

    pub fn create_default_thread(&mut self, branch: &str) {
        let now = Local::now().format("%H:%M").to_string();
        let welcome_msg = ChatMessage {
            id: "MSG-INIT".to_string(),
            role: MessageRole::Assistant,
            agent: ElegansAgent::SeniorDev,
            timestamp: now.clone(),
            content: "⚡ **VICTINI IDE** sẵn sàng. Đã kết nối trực tiếp tới não bộ **Antigravity CLI (`~/.local/bin/agy`)**.\nNgài Vmax có thể gõ `@` để đính kèm file, `$` để gọi Skill, `/` để chạy lệnh nhanh, hoặc bấm **↩ Reply / ❝ Quote** trên bất kỳ tin nhắn nào để điều hướng chuyên sâu.".to_string(),
            quoted: None,
            steps: vec![
                ExecutionStepItem {
                    icon: "✓".to_string(),
                    label: "Search Workspace".to_string(),
                    detail: "Auto-discovered Git Worktrees, Skills & Agents".to_string(),
                    is_done: true,
                },
                ExecutionStepItem {
                    icon: "✓".to_string(),
                    label: "AGY Brain Link".to_string(),
                    detail: "~/.local/bin/agy verified online".to_string(),
                    is_done: true,
                },
            ],
            duration_secs: 0.4,
            is_streaming: false,
        };

        self.threads.insert(
            0,
            ChatThread {
                id: format!("TH-{}", Local::now().format("%H%M%S")),
                title: "Phiên Vibe Code mới".to_string(),
                branch: branch.to_string(),
                updated_at: now,
                messages: vec![welcome_msg],
            },
        );
        self.active_thread_idx = 0;
    }

    pub fn active_thread(&self) -> &ChatThread {
        &self.threads[self.active_thread_idx]
    }

    pub fn active_thread_mut(&mut self) -> &mut ChatThread {
        &mut self.threads[self.active_thread_idx]
    }

    pub fn add_user_message(&mut self, prompt: &str, agent: ElegansAgent) -> (String, Option<QuotedContext>) {
        let quote = self.active_quote.take();
        let now = Local::now().format("%H:%M").to_string();
        let thread = self.active_thread_mut();
        let msg_id = format!("MSG-{}", thread.messages.len() + 1);

        if thread.messages.len() <= 1 {
            let short_title: String = prompt.chars().take(34).collect();
            thread.title = if prompt.chars().count() > 34 {
                format!("{}…", short_title)
            } else {
                short_title
            };
        }
        thread.updated_at = now.clone();

        thread.messages.push(ChatMessage {
            id: msg_id.clone(),
            role: MessageRole::User,
            agent,
            timestamp: now,
            content: prompt.to_string(),
            quoted: quote.clone(),
            steps: Vec::new(),
            duration_secs: 0.0,
            is_streaming: false,
        });

        (msg_id, quote)
    }

    pub fn begin_assistant_stream(&mut self, agent: ElegansAgent, quoted: Option<QuotedContext>) -> String {
        let now = Local::now().format("%H:%M").to_string();
        let thread = self.active_thread_mut();
        let msg_id = format!("MSG-{}", thread.messages.len() + 1);

        thread.messages.push(ChatMessage {
            id: msg_id.clone(),
            role: MessageRole::Assistant,
            agent,
            timestamp: now,
            content: String::new(),
            quoted,
            steps: vec![ExecutionStepItem {
                icon: "⚡".to_string(),
                label: format!("Elegans Router → {}", agent.name()),
                detail: format!("Tier: {}", agent.ponytail_tier()),
                is_done: true,
            }],
            duration_secs: 0.1,
            is_streaming: true,
        });

        msg_id
    }

    pub fn append_stream_line(&mut self, msg_id: &str, line: &str, elapsed_secs: f32) {
        let thread = self.active_thread_mut();
        if let Some(msg) = thread.messages.iter_mut().find(|m| m.id == msg_id) {
            msg.duration_secs = elapsed_secs;
            let trimmed = line.trim();

            if trimmed.starts_with("🏷️") {
                return;
            }

            // Detect tool / command / step lines vs content lines
            if trimmed.starts_with("$ ")
                || trimmed.starts_with("Running ")
                || trimmed.starts_with("Searching ")
                || trimmed.starts_with("Reading ")
                || trimmed.starts_with("Editing ")
            {
                msg.steps.push(ExecutionStepItem {
                    icon: "✓".to_string(),
                    label: "Action".to_string(),
                    detail: trimmed.to_string(),
                    is_done: true,
                });
            } else {
                if !msg.content.is_empty() {
                    msg.content.push('\n');
                }
                msg.content.push_str(line.trim_end_matches(['\r', '\n']));
            }
        }
    }

    pub fn finish_assistant_stream(&mut self, msg_id: &str, elapsed_secs: f32) {
        let thread = self.active_thread_mut();
        if let Some(msg) = thread.messages.iter_mut().find(|m| m.id == msg_id) {
            msg.is_streaming = false;
            msg.duration_secs = elapsed_secs.max(msg.duration_secs);
            if msg.content.trim().is_empty() {
                msg.content = "✓ Đã hoàn tất thực thi yêu cầu.".to_string();
            }
        }
    }

    pub fn push_system_notice(&mut self, text: impl Into<String>) {
        let now = Local::now().format("%H:%M").to_string();
        let thread = self.active_thread_mut();
        let msg_id = format!("SYS-{}", thread.messages.len() + 1);
        thread.messages.push(ChatMessage {
            id: msg_id,
            role: MessageRole::System,
            agent: ElegansAgent::SeniorDev,
            timestamp: now,
            content: text.into(),
            quoted: None,
            steps: Vec::new(),
            duration_secs: 0.0,
            is_streaming: false,
        });
    }
}
