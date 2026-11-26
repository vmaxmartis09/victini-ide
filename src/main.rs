#![allow(dead_code)]

mod ai;
mod browser_gitlab;
mod config;
mod elegans;
mod emulator;
mod git;
mod theme;
mod ui;

use ai::{
    AgentAvatarState, AgentKanbanManager, AgentStreamEvent, AgyBridge, ChatSessionManager,
    ElegansAgent, ExecutionMode, KanbanColumn, MemoryReaper,
};
use browser_gitlab::BrowserProfileManager;
use chrono::Timelike;
use config::{DiscoveredAgent, DiscoveredSkill, ProjectManager, SkillDiscovery};
use eframe::egui;
use emulator::{AdbController, EmulatorState};
use git::{DiffHunk, GitFileChange, GitManager, WorktreeInfo};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc as std_mpsc, Arc};
use theme::{CanhTheme, SharedTicker};
use tokio::sync::mpsc;
use ui::{
    BeeBoxAction, CenterViewMode, ChatUiAction, HeaderAction, KanbanAction, KanbanComposerState,
    LayoutState, SidebarAction, StudioEditorState, StudioModalAction,
};

struct VictiniApp {
    theme: CanhTheme,
    ticker: SharedTicker,
    layout: LayoutState,
    project_mgr: ProjectManager,
    git_manager: GitManager,
    profile_manager: BrowserProfileManager,
    emu_state: EmulatorState,
    reaper: MemoryReaper,
    kanban: AgentKanbanManager,
    kanban_composer: KanbanComposerState,
    chat_mgr: ChatSessionManager,
    studio_editor: StudioEditorState,

    // Workspace & Git state
    branch_name: String,
    head_commit: String,
    branches: Vec<String>,
    worktrees: Vec<WorktreeInfo>,
    file_changes: Vec<GitFileChange>,
    active_diff: Option<DiffHunk>,
    discovered_skills: Vec<DiscoveredSkill>,
    discovered_agents: Vec<DiscoveredAgent>,

    // Sidebar & Composer inputs
    sidebar_search: String,
    new_branch_input: String,
    commit_message: String,
    input_prompt: String,
    show_mention_popup: bool,
    show_skill_popup: bool,

    // AI Streaming state
    active_agent: ElegansAgent,
    avatar_state: AgentAvatarState,
    exec_mode: ExecutionMode,
    cancel_flag: Arc<AtomicBool>,
    event_receiver: Option<std_mpsc::Receiver<AgentStreamEvent>>,
}

