use crate::ai::ElegansAgent;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc as std_mpsc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KanbanColumn {
    Backlog,
    InProgress,
    Verifying,
    Done,
}

impl KanbanColumn {
    pub fn title(&self) -> &'static str {
        match self {
            Self::Backlog => "BACKLOG",
            Self::InProgress => "IN PROGRESS ⚡",
            Self::Verifying => "VERIFYING 📱",
            Self::Done => "DONE ✨",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTaskCard {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: String, // "P0", "P1", "P2"
    pub assigned_agent: ElegansAgent,
    pub custom_agent_name: Option<String>,
    pub worktree_branch: String,
    pub column: KanbanColumn,
    pub progress_pct: f32,
    pub time_spent_secs: u64,
    pub live_output: String,
    pub full_logs: Vec<String>,
    #[serde(skip)]
    pub is_running_bg: bool,
    pub auto_schedule_idle: bool,
}

#[derive(Clone, Debug)]
pub enum BgAgentEvent {
    Started {
        task_id: String,
    },
    OutputLine {
        task_id: String,
        line: String,
        elapsed_secs: u64,
    },
    Completed {
        task_id: String,
        elapsed_secs: u64,
        success: bool,
    },
}

#[derive(Serialize, Deserialize)]
pub struct AgentKanbanManager {
    pub tasks: Vec<AgentTaskCard>,
    pub auto_run_backlog_on_idle: bool,
    #[serde(skip)]
    pub bg_sender: Option<std_mpsc::Sender<BgAgentEvent>>,
    #[serde(skip)]
    pub bg_receiver: Option<std_mpsc::Receiver<BgAgentEvent>>,
}

impl AgentKanbanManager {
    pub fn load_for_project(project_root: &Path) -> Self {
        let (tx, rx) = std_mpsc::channel();
        let path = Self::storage_path(project_root);
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(mut mgr) = serde_json::from_str::<AgentKanbanManager>(&raw) {
                mgr.bg_sender = Some(tx);
                mgr.bg_receiver = Some(rx);
                return mgr;
            }
        }

        let mut mgr = Self::default_workspace_board();
        mgr.bg_sender = Some(tx);
        mgr.bg_receiver = Some(rx);
        mgr.save(project_root);
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
        dir.join(format!("kanban_{}.json", proj_name))
    }

    pub fn save(&self, project_root: &Path) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::storage_path(project_root), json);
        }
    }

    pub fn default_workspace_board() -> Self {
        Self {
            auto_run_backlog_on_idle: false,
            bg_sender: None,
            bg_receiver: None,
            tasks: vec![
                AgentTaskCard {
                    id: "TASK-01".to_string(),
                    title: "Adaptive Responsive Layout & 12 Canh Giờ Physics".to_string(),
                    description: "Đảm bảo 3 panel co giãn mượt theo kích thước cửa sổ, tích hợp SharedTicker::damp.".to_string(),
                    priority: "P0".to_string(),
                    assigned_agent: ElegansAgent::UiUxCritic,
                    custom_agent_name: None,
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::Done,
                    progress_pct: 1.0,
                    time_spent_secs: 14,
                    live_output: "✓ Responsive 3 breakpoints & Notionists motion avatars active".to_string(),
                    full_logs: vec!["✓ Đã kích hoạt Responsive Breakpoints (Compact/Medium/Wide)".to_string()],
                    is_running_bg: false,
                    auto_schedule_idle: false,
                },
                AgentTaskCard {
                    id: "TASK-02".to_string(),
                    title: "Structured Chat Engine, Reply & Quote Context".to_string(),
                    description: "Hỗ trợ trả lời tin nhắn, trích dẫn block code và stream trực tiếp từ ~/.local/bin/agy.".to_string(),
                    priority: "P0".to_string(),
                    assigned_agent: ElegansAgent::SeniorDev,
                    custom_agent_name: None,
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::Done,
                    progress_pct: 1.0,
                    time_spent_secs: 22,
                    live_output: "✓ Reply & Quote banner nối trực tiếp vào AGY prompt".to_string(),
                    full_logs: vec!["✓ Hoàn tất cấu trúc ChatSessionManager".to_string()],
                    is_running_bg: false,
                    auto_schedule_idle: false,
                },
                AgentTaskCard {
                    id: "TASK-03".to_string(),
                    title: "Kiểm định 5 Viewports & Live Screencast trên BeeBox".to_string(),
                    description: "Chụp màn hình trực tiếp qua ADB/CDP và kiểm tra chạm cảm ứng trên 5 kích thước màn hình.".to_string(),
                    priority: "P1".to_string(),
                    assigned_agent: ElegansAgent::QaTester,
                    custom_agent_name: None,
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::InProgress,
                    progress_pct: 0.72,
                    time_spent_secs: 18,
                    live_output: "📱 Đang đồng bộ viewport iPhone 15 Pro (390×844)...".to_string(),
                    full_logs: vec!["📱 Khởi tạo bộ khung giả lập BeeBox".to_string()],
                    is_running_bg: false,
                    auto_schedule_idle: true,
                },
                AgentTaskCard {
                    id: "TASK-04".to_string(),
                    title: "Rà soát bảo mật & Tối ưu GitLab Merge Request".to_string(),
                    description: "Kiểm tra diff trước khi commit/push và tạo link MR tự động trên Ego Lite.".to_string(),
                    priority: "P1".to_string(),
                    assigned_agent: ElegansAgent::CodeAuditor,
                    custom_agent_name: None,
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::Backlog,
                    progress_pct: 0.0,
                    time_spent_secs: 0,
                    live_output: "Sẵn sàng chạy ngầm (Click ▶ Run BG để giao Agent)".to_string(),
                    full_logs: Vec::new(),
                    is_running_bg: false,
                    auto_schedule_idle: true,
                },
            ],
        }
    }

    pub fn create_task(
        &mut self,
        title: &str,
        description: &str,
        priority: &str,
        agent: ElegansAgent,
        custom_agent: Option<String>,
        branch: &str,
        run_immediately: bool,
        project_root: &Path,
    ) -> String {
        let next_num = self.tasks.len() + 1;
        let id = format!("TASK-{:02}", next_num);
        let card = AgentTaskCard {
            id: id.clone(),
            title: title.trim().to_string(),
            description: description.trim().to_string(),
            priority: priority.to_string(),
            assigned_agent: agent,
            custom_agent_name: custom_agent,
            worktree_branch: branch.to_string(),
            column: if run_immediately {
                KanbanColumn::InProgress
            } else {
                KanbanColumn::Backlog
            },
            progress_pct: 0.0,
            time_spent_secs: 0,
            live_output: "Đã khởi tạo task — Sẵn sàng thực thi".to_string(),
            full_logs: Vec::new(),
            is_running_bg: false,
            auto_schedule_idle: true,
        };
        self.tasks.push(card);
        if run_immediately {
            self.spawn_background_agent(&id, project_root.to_path_buf());
        }
        self.save(project_root);
        id
    }

    pub fn assign_agent(
        &mut self,
        task_id: &str,
        agent: ElegansAgent,
        custom_name: Option<String>,
        project_root: &Path,
    ) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.assigned_agent = agent;
            t.custom_agent_name = custom_name;
            t.live_output = format!("⚡ Đã gán cho {} ({})", agent.name(), agent.title());
        }
        self.save(project_root);
    }

    pub fn move_task_left(&mut self, task_id: &str, project_root: &Path) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.column = match t.column {
                KanbanColumn::Backlog => KanbanColumn::Backlog,
                KanbanColumn::InProgress => KanbanColumn::Backlog,
                KanbanColumn::Verifying => KanbanColumn::InProgress,
                KanbanColumn::Done => KanbanColumn::Verifying,
            };
        }
        self.save(project_root);
    }

    pub fn advance_task(&mut self, task_id: &str, project_root: &Path) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.column = match t.column {
                KanbanColumn::Backlog => KanbanColumn::InProgress,
                KanbanColumn::InProgress => KanbanColumn::Verifying,
                KanbanColumn::Verifying => KanbanColumn::Done,
                KanbanColumn::Done => KanbanColumn::Backlog,
            };
            if t.column == KanbanColumn::Done {
                t.progress_pct = 1.0;
            }
        }
        self.save(project_root);
    }

    pub fn delete_task(&mut self, task_id: &str, project_root: &Path) {
        self.tasks.retain(|t| t.id != task_id);
        self.save(project_root);
    }

    /// Spawns a real background `agy -p` worker for this Kanban card without blocking the main UI or chat
    pub fn spawn_background_agent(&mut self, task_id: &str, workdir: PathBuf) {
        let Some(tx) = self.bg_sender.clone() else {
            return;
        };
        let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) else {
            return;
        };
        if task.is_running_bg {
            return;
        }

        task.is_running_bg = true;
        task.column = KanbanColumn::InProgress;
        task.progress_pct = 0.12;
        task.live_output = format!("⚡ [Background {}] Đang khởi chạy AGY...", task.assigned_agent.name());

        let id_clone = task.id.clone();
        let prompt = format!(
            "[Swarm Background Task {} - Role: {} ({})]\nNhiệm vụ: {}\nChi tiết: {}\nHãy trả lời ngắn gọn các bước kiểm tra và kết quả thực hiện.",
            task.id,
            task.assigned_agent.title(),
            task.custom_agent_name.as_deref().unwrap_or(task.assigned_agent.name()),
            task.title,
            task.description
        );

        tokio::spawn(async move {
            let start = Instant::now();
            let _ = tx.send(BgAgentEvent::Started {
                task_id: id_clone.clone(),
            });

            let agy_bin = crate::config::paths::get_agy_path();
            let mut cmd = Command::new(&agy_bin);
            cmd.current_dir(&workdir);
            cmd.arg("-p").arg(&prompt);
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            match cmd.spawn() {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let mut reader = BufReader::new(stdout).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                let _ = tx.send(BgAgentEvent::OutputLine {
                                    task_id: id_clone.clone(),
                                    line: trimmed.to_string(),
                                    elapsed_secs: start.elapsed().as_secs(),
                                });
                            }
                        }
                    }
                    let status = child.wait().await;
                    let ok = status.map(|s| s.success()).unwrap_or(false);
                    let _ = tx.send(BgAgentEvent::Completed {
                        task_id: id_clone,
                        elapsed_secs: start.elapsed().as_secs(),
                        success: ok,
                    });
                }
                Err(e) => {
                    let _ = tx.send(BgAgentEvent::OutputLine {
                        task_id: id_clone.clone(),
                        line: format!("❌ Lỗi chạy ngầm AGY: {}", e),
                        elapsed_secs: start.elapsed().as_secs(),
                    });
                    let _ = tx.send(BgAgentEvent::Completed {
                        task_id: id_clone,
                        elapsed_secs: start.elapsed().as_secs(),
                        success: false,
                    });
                }
            }
        });
    }

    /// Drains background agent telemetry events each UI frame
    pub fn poll_background_events(&mut self, project_root: &Path) -> bool {
        let mut updated = false;
        let mut events = Vec::new();
        if let Some(ref rx) = self.bg_receiver {
            while let Ok(ev) = rx.try_recv() {
                events.push(ev);
            }
        }

        for ev in events {
            updated = true;
            match ev {
                BgAgentEvent::Started { task_id } => {
                    if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        t.is_running_bg = true;
                        t.progress_pct = 0.2;
                    }
                }
                BgAgentEvent::OutputLine {
                    task_id,
                    line,
                    elapsed_secs,
                } => {
                    if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        t.time_spent_secs = elapsed_secs;
                        t.live_output = line.clone();
                        t.full_logs.push(line);
                        t.progress_pct = (t.progress_pct + 0.12).min(0.92);
                    }
                }
                BgAgentEvent::Completed {
                    task_id,
                    elapsed_secs,
                    success,
                } => {
                    if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        t.is_running_bg = false;
                        t.time_spent_secs = elapsed_secs.max(t.time_spent_secs);
                        t.progress_pct = 1.0;
                        if success {
                            t.column = KanbanColumn::Verifying;
                            if t.full_logs.is_empty() {
                                t.live_output = "✓ Agent chạy ngầm hoàn tất — Chờ nghiệm thu".to_string();
                            }
                        }
                    }
                }
            }
        }

        if updated {
            self.save(project_root);
        }
        updated
    }

    pub fn any_background_running(&self) -> bool {
        self.tasks.iter().any(|t| t.is_running_bg)
    }
}
