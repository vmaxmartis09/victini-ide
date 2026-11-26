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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadAgentMode {
    SwarmAuto,
    DedicatedCore(ElegansAgent),
    DedicatedCustom {
        name: String,
        role: String,
        system_prompt: String,
    },
}

impl Default for ThreadAgentMode {
    fn default() -> Self {
        Self::SwarmAuto
    }
}

impl ThreadAgentMode {
    pub fn badge_label(&self, fallback: ElegansAgent) -> String {
        match self {
            Self::SwarmAuto => format!("Swarm Auto · {}", fallback.name()),
            Self::DedicatedCore(ag) => format!("1-on-1 · {}", ag.name()),
            Self::DedicatedCustom { name, .. } => format!("1-on-1 · @{}", name),
        }
    }

    pub fn short_tag(&self) -> String {
        match self {
            Self::SwarmAuto => "Swarm Auto".to_string(),
            Self::DedicatedCore(ag) => ag.name().to_string(),
            Self::DedicatedCustom { name, .. } => format!("@{}", name),
        }
    }

    pub fn role_description(&self) -> String {
        match self {
            Self::SwarmAuto => "Tự động điều phối Subagent theo ngữ cảnh câu hỏi".to_string(),
            Self::DedicatedCore(ag) => format!("{} ({})", ag.title(), ag.ponytail_tier()),
            Self::DedicatedCustom { role, .. } => role.clone(),
        }
    }