impl VictiniApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        theme::configure_be_vietnam_pro_fonts(&cc.egui_ctx);

        let theme = CanhTheme::dan_theme();
        Self::apply_theme_visuals(&cc.egui_ctx, &theme);

        let root = config::paths::detect_workspace_root();
        let project_mgr = ProjectManager::load_or_init(root.clone());
        let active_root = project_mgr.active_project.clone();

        let git_manager = GitManager::new(active_root.clone());
        let branch_name = git_manager.get_branch_name();
        let head_commit = git_manager.get_head_commit_short();
        let branches = git_manager.list_branches();
        let worktrees = git_manager.list_worktrees();
        let file_changes = git_manager.get_status();
        let (discovered_skills, discovered_agents) = SkillDiscovery::scan_workspace(&active_root);

        let profile_manager = BrowserProfileManager::new(9222);
        let emu_state = EmulatorState::default();
        let reaper = MemoryReaper::new(30);
        let kanban = AgentKanbanManager::load_for_project(&active_root);
        let chat_mgr = ChatSessionManager::load_for_project(&active_root, &branch_name);

        Self {
            theme,
            ticker: SharedTicker::new(),
            layout: LayoutState::default(),
            project_mgr,
            git_manager,
            profile_manager,
            emu_state,
            reaper,
            kanban,
            kanban_composer: KanbanComposerState::default(),
            chat_mgr,
            studio_editor: StudioEditorState::default(),
            branch_name,
            head_commit,
            branches,
            worktrees,
            file_changes,
            active_diff: None,
            discovered_skills,
            discovered_agents,
            sidebar_search: String::new(),
            new_branch_input: String::new(),
            commit_message: String::new(),
            input_prompt: String::new(),
            show_mention_popup: false,
            show_skill_popup: false,
            active_agent: ElegansAgent::SeniorDev,
            avatar_state: AgentAvatarState::Idle,
            exec_mode: ExecutionMode::Auto,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            event_receiver: None,
        }
    }

    fn apply_theme_visuals(ctx: &egui::Context, theme: &CanhTheme) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = theme.bg_field;
        visuals.window_fill = theme.panel_bg;
        visuals.extreme_bg_color = theme.bg_ink;
        ctx.set_visuals(visuals);
    }

    fn refresh_workspace(&mut self) {
        let root = self.project_mgr.active_project.clone();
        self.git_manager.set_repo_path(root.clone());
        self.branch_name = self.git_manager.get_branch_name();
        self.head_commit = self.git_manager.get_head_commit_short();
        self.branches = self.git_manager.list_branches();
        self.worktrees = self.git_manager.list_worktrees();
        self.file_changes = self.git_manager.get_status();
        self.project_mgr.update_branch(&root, &self.branch_name);
        let (skills, agents) = SkillDiscovery::scan_workspace(&root);
        self.discovered_skills = skills;
        self.discovered_agents = agents;
    }

    fn switch_to_project(&mut self, new_root: PathBuf) {
        self.project_mgr.active_project = new_root.clone();
        let _ = self.project_mgr.save();
        self.refresh_workspace();
        self.kanban = AgentKanbanManager::load_for_project(&new_root);
        self.chat_mgr = ChatSessionManager::load_for_project(&new_root, &self.branch_name);
        self.layout.center_mode = CenterViewMode::ChatTimeline;
    }

    fn dispatch_chat_prompt(&mut self) {
        let raw = self.input_prompt.trim().to_string();
        if raw.is_empty() {
            return;
        }
        self.input_prompt.clear();
        self.show_mention_popup = false;
        self.show_skill_popup = false;

        // Handle built-in slash commands cleanly
        if raw == "/clear" {
            self.chat_mgr.create_default_thread(&self.branch_name);
            self.chat_mgr.save(&self.project_mgr.active_project);
            return;
        } else if raw == "/kanban" {
            self.layout.center_mode = CenterViewMode::KanbanBoard;
            return;
        } else if raw == "/commit" {
            let _ = self.git_manager.stage_all();
            match self
                .git_manager
                .commit("feat(victini): autonomous slash commit")
            {
                Ok(oid) => self
                    .chat_mgr
                    .push_system_notice(format!("✓ Đã commit thành công ({})", &oid[..7.min(oid.len())])),
                Err(e) => self
                    .chat_mgr
                    .push_system_notice(format!("⚠️ Commit lỗi: {}", e)),
            }
            self.refresh_workspace();
            return;
        }

        let detected_agent = AgyBridge::detect_agent_for_prompt(&raw, self.active_agent);
        self.active_agent = detected_agent;
        let (_user_msg_id, quoted) = self.chat_mgr.add_user_message(&raw, detected_agent);
        let assistant_msg_id = self
            .chat_mgr
            .begin_assistant_stream(detected_agent, quoted.clone());
        self.chat_mgr.save(&self.project_mgr.active_project);

        let full_prompt = AgyBridge::build_contextual_prompt(&raw, &quoted, detected_agent);
        self.avatar_state = AgentAvatarState::Thinking;
        self.cancel_flag = Arc::new(AtomicBool::new(false));

        let (tokio_tx, mut tokio_rx) = mpsc::unbounded_channel();
        let (std_tx, std_rx) = std_mpsc::channel();
        self.event_receiver = Some(std_rx);

        tokio::spawn(async move {
            while let Some(evt) = tokio_rx.recv().await {
                let _ = std_tx.send(evt);
            }
        });

        AgyBridge::spawn_vibe_session(
            assistant_msg_id,
            full_prompt,
            detected_agent,
            self.project_mgr.active_project.clone(),
            self.cancel_flag.clone(),
            tokio_tx,
        );
    }
}

