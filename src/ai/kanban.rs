use crate::ai::ElegansAgent;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::mpsc as std_mpsc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            Self::InProgress => "IN PROGRESS",
            Self::Verifying => "VERIFYING",
            Self::Done => "DONE",
        }
    }

    pub fn all() -> [KanbanColumn; 4] {
        [
            Self::Backlog,
            Self::InProgress,
            Self::Verifying,
            Self::Done,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTaskCard {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: String, // "P0", "P1", "P2"
    pub assigned_agent: ElegansAgent,
    #[serde(default)]
    pub custom_agent_name: Option<String>,
    #[serde(default)]
    pub attached_skills: Vec<String>,
    pub worktree_branch: String,
    pub column: KanbanColumn,
    pub progress_pct: f32,
    pub time_spent_secs: u64,
    pub live_output: String,
    pub full_logs: Vec<String>,
    #[serde(skip)]
    pub is_running_bg: bool,
    #[serde(default)]
    pub run_failed: bool,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub review_note: Option<String>,
    #[serde(default)]
    pub auto_schedule_idle: bool,
    #[serde(default)]
    pub created_at: String,
}

impl AgentTaskCard {
    pub fn display_agent_name(&self) -> &str {
        self.custom_agent_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| self.assigned_agent.name())
    }
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
    Stopped {
        task_id: String,
        elapsed_secs: u64,
    },
    Completed {
        task_id: String,
        elapsed_secs: u64,
        success: bool,
        error: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct AgentKanbanManager {
    pub tasks: Vec<AgentTaskCard>,
    pub auto_run_backlog_on_idle: bool,
    #[serde(default)]
    pub next_task_seq: u32,
    #[serde(skip)]
    pub bg_sender: Option<std_mpsc::Sender<BgAgentEvent>>,
    #[serde(skip)]
    pub bg_receiver: Option<std_mpsc::Receiver<BgAgentEvent>>,
    #[serde(skip)]
    cancel_handles: HashMap<String, oneshot::Sender<()>>,
}

impl AgentKanbanManager {
    pub fn load_for_project(project_root: &Path) -> Self {
        let (tx, rx) = std_mpsc::channel();
        let path = Self::storage_path(project_root);
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(mut mgr) = serde_json::from_str::<AgentKanbanManager>(&raw) {
                mgr.sync_next_task_seq();
                mgr.bg_sender = Some(tx);
                mgr.bg_receiver = Some(rx);
                mgr.cancel_handles = HashMap::new();
                return mgr;
            }
        }

        let mut mgr = Self::default_workspace_board();
        mgr.sync_next_task_seq();
        mgr.bg_sender = Some(tx);
        mgr.bg_receiver = Some(rx);
        mgr.cancel_handles = HashMap::new();
        mgr.save(project_root);
        mgr
    }

    fn sync_next_task_seq(&mut self) {
        let max_existing = self
            .tasks
            .iter()
            .filter_map(|t| {
                t.id.strip_prefix("TASK-")
                    .and_then(|num| num.parse::<u32>().ok())
            })
            .max()
            .unwrap_or(0);
        if self.next_task_seq <= max_existing {
            self.next_task_seq = max_existing + 1;
        }
        if self.next_task_seq == 0 {
            self.next_task_seq = 1;
        }
    }

    pub fn allocate_next_id(&mut self) -> String {
        self.sync_next_task_seq();
        let seq = self.next_task_seq;
        self.next_task_seq = seq + 1;
        format!("TASK-{:02}", seq)
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
        let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
        Self {
            auto_run_backlog_on_idle: false,
            next_task_seq: 5,
            bg_sender: None,
            bg_receiver: None,
            cancel_handles: HashMap::new(),
            tasks: vec![
                AgentTaskCard {
                    id: "TASK-01".to_string(),
                    title: "Adaptive Responsive Layout & Smooth Physics".to_string(),
                    description: "Đảm bảo 3 panel co giãn mượt theo kích thước cửa sổ, tích hợp SharedTicker::damp.".to_string(),
                    priority: "P0".to_string(),
                    assigned_agent: ElegansAgent::UiUxCritic,
                    custom_agent_name: None,
                    attached_skills: Vec::new(),
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::Done,
                    progress_pct: 1.0,
                    time_spent_secs: 14,
                    live_output: "Hoàn tất 3 breakpoints responsive và Notionists motion avatars.".to_string(),
                    full_logs: vec!["[10:00:01] Đã kích hoạt Responsive Breakpoints (Compact/Medium/Wide)".to_string()],
                    is_running_bg: false,
                    run_failed: false,
                    last_error: None,
                    review_note: None,
                    auto_schedule_idle: false,
                    created_at: now.clone(),
                },
                AgentTaskCard {
                    id: "TASK-02".to_string(),
                    title: "Structured Chat Engine, Reply & Quote Context".to_string(),
                    description: "Hỗ trợ trả lời tin nhắn, trích dẫn block code và stream trực tiếp từ agy.".to_string(),
                    priority: "P0".to_string(),
                    assigned_agent: ElegansAgent::SeniorDev,
                    custom_agent_name: None,
                    attached_skills: Vec::new(),
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::Done,
                    progress_pct: 1.0,
                    time_spent_secs: 22,
                    live_output: "Reply & Quote banner đã nối trực tiếp vào AGY prompt.".to_string(),
                    full_logs: vec!["[10:05:12] Hoàn tất cấu trúc ChatSessionManager".to_string()],
                    is_running_bg: false,
                    run_failed: false,
                    last_error: None,
                    review_note: None,
                    auto_schedule_idle: false,
                    created_at: now.clone(),
                },
                AgentTaskCard {
                    id: "TASK-03".to_string(),
                    title: "Kiểm định 5 Viewports & Live Screencast trên BeeBox".to_string(),
                    description: "Chụp màn hình trực tiếp qua ADB/CDP và kiểm tra chạm cảm ứng trên 5 kích thước màn hình.".to_string(),
                    priority: "P1".to_string(),
                    assigned_agent: ElegansAgent::QaTester,
                    custom_agent_name: None,
                    attached_skills: Vec::new(),
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::InProgress,
                    progress_pct: 0.72,
                    time_spent_secs: 18,
                    live_output: "Đang đồng bộ viewport iPhone 15 Pro (390x844)...".to_string(),
                    full_logs: vec!["[10:12:00] Khởi tạo bộ khung giả lập BeeBox".to_string()],
                    is_running_bg: false,
                    run_failed: false,
                    last_error: None,
                    review_note: None,
                    auto_schedule_idle: true,
                    created_at: now.clone(),
                },
                AgentTaskCard {
                    id: "TASK-04".to_string(),
                    title: "Rà soát bảo mật & Tối ưu GitLab Merge Request".to_string(),
                    description: "Kiểm tra diff trước khi commit/push và tạo link MR tự động trên Ego Lite.".to_string(),
                    priority: "P1".to_string(),
                    assigned_agent: ElegansAgent::CodeAuditor,
                    custom_agent_name: None,
                    attached_skills: Vec::new(),
                    worktree_branch: "main".to_string(),
                    column: KanbanColumn::Backlog,
                    progress_pct: 0.0,
                    time_spent_secs: 0,
                    live_output: "Sẵn sàng chạy ngầm (bấm Chạy nền để giao Agent).".to_string(),
                    full_logs: Vec::new(),
                    is_running_bg: false,
                    run_failed: false,
                    last_error: None,
                    review_note: None,
                    auto_schedule_idle: true,
                    created_at: now,
                },
            ],
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_task(
        &mut self,
        title: &str,
        description: &str,
        priority: &str,
        agent: ElegansAgent,
        custom_agent: Option<String>,
        attached_skills: Vec<String>,
        branch: &str,
        column: KanbanColumn,
        auto_schedule_idle: bool,
        run_immediately: bool,
        project_root: &Path,
    ) -> String {
        let id = self.allocate_next_id();
        let target_col = if run_immediately {
            KanbanColumn::InProgress
        } else {
            column
        };
        let card = AgentTaskCard {
            id: id.clone(),
            title: title.trim().to_string(),
            description: description.trim().to_string(),
            priority: priority.to_string(),
            assigned_agent: agent,
            custom_agent_name: custom_agent.filter(|s| !s.trim().is_empty()),
            attached_skills,
            worktree_branch: branch.to_string(),
            column: target_col,
            progress_pct: if target_col == KanbanColumn::Done {
                1.0
            } else {
                0.0
            },
            time_spent_secs: 0,
            live_output: "Đã khởi tạo task — Sẵn sàng thực thi".to_string(),
            full_logs: Vec::new(),
            is_running_bg: false,
            run_failed: false,
            last_error: None,
            review_note: None,
            auto_schedule_idle,
            created_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
        };
        self.tasks.push(card);
        if run_immediately {
            self.spawn_background_agent(&id, project_root.to_path_buf(), None);
        }
        self.save(project_root);
        id
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_task(
        &mut self,
        task_id: &str,
        title: String,
        description: String,
        priority: String,
        agent: ElegansAgent,
        custom_agent: Option<String>,
        attached_skills: Vec<String>,
        branch: String,
        column: KanbanColumn,
        auto_schedule_idle: bool,
        project_root: &Path,
    ) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if !title.trim().is_empty() {
                t.title = title.trim().to_string();
            }
            t.description = description;
            t.priority = priority;
            t.assigned_agent = agent;
            t.custom_agent_name = custom_agent.filter(|s| !s.trim().is_empty());
            t.attached_skills = attached_skills;
            if !branch.trim().is_empty() {
                t.worktree_branch = branch;
            }
            t.column = column;
            if column == KanbanColumn::Done {
                t.progress_pct = 1.0;
                t.run_failed = false;
            }
            t.auto_schedule_idle = auto_schedule_idle;
        }
        self.save(project_root);
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
            t.custom_agent_name = custom_name.filter(|s| !s.trim().is_empty());
            let label = t.display_agent_name().to_string();
            t.live_output = format!("Đã gán cho {}", label);
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

    /// Moves a task forward one stage without wrapping Done back to Backlog.
    pub fn advance_task(&mut self, task_id: &str, project_root: &Path) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.column = match t.column {
                KanbanColumn::Backlog => KanbanColumn::InProgress,
                KanbanColumn::InProgress => KanbanColumn::Verifying,
                KanbanColumn::Verifying => KanbanColumn::Done,
                KanbanColumn::Done => KanbanColumn::Done,
            };
            if t.column == KanbanColumn::Done {
                t.progress_pct = 1.0;
                t.run_failed = false;
            }
        }
        self.save(project_root);
    }

    /// Drag-and-drop move to a target column and optional position before another task.
    pub fn move_task_to_column(
        &mut self,
        task_id: &str,
        target_col: KanbanColumn,
        insert_before_id: Option<&str>,
        project_root: &Path,
    ) {
        let Some(from_idx) = self.tasks.iter().position(|t| t.id == task_id) else {
            return;
        };
        if insert_before_id == Some(task_id) {
            return;
        }
        let mut card = self.tasks.remove(from_idx);
        card.column = target_col;
        if target_col == KanbanColumn::Done {
            card.progress_pct = 1.0;
            card.run_failed = false;
        } else if target_col == KanbanColumn::Backlog && !card.is_running_bg {
            card.run_failed = false;
        }

        if let Some(before_id) = insert_before_id {
            if let Some(pos) = self.tasks.iter().position(|t| t.id == before_id) {
                self.tasks.insert(pos, card);
            } else {
                self.tasks.push(card);
            }
        } else {
            self.tasks.push(card);
        }
        self.save(project_root);
    }

    pub fn approve_task(&mut self, task_id: &str, project_root: &Path) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.column = KanbanColumn::Done;
            t.progress_pct = 1.0;
            t.run_failed = false;
            t.last_error = None;
            let stamp = Local::now().format("%H:%M:%S");
            let line = format!("[{}] Đã duyệt hoàn tất.", stamp);
            t.live_output = line.clone();
            t.full_logs.push(line);
        }
        self.save(project_root);
    }

    pub fn reject_task(&mut self, task_id: &str, note: &str, project_root: &Path) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            t.column = KanbanColumn::Backlog;
            t.progress_pct = 0.0;
            t.run_failed = false;
            let clean_note = note.trim();
            if !clean_note.is_empty() {
                t.review_note = Some(clean_note.to_string());
                let stamp = Local::now().format("%H:%M:%S");
                let line = format!("[{}] Trả lại Backlog: {}", stamp, clean_note);
                t.live_output = line.clone();
                t.full_logs.push(line);
            } else {
                let stamp = Local::now().format("%H:%M:%S");
                let line = format!("[{}] Đã trả lại Backlog.", stamp);
                t.live_output = line.clone();
                t.full_logs.push(line);
            }
        }
        self.save(project_root);
    }

    pub fn duplicate_task(&mut self, task_id: &str, project_root: &Path) -> Option<String> {
        let orig = self.tasks.iter().find(|t| t.id == task_id)?.clone();
        let new_id = self.allocate_next_id();
        let copy = AgentTaskCard {
            id: new_id.clone(),
            title: format!("{} (bản sao)", orig.title),
            description: orig.description,
            priority: orig.priority,
            assigned_agent: orig.assigned_agent,
            custom_agent_name: orig.custom_agent_name,
            attached_skills: orig.attached_skills,
            worktree_branch: orig.worktree_branch,
            column: KanbanColumn::Backlog,
            progress_pct: 0.0,
            time_spent_secs: 0,
            live_output: "Đã nhân bản từ thẻ gốc — Sẵn sàng thực thi".to_string(),
            full_logs: Vec::new(),
            is_running_bg: false,
            run_failed: false,
            last_error: None,
            review_note: None,
            auto_schedule_idle: orig.auto_schedule_idle,
            created_at: Local::now().format("%Y-%m-%d %H:%M").to_string(),
        };
        self.tasks.push(copy);
        self.save(project_root);
        Some(new_id)
    }

    pub fn delete_task(&mut self, task_id: &str, project_root: &Path) -> Option<AgentTaskCard> {
        // Stop any running background process first
        self.stop_background_agent(task_id, project_root);
        if let Some(idx) = self.tasks.iter().position(|t| t.id == task_id) {
            let removed = self.tasks.remove(idx);
            self.save(project_root);
            return Some(removed);
        }
        None
    }

    pub fn restore_task(&mut self, card: AgentTaskCard, project_root: &Path) {
        if !self.tasks.iter().any(|t| t.id == card.id) {
            self.tasks.push(card);
            self.sync_next_task_seq();
            self.save(project_root);
        }
    }

    /// Stops a running background agent process immediately and moves card back to Backlog.
    pub fn stop_background_agent(&mut self, task_id: &str, project_root: &Path) {
        if let Some(cancel_tx) = self.cancel_handles.remove(task_id) {
            let _ = cancel_tx.send(());
        }
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            if t.is_running_bg {
                t.is_running_bg = false;
                t.column = KanbanColumn::Backlog;
                let stamp = Local::now().format("%H:%M:%S");
                let line = format!("[{}] Đã dừng tiến trình chạy nền.", stamp);
                t.live_output = line.clone();
                t.full_logs.push(line);
                self.save(project_root);
            }
        }
    }

    /// Spawns a real background `agy -p` worker for this Kanban card without blocking the main UI or chat.
    pub fn spawn_background_agent(
        &mut self,
        task_id: &str,
        workdir: PathBuf,
        custom_agent_prompt: Option<String>,
    ) {
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
        task.run_failed = false;
        task.last_error = None;
        task.column = KanbanColumn::InProgress;
        task.progress_pct = 0.08;
        let agent_label = task.display_agent_name().to_string();
        let stamp = Local::now().format("%H:%M:%S");
        let start_line = format!("[{}] Khởi chạy AGY ({})...", stamp, agent_label);
        task.live_output = start_line.clone();
        task.full_logs.push(start_line);

        let id_clone = task.id.clone();
        let skills_clause = if task.attached_skills.is_empty() {
            String::new()
        } else {
            let list: Vec<String> = task
                .attached_skills
                .iter()
                .map(|s| format!("${}", s))
                .collect();
            format!("\nSkills áp dụng: {}", list.join(" "))
        };
        let review_clause = task
            .review_note
            .as_deref()
            .map(|n| format!("\nGhi chú cần sửa từ lần kiểm tra trước: {}", n))
            .unwrap_or_default();
        let custom_sys = custom_agent_prompt
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| format!("\nChỉ dẫn chuyên biệt của Agent:\n{}\n", s))
            .unwrap_or_default();

        let prompt = format!(
            "[Swarm Background Task {} - Role: {} ({})]{}\nBranch: {}\nNhiệm vụ: {}\nChi tiết: {}{}{}\nHãy trả lời ngắn gọn các bước kiểm tra và kết quả thực hiện.",
            task.id,
            task.assigned_agent.title(),
            agent_label,
            custom_sys,
            task.worktree_branch,
            task.title,
            task.description,
            skills_clause,
            review_clause,
        );

        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
        self.cancel_handles.insert(id_clone.clone(), cancel_tx);

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
                    let stdout = child.stdout.take();
                    let stderr = child.stderr.take();

                    let tx_out = tx.clone();
                    let id_out = id_clone.clone();
                    let out_task = tokio::spawn(async move {
                        if let Some(out) = stdout {
                            let mut reader = BufReader::new(out).lines();
                            while let Ok(Some(line)) = reader.next_line().await {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    let _ = tx_out.send(BgAgentEvent::OutputLine {
                                        task_id: id_out.clone(),
                                        line: trimmed.to_string(),
                                        elapsed_secs: start.elapsed().as_secs(),
                                    });
                                }
                            }
                        }
                    });

                    let tx_err = tx.clone();
                    let id_err = id_clone.clone();
                    let err_task = tokio::spawn(async move {
                        let mut last_err = None;
                        if let Some(err) = stderr {
                            let mut reader = BufReader::new(err).lines();
                            while let Ok(Some(line)) = reader.next_line().await {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    last_err = Some(trimmed.to_string());
                                    let _ = tx_err.send(BgAgentEvent::OutputLine {
                                        task_id: id_err.clone(),
                                        line: format!("[stderr] {}", trimmed),
                                        elapsed_secs: start.elapsed().as_secs(),
                                    });
                                }
                            }
                        }
                        last_err
                    });

                    tokio::select! {
                        _ = &mut cancel_rx => {
                            let _ = child.kill().await;
                            out_task.abort();
                            err_task.abort();
                            let _ = tx.send(BgAgentEvent::Stopped {
                                task_id: id_clone,
                                elapsed_secs: start.elapsed().as_secs(),
                            });
                        }
                        status = child.wait() => {
                            let _ = out_task.await;
                            let last_stderr = err_task.await.ok().flatten();
                            let ok = status.as_ref().map(|s| s.success()).unwrap_or(false);
                            let err_msg = if ok {
                                None
                            } else {
                                Some(last_stderr.unwrap_or_else(|| {
                                    match status {
                                        Ok(s) => format!("Tiến trình thoát với mã {:?}", s.code()),
                                        Err(e) => format!("Lỗi tiến trình: {}", e),
                                    }
                                }))
                            };
                            let _ = tx.send(BgAgentEvent::Completed {
                                task_id: id_clone,
                                elapsed_secs: start.elapsed().as_secs(),
                                success: ok,
                                error: err_msg,
                            });
                        }
                    }
                }
                Err(e) => {
                    let msg = format!("Không thể khởi chạy AGY ({:?}): {}", agy_bin, e);
                    let _ = tx.send(BgAgentEvent::OutputLine {
                        task_id: id_clone.clone(),
                        line: msg.clone(),
                        elapsed_secs: start.elapsed().as_secs(),
                    });
                    let _ = tx.send(BgAgentEvent::Completed {
                        task_id: id_clone,
                        elapsed_secs: start.elapsed().as_secs(),
                        success: false,
                        error: Some(msg),
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
            let stamp = Local::now().format("%H:%M:%S");
            match ev {
                BgAgentEvent::Started { task_id } => {
                    if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        t.is_running_bg = true;
                        t.run_failed = false;
                        t.progress_pct = 0.15;
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
                        t.full_logs.push(format!("[{}] {}", stamp, line));
                        // Asymptotic progress while running so it never falsely hits 100%
                        t.progress_pct = (t.progress_pct + (0.92 - t.progress_pct) * 0.12).min(0.92);
                    }
                }
                BgAgentEvent::Stopped {
                    task_id,
                    elapsed_secs,
                } => {
                    self.cancel_handles.remove(&task_id);
                    if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        t.is_running_bg = false;
                        t.time_spent_secs = elapsed_secs.max(t.time_spent_secs);
                        t.column = KanbanColumn::Backlog;
                    }
                }
                BgAgentEvent::Completed {
                    task_id,
                    elapsed_secs,
                    success,
                    error,
                } => {
                    self.cancel_handles.remove(&task_id);
                    if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        t.is_running_bg = false;
                        t.time_spent_secs = elapsed_secs.max(t.time_spent_secs);
                        if success {
                            t.run_failed = false;
                            t.last_error = None;
                            t.progress_pct = 1.0;
                            t.column = KanbanColumn::Verifying;
                            let done_line = format!("[{}] Hoàn tất — Chờ nghiệm thu.", stamp);
                            t.full_logs.push(done_line);
                            if t.live_output.is_empty() {
                                t.live_output = "Hoàn tất — Chờ nghiệm thu".to_string();
                            }
                        } else {
                            t.run_failed = true;
                            t.last_error = error.clone();
                            t.progress_pct = t.progress_pct.min(0.85);
                            let err_txt = error.unwrap_or_else(|| "Thực thi thất bại".to_string());
                            t.live_output = format!("Lỗi: {}", err_txt);
                            t.full_logs.push(format!("[{}] Lỗi: {}", stamp, err_txt));
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

    pub fn running_tasks(&self) -> Vec<&AgentTaskCard> {
        self.tasks.iter().filter(|t| t.is_running_bg).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "victini_kanban_test_{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&p);
        p
    }

    #[test]
    fn task_ids_never_collide_after_deletion() {
        let root = temp_root();
        let mut mgr = AgentKanbanManager::default_workspace_board();
        let id5 = mgr.create_task(
            "Task 5",
            "",
            "P1",
            ElegansAgent::SeniorDev,
            None,
            vec![],
            "main",
            KanbanColumn::Backlog,
            true,
            false,
            &root,
        );
        assert_eq!(id5, "TASK-05");
        let deleted = mgr.delete_task("TASK-04", &root);
        assert!(deleted.is_some());
        let id6 = mgr.create_task(
            "Task 6",
            "",
            "P2",
            ElegansAgent::QaTester,
            None,
            vec![],
            "main",
            KanbanColumn::Backlog,
            false,
            false,
            &root,
        );
        assert_eq!(id6, "TASK-06");
        assert_ne!(id5, id6);
    }

    #[test]
    fn advance_task_does_not_wrap_done_to_backlog() {
        let root = temp_root();
        let mut mgr = AgentKanbanManager::default_workspace_board();
        mgr.advance_task("TASK-01", &root);
        let t = mgr.tasks.iter().find(|t| t.id == "TASK-01").unwrap();
        assert_eq!(t.column, KanbanColumn::Done);
    }

    #[test]
    fn move_approve_reject_and_restore_work() {
        let root = temp_root();
        let mut mgr = AgentKanbanManager::default_workspace_board();
        mgr.move_task_to_column("TASK-04", KanbanColumn::Verifying, None, &root);
        assert_eq!(
            mgr.tasks.iter().find(|t| t.id == "TASK-04").unwrap().column,
            KanbanColumn::Verifying
        );
        mgr.reject_task("TASK-04", "Thiếu unit test", &root);
        let t = mgr.tasks.iter().find(|t| t.id == "TASK-04").unwrap();
        assert_eq!(t.column, KanbanColumn::Backlog);
        assert_eq!(t.review_note.as_deref(), Some("Thiếu unit test"));

        mgr.approve_task("TASK-04", &root);
        assert_eq!(
            mgr.tasks.iter().find(|t| t.id == "TASK-04").unwrap().column,
            KanbanColumn::Done
        );

        let removed = mgr.delete_task("TASK-04", &root).unwrap();
        assert!(mgr.tasks.iter().all(|t| t.id != "TASK-04"));
        mgr.restore_task(removed, &root);
        assert!(mgr.tasks.iter().any(|t| t.id == "TASK-04"));
    }
}