    pub fn resolve_core_agent(&self, fallback: ElegansAgent) -> ElegansAgent {
        match self {
            Self::SwarmAuto => fallback,
            Self::DedicatedCore(ag) => *ag,
            Self::DedicatedCustom { .. } => ElegansAgent::SeniorDev,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuotedContext {
    pub source_msg_id: String,
    pub author_label: String,
    pub snippet: String,
    pub is_quote_block: bool,
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
    #[serde(default)]
    pub agent_mode: ThreadAgentMode,
    #[serde(default)]
    pub attached_files: Vec<String>,
    #[serde(default)]
    pub attached_skills: Vec<String>,
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
            content: "**VICTINI IDE** sẵn sàng. Đã kết nối trực tiếp tới não bộ **Antigravity CLI (`~/.local/bin/agy`)**.\nNgài Vmax có thể:\n- Gõ `@` hoặc bấm nút `+` để đính kèm file vào ngữ cảnh\n- Gõ `$` để kích hoạt Skill (`SKILL.md`), hoặc `/` để chạy lệnh nhanh (`/diff`, `/commit`, `/review`, `/kanban`)\n- Chọn tab **Agents** ở Sidebar trái hoặc bấm vào pill Agent ở góc dưới để **Chat 1-1 với từng Agent riêng biệt**.".to_string(),
            quoted: None,
            steps: vec![
                ExecutionStepItem {
                    icon: "✓".to_string(),
                    label: "Workspace Indexed".to_string(),
                    detail: "Auto-discovered Git Worktrees, Changes, Skills & Custom Subagents".to_string(),
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
                title: "Swarm Vibe Session".to_string(),
                branch: branch.to_string(),
                updated_at: now,
                agent_mode: ThreadAgentMode::SwarmAuto,
                attached_files: Vec::new(),
                attached_skills: Vec::new(),
                messages: vec![welcome_msg],
            },
        );
        self.active_thread_idx = 0;
    }

    /// Open or resume a dedicated 1-on-1 chat thread with one of the 6 Core Elegans Agents
    pub fn open_or_create_dedicated_core_thread(
        &mut self,
        agent: ElegansAgent,
        branch: &str,
    ) -> usize {
        if let Some((idx, _)) = self
            .threads
            .iter()
            .enumerate()
            .find(|(_, t)| t.agent_mode == ThreadAgentMode::DedicatedCore(agent))
        {
            self.active_thread_idx = idx;
            return idx;
        }

        let now = Local::now().format("%H:%M").to_string();
        let intro = ChatMessage {
            id: "MSG-AGENT-INIT".to_string(),
            role: MessageRole::Assistant,
            agent,
            timestamp: now.clone(),
            content: format!(
                "**[Phiên Chat 1-1 Chuyên Biệt · {}]**\nChào Ngài Vmax, con là **{}** (*{}* — Tier `{}`).\nMọi yêu cầu trong phiên này sẽ được con xử lý trực tiếp theo đúng chuyên môn mà không qua bộ định tuyến chung. Ngài có thể đính kèm `@file` hoặc `$skill` để con làm việc ngay.",
                agent.name(),
                agent.name(),
                agent.title(),
                agent.ponytail_tier()
            ),
            quoted: None,
            steps: vec![ExecutionStepItem {
                icon: "✓".to_string(),
                label: format!("Dedicated 1-on-1 Lock -> {}", agent.name()),
                detail: format!("Role: {} | Tier: {}", agent.title(), agent.ponytail_tier()),
                is_done: true,
            }],
            duration_secs: 0.2,
            is_streaming: false,
        };

        self.threads.insert(
            0,
            ChatThread {
                id: format!("TH-{}-{}", agent.name(), Local::now().format("%H%M%S")),
                title: format!("1-1: {}", agent.name()),
                branch: branch.to_string(),
                updated_at: now,
                agent_mode: ThreadAgentMode::DedicatedCore(agent),
                attached_files: Vec::new(),
                attached_skills: Vec::new(),
                messages: vec![intro],
            },
        );
        self.active_thread_idx = 0;
        0
    }

    /// Open or resume a dedicated 1-on-1 chat thread with a Custom Subagent (.claude/agents/*.md)
    pub fn open_or_create_dedicated_custom_thread(
        &mut self,
        name: &str,
        role: &str,
        system_prompt: &str,
        branch: &str,
    ) -> usize {
        if let Some((idx, _)) = self.threads.iter().enumerate().find(|(_, t)| {
            matches!(
                &t.agent_mode,
                ThreadAgentMode::DedicatedCustom { name: n, .. } if n == name
            )
        }) {
            self.active_thread_idx = idx;
            return idx;
        }

        let now = Local::now().format("%H:%M").to_string();
        let intro = ChatMessage {
            id: "MSG-CUSTOM-INIT".to_string(),
            role: MessageRole::Assistant,
            agent: ElegansAgent::SeniorDev,
            timestamp: now.clone(),
            content: format!(
                "**[Phiên Chat 1-1 Subagent · @{}]**\nVai trò: *{}*\nSystem Prompt riêng của `@{} ` đã được nạp vào phiên làm việc này. Ngài Vmax cần con thực thi tác vụ gì?",
                name, role, name
            ),
            quoted: None,
            steps: vec![ExecutionStepItem {
                icon: "✓".to_string(),
                label: format!("Loaded Subagent @{}.md", name),
                detail: role.to_string(),
                is_done: true,
            }],
            duration_secs: 0.2,
            is_streaming: false,
        };

        self.threads.insert(
            0,
            ChatThread {
                id: format!("TH-CUSTOM-{}", Local::now().format("%H%M%S")),
                title: format!("1-1: @{}", name),
                branch: branch.to_string(),
                updated_at: now,
                agent_mode: ThreadAgentMode::DedicatedCustom {
                    name: name.to_string(),
                    role: role.to_string(),
                    system_prompt: system_prompt.to_string(),
                },
                attached_files: Vec::new(),
                attached_skills: Vec::new(),
                messages: vec![intro],
            },
        );
        self.active_thread_idx = 0;
        0
    }

    pub fn active_thread(&self) -> &ChatThread {
        &self.threads[self.active_thread_idx]
    }

    pub fn active_thread_mut(&mut self) -> &mut ChatThread {
        &mut self.threads[self.active_thread_idx]
    }

    pub fn toggle_attached_file(&mut self, rel_path: String) {
        let files = &mut self.active_thread_mut().attached_files;
        if let Some(pos) = files.iter().position(|p| p == &rel_path) {
            files.remove(pos);
        } else {
            files.push(rel_path);
        }
    }

    pub fn toggle_attached_skill(&mut self, skill_name: String) {
        let skills = &mut self.active_thread_mut().attached_skills;
        if let Some(pos) = skills.iter().position(|s| s == &skill_name) {
            skills.remove(pos);
        } else {
            skills.push(skill_name);
        }
    }

    pub fn add_user_message(
        &mut self,
        prompt: &str,
        agent: ElegansAgent,
    ) -> (String, Option<QuotedContext>) {
        let quote = self.active_quote.take();
        let now = Local::now().format("%H:%M").to_string();
        let thread = self.active_thread_mut();
        let msg_id = format!("MSG-{}", thread.messages.len() + 1);

        if thread.messages.len() <= 1 && matches!(thread.agent_mode, ThreadAgentMode::SwarmAuto) {
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

    pub fn begin_assistant_stream(
        &mut self,
        agent: ElegansAgent,
        quoted: Option<QuotedContext>,
    ) -> String {
        let now = Local::now().format("%H:%M").to_string();
        let thread = self.active_thread_mut();
        let msg_id = format!("MSG-{}", thread.messages.len() + 1);
        let route_label = match &thread.agent_mode {
            ThreadAgentMode::SwarmAuto => format!("Swarm Router -> {}", agent.name()),
            ThreadAgentMode::DedicatedCore(ag) => format!("Dedicated 1-on-1 -> {}", ag.name()),
            ThreadAgentMode::DedicatedCustom { name, .. } => {
                format!("Custom Subagent -> @{}", name)
            }
        };

        let mut init_steps = vec![ExecutionStepItem {
            icon: "✓".to_string(),
            label: route_label,
            detail: thread.agent_mode.role_description(),
            is_done: true,
        }];

        for f in &thread.attached_files {
            init_steps.push(ExecutionStepItem {
                icon: "✓".to_string(),
                label: format!("Attached @{}", f),
                detail: "Injected file context into AGY prompt".to_string(),
                is_done: true,
            });
        }
        for s in &thread.attached_skills {
            init_steps.push(ExecutionStepItem {
                icon: "✓".to_string(),
                label: format!("Skill ${}", s),
                detail: "Loaded SKILL.md instructions".to_string(),
                is_done: true,
            });
        }

        thread.messages.push(ChatMessage {
            id: msg_id.clone(),
            role: MessageRole::Assistant,
            agent,
            timestamp: now,
            content: String::new(),
            quoted,
            steps: init_steps,
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