impl eframe::App for VictiniApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let _dt = self.ticker.tick();
        let elapsed_secs = self.ticker.elapsed_secs;

        // 1. Responsive Breakpoint Detection & Global Keyboard Shortcuts
        let win_width = ctx.screen_rect().width();
        let breakpoint = self.layout.compute_breakpoint(win_width);

        ctx.input(|i| {
            if i.modifiers.command && i.key_pressed(egui::Key::B) {
                self.layout.show_left_sidebar = !self.layout.show_left_sidebar;
            }
            if i.modifiers.command && i.key_pressed(egui::Key::E) {
                self.layout.show_right_dock = !self.layout.show_right_dock;
            }
            if i.modifiers.command && i.key_pressed(egui::Key::K) {
                self.layout.center_mode = match self.layout.center_mode {
                    CenterViewMode::KanbanBoard => CenterViewMode::ChatTimeline,
                    _ => CenterViewMode::KanbanBoard,
                };
            }
        });

        // 2. Auto Canh Giờ System Clock Theme
        if self.layout.auto_canh_gio {
            let hour = chrono::Local::now().hour();
            let next_theme = CanhTheme::for_hour(hour);
            if next_theme.id != self.theme.id {
                self.theme = next_theme;
                Self::apply_theme_visuals(ctx, &self.theme);
            }
        }

        // 3. Memory Reaper & Idle Background Auto-Scheduler
        if self.reaper.check_idle_and_reap() {
            if self.avatar_state == AgentAvatarState::Idle
                && !self.kanban.any_background_running()
            {
                self.avatar_state = AgentAvatarState::Offline;
                self.emu_state.clear_memory_cache();
            }

            // If Auto-Run Backlog on Idle is enabled, wake 1 backlog task in background
            if self.kanban.auto_run_backlog_on_idle && !self.kanban.any_background_running() {
                if let Some(next_id) = self
                    .kanban
                    .tasks
                    .iter()
                    .find(|t| t.column == KanbanColumn::Backlog && t.auto_schedule_idle)
                    .map(|t| t.id.clone())
                {
                    let root = self.project_mgr.active_project.clone();
                    self.kanban.spawn_background_agent(&next_id, root);
                }
            }
        }

        // 4. Poll Background Kanban Agents & Foreground Chat Stream
        let mut need_repaint = false;
        let project_root = self.project_mgr.active_project.clone();
        if self.kanban.poll_background_events(&project_root) {
            need_repaint = true;
        }

        let mut pending_events = Vec::new();
        if let Some(ref rx) = self.event_receiver {
            while let Ok(event) = rx.try_recv() {
                pending_events.push(event);
            }
        }

        for event in pending_events {
            need_repaint = true;
            self.reaper.mark_activity();
            match event {
                AgentStreamEvent::AgentActivated { agent, .. } => {
                    self.active_agent = agent;
                    self.avatar_state = AgentAvatarState::Thinking;
                }
                AgentStreamEvent::Line {
                    msg_id,
                    text,
                    elapsed_secs,
                } => {
                    self.avatar_state = AgentAvatarState::Answering;
                    self.chat_mgr.append_stream_line(&msg_id, &text, elapsed_secs);
                }
                AgentStreamEvent::Finished {
                    msg_id,
                    elapsed_secs,
                } => {
                    self.avatar_state = AgentAvatarState::Idle;
                    self.chat_mgr.finish_assistant_stream(&msg_id, elapsed_secs);
                    self.chat_mgr.save(&project_root);
                    self.refresh_workspace();
                }
            }
        }

        if need_repaint || self.ticker.should_repaint() || self.kanban.any_background_running() {
            ctx.request_repaint();
        }

        // 5. TOP HEADER BAR
        let running_bg_count = self
            .kanban
            .tasks
            .iter()
            .filter(|t| t.is_running_bg)
            .count();
        let proj_name = self.project_mgr.active_project_name();
        let mut header_action = None;

        egui::TopBottomPanel::top("victini_top_header")
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_ink)
                    .inner_margin(egui::Margin::symmetric(12.0, 7.0)),
            )
            .show(ctx, |ui| {
                header_action = ui::render_transparent_header(
                    ui,
                    &self.theme,
                    &proj_name,
                    &self.branch_name,
                    self.active_agent.title(),
                    running_bg_count,
                    &self.layout,
                    breakpoint,
                );
            });

        if let Some(act) = header_action {
            self.reaper.mark_activity();
            match act {
                HeaderAction::ToggleLeftSidebar => {
                    if self.layout.show_left_sidebar && !self.layout.left_mini_mode {
                        self.layout.left_mini_mode = true;
                    } else if self.layout.show_left_sidebar && self.layout.left_mini_mode {
                        self.layout.show_left_sidebar = false;
                    } else {
                        self.layout.show_left_sidebar = true;
                        self.layout.left_mini_mode = false;
                    }
                }
                HeaderAction::ToggleRightDock => {
                    self.layout.show_right_dock = !self.layout.show_right_dock;
                }
                HeaderAction::SetCenterMode(m) => {
                    self.layout.center_mode = m;
                }
                HeaderAction::OpenProjectsModal => {
                    self.layout.center_mode = CenterViewMode::ProjectsModal;
                }
                HeaderAction::LaunchGitLabBrowser => {
                    let mr_url = self.git_manager.get_gitlab_mr_url();
                    let _ = self.profile_manager.launch_or_connect(Some(&mr_url));
                }
                HeaderAction::RefreshWorkspace => {
                    self.refresh_workspace();
                }
                HeaderAction::CycleCanhTheme => {
                    let all = CanhTheme::all_themes();
                    let cur_idx = all
                        .iter()
                        .position(|t| t.id == self.theme.id)
                        .unwrap_or(0);
                    self.theme = all[(cur_idx + 1) % all.len()].clone();
                    Self::apply_theme_visuals(ctx, &self.theme);
                }
            }
        }

        // 6. LEFT SIDEBAR (Resizable or Mini-Icon Rail with Smooth Physics Width)
        if self.layout.show_left_sidebar {
            let target_w = if self.layout.left_mini_mode {
                54.0
            } else {
                self.layout.left_panel_width
            };
            let anim_w = self.ticker.animate_float("left_sidebar_w", target_w, 0.28);

            let thread_summaries: Vec<(usize, String, String)> = self
                .chat_mgr
                .threads
                .iter()
                .enumerate()
                .map(|(i, t)| (i, t.title.clone(), t.updated_at.clone()))
                .collect();

            let mut sidebar_action = None;
            let mut panel = egui::SidePanel::left("victini_qoder_sidebar")
                .frame(
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(0x09, 0x0A, 0x0E))
                        .inner_margin(egui::Margin::same(if self.layout.left_mini_mode {
                            6.0
                        } else {
                            10.0
                        })),
                );

            if self.layout.left_mini_mode {
                panel = panel.resizable(false).exact_width(anim_w);
            } else {
                panel = panel
                    .resizable(true)
                    .default_width(anim_w)
                    .width_range(190.0..=360.0);
            }

            panel.show(ctx, |ui| {
                sidebar_action = ui::render_qoder_sidebar(
                    ui,
                    &self.theme,
                    self.layout.left_mini_mode,
                    self.layout.center_mode,
                    &mut self.sidebar_search,
                    &self.worktrees,
                    &self.branches,
                    &mut self.new_branch_input,
                    &self.file_changes,
                    &mut self.commit_message,
                    &self.discovered_skills,
                    &self.discovered_agents,
                    &thread_summaries,
                    self.chat_mgr.active_thread_idx,
                );
            });

            if let Some(act) = sidebar_action {
                self.reaper.mark_activity();
                match act {
                    SidebarAction::ToggleMiniMode => {
                        self.layout.left_mini_mode = !self.layout.left_mini_mode;
                    }
                    SidebarAction::NewChatThread => {
                        self.chat_mgr.create_default_thread(&self.branch_name);
                        self.chat_mgr.save(&project_root);
                        self.layout.center_mode = CenterViewMode::ChatTimeline;
                    }
                    SidebarAction::SelectChatThread(idx) => {
                        if idx < self.chat_mgr.threads.len() {
                            self.chat_mgr.active_thread_idx = idx;
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                    }
                    SidebarAction::SwitchCenterView(mode) => {
                        self.layout.center_mode = mode;
                    }
                    SidebarAction::SelectWorktree(branch) => {
                        let _ = self.git_manager.switch_branch(&branch);
                        self.refresh_workspace();
                    }
                    SidebarAction::CreateNewBranch(b_name) => {
                        if let Err(e) = self.git_manager.create_and_checkout_branch(&b_name) {
                            self.chat_mgr
                                .push_system_notice(format!("⚠️ Tạo nhánh lỗi: {}", e));
                        } else {
                            self.chat_mgr
                                .push_system_notice(format!("✓ Đã tạo & chuyển sang nhánh '{}'", b_name));
                        }
                        self.new_branch_input.clear();
                        self.refresh_workspace();
                    }
                    SidebarAction::StageFile(p) => {
                        let _ = self.git_manager.stage_file(&p);
                        self.refresh_workspace();
                    }
                    SidebarAction::UnstageFile(p) => {
                        let _ = self.git_manager.unstage_file(&p);
                        self.refresh_workspace();
                    }
                    SidebarAction::DiscardFile(p) => {
                        let _ = self.git_manager.discard_file(&p);
                        self.refresh_workspace();
                    }
                    SidebarAction::StageAll => {
                        let _ = self.git_manager.stage_all();
                        self.refresh_workspace();
                    }
                    SidebarAction::ViewFileDiff(p) => {
                        self.active_diff = self.git_manager.compute_file_diff(&p).ok();
                        self.layout.center_mode = CenterViewMode::GitDiffModal;
                    }
                    SidebarAction::AiGenerateCommitMsg => {
                        let changed_list: Vec<String> = self
                            .file_changes
                            .iter()
                            .take(4)
                            .map(|f| {
                                f.path
                                    .rsplit('/')
                                    .next()
                                    .unwrap_or(&f.path)
                                    .to_string()
                            })
                            .collect();
                        if changed_list.is_empty() {
                            self.commit_message = "chore(victini): sync workspace".to_string();
                        } else {
                            self.commit_message = format!(
                                "feat({}): update {}",
                                self.branch_name,
                                changed_list.join(", ")
                            );
                        }
                    }
                    SidebarAction::CommitWithMessage(msg) => {
                        let _ = self.git_manager.stage_all();
                        match self.git_manager.commit(&msg) {
                            Ok(oid) => {
                                self.chat_mgr.push_system_notice(format!(
                                    "✓ Committed '{}' ({})",
                                    msg,
                                    &oid[..7.min(oid.len())]
                                ));
                                self.commit_message.clear();
                            }
                            Err(e) => {
                                self.chat_mgr
                                    .push_system_notice(format!("⚠️ Commit thất bại: {}", e));
                            }
                        }
                        self.refresh_workspace();
                    }
                    SidebarAction::GitPush => match self.git_manager.push_current_branch() {
                        Ok(res) => self.chat_mgr.push_system_notice(res),
                        Err(e) => self.chat_mgr.push_system_notice(format!("⚠️ {}", e)),
                    },
                    SidebarAction::GitPull => match self.git_manager.pull_rebase() {
                        Ok(res) => {
                            self.chat_mgr.push_system_notice(res);
                            self.refresh_workspace();
                        }
                        Err(e) => self.chat_mgr.push_system_notice(format!("⚠️ {}", e)),
                    },
                    SidebarAction::CreateGitLabMr => {
                        let _ = self.git_manager.push_current_branch();
                        let mr_url = self.git_manager.get_gitlab_mr_url();
                        let _ = self.profile_manager.launch_or_connect(Some(&mr_url));
                        self.chat_mgr.push_system_notice(format!(
                            "🦊 Đã mở trang tạo GitLab Merge Request: {}",
                            mr_url
                        ));
                    }
                    SidebarAction::OpenSkillStudioNewSkill => {
                        self.studio_editor.is_editing_agent = false;
                        self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                    }
                    SidebarAction::OpenSkillStudioNewAgent => {
                        self.studio_editor.is_editing_agent = true;
                        self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                    }
                    SidebarAction::InspectSkill(idx) => {
                        if let Some(sk) = self.discovered_skills.get(idx) {
                            self.studio_editor.is_editing_agent = false;
                            self.studio_editor.skill_name = sk.name.clone();
                            self.studio_editor.skill_desc = sk.description.clone();
                            self.studio_editor.skill_body = sk.content.clone();
                            self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                        }
                    }
                    SidebarAction::InspectAgent(idx) => {
                        if let Some(ag) = self.discovered_agents.get(idx) {
                            self.studio_editor.is_editing_agent = true;
                            self.studio_editor.agent_name = ag.name.clone();
                            self.studio_editor.agent_role = ag.role.clone();
                            self.studio_editor.agent_prompt = ag.content.clone();
                            self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                        }
                    }
                }
            }
        }

        // 7. RIGHT RAIL: T3 CODE BEEBOX & REMOTE EMULATOR DOCK (Resizable & Collapsible)
        if self.layout.show_right_dock {
            let mut beebox_action = None;
            egui::SidePanel::right("victini_beebox_dock")
                .resizable(true)
                .default_width(self.layout.right_panel_width)
                .width_range(220.0..=380.0)
                .frame(
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(0x0A, 0x0C, 0x11))
                        .inner_margin(egui::Margin::same(10.0)),
                )
                .show(ctx, |ui| {
                    beebox_action = ui::render_beebox_panel(
                        ui,
                        &self.theme,
                        elapsed_secs,
                        &proj_name,
                        &self.branch_name,
                        &self.head_commit,
                        &self.file_changes,
                        &self.kanban.tasks,
                        &mut self.emu_state,
                    );
                });

            if let Some(act) = beebox_action {
                self.reaper.mark_activity();
                match act {
                    BeeBoxAction::OpenProjectsModal => {
                        self.layout.center_mode = CenterViewMode::ProjectsModal;
                    }
                    BeeBoxAction::SetupWorktree => {
                        let wt_branch = format!("feat/vibe-{}", self.worktrees.len() + 1);
                        let new_wt_path = project_root.join(format!("../victini-wt-{}", self.worktrees.len() + 1));
                        match self.git_manager.setup_new_worktree(&wt_branch, &new_wt_path) {
                            Ok(_) => self.chat_mgr.push_system_notice(format!(
                                "🔧 Đã khởi tạo Worktree mới '{}' tại {:?}",
                                wt_branch, new_wt_path
                            )),
                            Err(e) => self.chat_mgr.push_system_notice(format!(
                                "⚠️ Không thể tạo Worktree: {}",
                                e
                            )),
                        }
                        self.refresh_workspace();
                    }
                    BeeBoxAction::OpenGitLabMr => {
                        let mr_url = self.git_manager.get_gitlab_mr_url();
                        let _ = self.profile_manager.launch_or_connect(Some(&mr_url));
                    }
                    BeeBoxAction::TriggerTap(x, y) => {
                        let _ = AdbController::dispatch_tap(x, y);
                    }
                    BeeBoxAction::SwitchPreset(_idx) => {}
                    BeeBoxAction::CaptureAdbFrame => {
                        self.emu_state.refresh_adb_screenshot();
                    }
                    BeeBoxAction::RunBackgroundTask(task_id) => {
                        self.kanban
                            .spawn_background_agent(&task_id, project_root.clone());
                    }
                }
            }
        }

        // 8. CENTER WORKSPACE AREA (Responsive Chat / Kanban / Split / Diff / Projects / Studio / Settings)
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_field)
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                let mut chat_action = None;
                let mut kanban_action = None;
                let mut modal_action = None;

                match self.layout.center_mode {
                    CenterViewMode::ChatTimeline => {
                        chat_action = ui::render_qoder_chat_timeline(
                            ui,
                            &self.theme,
                            elapsed_secs,
                            self.active_agent,
                            self.avatar_state,
                            self.exec_mode,
                            &self.chat_mgr,
                            &self.file_changes,
                            &self.discovered_skills,
                            &mut self.input_prompt,
                            &mut self.show_mention_popup,
                            &mut self.show_skill_popup,
                        );
                    }
                    CenterViewMode::KanbanBoard => {
                        kanban_action = ui::render_agent_kanban_view(
                            ui,
                            &self.theme,
                            elapsed_secs,
                            &mut self.kanban,
                            &mut self.kanban_composer,
                        );
                    }
                    CenterViewMode::SplitChatKanban => {
                        ui.columns(2, |cols| {
                            chat_action = ui::render_qoder_chat_timeline(
                                &mut cols[0],
                                &self.theme,
                                elapsed_secs,
                                self.active_agent,
                                self.avatar_state,
                                self.exec_mode,
                                &self.chat_mgr,
                                &self.file_changes,
                                &self.discovered_skills,
                                &mut self.input_prompt,
                                &mut self.show_mention_popup,
                                &mut self.show_skill_popup,
                            );
                            kanban_action = ui::render_agent_kanban_view(
                                &mut cols[1],
                                &self.theme,
                                elapsed_secs,
                                &mut self.kanban,
                                &mut self.kanban_composer,
                            );
                        });
                    }
                    CenterViewMode::GitDiffModal => {
                        modal_action =
                            ui::render_git_diff_viewer(ui, &self.theme, &self.active_diff);
                    }
                    CenterViewMode::ProjectsModal => {
                        modal_action = ui::render_projects_manager(
                            ui,
                            &self.theme,
                            &self.project_mgr,
                            &mut self.studio_editor,
                        );
                    }
                    CenterViewMode::AgentSkillStudio => {
                        modal_action = ui::render_agent_skill_studio(
                            ui,
                            &self.theme,
                            elapsed_secs,
                            &self.discovered_skills,
                            &self.discovered_agents,
                            &mut self.studio_editor,
                        );
                    }
                    CenterViewMode::SettingsModal => {
                        modal_action = ui::render_settings_modal(
                            ui,
                            &self.theme,
                            &mut self.layout,
                            &mut self.reaper,
                            &self.ticker,
                        );
                    }
                }

                // Handle Chat UI Actions
                if let Some(c_act) = chat_action {
                    self.reaper.mark_activity();
                    match c_act {
                        ChatUiAction::SendPrompt => {
                            self.dispatch_chat_prompt();
                        }
                        ChatUiAction::StopStream => {
                            self.cancel_flag.store(true, Ordering::Relaxed);
                            self.avatar_state = AgentAvatarState::Idle;
                        }
                        ChatUiAction::SetExecutionMode(m) => {
                            self.exec_mode = m;
                        }
                        ChatUiAction::SelectActiveAgent(ag) => {
                            self.active_agent = ag;
                        }
                        ChatUiAction::SetQuote(q) => {
                            self.chat_mgr.active_quote = Some(q);
                        }
                        ChatUiAction::ClearQuote => {
                            self.chat_mgr.active_quote = None;
                        }
                        ChatUiAction::SendMessageToKanban {
                            title,
                            detail,
                            agent,
                        } => {
                            let id = self.kanban.create_task(
                                &title,
                                &detail,
                                "P1",
                                agent,
                                None,
                                &self.branch_name,
                                false,
                                &project_root,
                            );
                            self.chat_mgr.push_system_notice(format!(
                                "📊 Đã tạo thẻ {} trên Swarm Kanban cho {}",
                                id,
                                agent.name()
                            ));
                        }
                    }
                }

                // Handle Kanban UI Actions
                if let Some(k_act) = kanban_action {
                    self.reaper.mark_activity();
                    match k_act {
                        KanbanAction::CreateTask {
                            title,
                            description,
                            priority,
                            agent,
                            run_now,
                        } => {
                            self.kanban.create_task(
                                &title,
                                &description,
                                &priority,
                                agent,
                                None,
                                &self.branch_name,
                                run_now,
                                &project_root,
                            );
                        }
                        KanbanAction::ReassignAgent {
                            task_id,
                            next_agent,
                        } => {
                            self.kanban
                                .assign_agent(&task_id, next_agent, None, &project_root);
                        }
                        KanbanAction::MoveLeft(id) => {
                            self.kanban.move_task_left(&id, &project_root);
                        }
                        KanbanAction::AdvanceTask(id) => {
                            self.kanban.advance_task(&id, &project_root);
                        }
                        KanbanAction::RunInBackground(id) => {
                            self.kanban
                                .spawn_background_agent(&id, project_root.clone());
                        }
                        KanbanAction::QuoteTaskToChat(q) => {
                            self.chat_mgr.active_quote = Some(q);
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                        KanbanAction::DeleteTask(id) => {
                            self.kanban.delete_task(&id, &project_root);
                        }
                    }
                }

                // Handle Studio & Modal Actions
                if let Some(m_act) = modal_action {
                    self.reaper.mark_activity();
                    match m_act {
                        StudioModalAction::CloseToChat => {
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                        StudioModalAction::SwitchProject(p) => {
                            self.switch_to_project(p);
                        }
                        StudioModalAction::AddNewProject(path_str) => {
                            if let Ok(p) = self.project_mgr.add_or_switch_project(&path_str) {
                                self.switch_to_project(p);
                            }
                        }
                        StudioModalAction::TogglePinProject(p) => {
                            self.project_mgr.toggle_pin(&p);
                        }
                        StudioModalAction::QuoteDiffToChat(q) => {
                            self.chat_mgr.active_quote = Some(q);
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                        StudioModalAction::SaveSkill { name, desc, body } => {
                            let _ = SkillDiscovery::create_or_update_skill(
                                &project_root,
                                &name,
                                &desc,
                                &body,
                            );
                            self.refresh_workspace();
                        }
                        StudioModalAction::SaveAgent { name, role, prompt } => {
                            let _ = SkillDiscovery::create_or_update_agent(
                                &project_root,
                                &name,
                                &role,
                                &prompt,
                            );
                            self.refresh_workspace();
                        }
                        StudioModalAction::DeleteSkill(p) => {
                            let _ = SkillDiscovery::delete_skill(&p);
                            self.refresh_workspace();
                        }
                        StudioModalAction::DeleteAgent(p) => {
                            let _ = SkillDiscovery::delete_agent(&p);
                            self.refresh_workspace();
                        }
                        StudioModalAction::InsertSkillIntoChat(token) => {
                            self.input_prompt.push_str(&token);
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                        StudioModalAction::SelectTheme(t) => {
                            self.theme = t;
                            Self::apply_theme_visuals(ctx, &self.theme);
                        }
                    }
                }
            });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 800.0])
            .with_min_inner_size([640.0, 460.0])
            .with_title("⚡ VICTINI IDE — Autonomous Vibe-Code & Swarm Kanban Station"),
        ..Default::default()
    };

    eframe::run_native(
        "Victini IDE",
        native_options,
        Box::new(|cc| Ok(Box::new(VictiniApp::new(cc)))),
    )
}
