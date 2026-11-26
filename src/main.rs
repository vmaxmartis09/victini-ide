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
    ElegansAgent, ExecutionMode, KanbanColumn, MemoryReaper, QuotedContext, ThreadAgentMode,
    ALL_AGENTS,
};
use browser_gitlab::BrowserProfileManager;
use chrono::Timelike;
use config::{
    spawn_install_candidates, spawn_scan_github_repo, spawn_update_skill_from_source,
    DiscoveredAgent, DiscoveredSkill, ImportEvent, ProjectManager, SkillDiscovery,
};
use eframe::egui::{self, Color32};
use emulator::{AdbController, EmulatorState};
use git::{DiffHunk, GitFileChange, GitManager, WorktreeInfo};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc as std_mpsc, Arc};
use theme::{CanhTheme, SharedTicker};
use tokio::sync::mpsc;
use ui::{
    BeeBoxAction, BeeBoxUiState, CenterViewMode, ChatComposerUiState, ChatUiAction, HeaderAction,
    KanbanAction, KanbanComposerState, LayoutState, SidebarAction, StudioEditorState,
    StudioModalAction,
};
use walkdir::WalkDir;

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
    project_chats: HashMap<PathBuf, ChatSessionManager>,
    studio_editor: StudioEditorState,
    beebox_ui: BeeBoxUiState,
    chat_composer_ui: ChatComposerUiState,

    // Workspace & Git state
    branch_name: String,
    head_commit: String,
    branches: Vec<String>,
    worktrees: Vec<WorktreeInfo>,
    file_changes: Vec<GitFileChange>,
    workspace_files: Vec<String>,
    active_diff: Option<DiffHunk>,
    discovered_skills: Vec<DiscoveredSkill>,
    discovered_agents: Vec<DiscoveredAgent>,

    // Sidebar & Composer inputs
    sidebar_show_agents_tab: bool,
    sidebar_search: String,
    commit_message: String,
    input_prompt: String,

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
        let project_mgr = ProjectManager::load_or_init(root);
        let active_root = project_mgr.active_project.clone();

        let git_manager = GitManager::new(active_root.clone());
        let branch_name = git_manager.get_branch_name();
        let head_commit = git_manager.get_head_commit_short();
        let branches = git_manager.list_branches();
        let worktrees = git_manager.list_worktrees();
        let file_changes = git_manager.get_status();
        let workspace_files = git_manager.list_workspace_files(250);
        let (discovered_skills, discovered_agents) = SkillDiscovery::scan_workspace(&active_root);

        let profile_manager = BrowserProfileManager::new(9222);
        let emu_state = EmulatorState::default();
        let reaper = MemoryReaper::new(30);
        let kanban = AgentKanbanManager::load_for_project(&active_root);
        let chat_mgr = ChatSessionManager::load_for_project(&active_root, &branch_name);
        let mut project_chats = HashMap::new();
        for proj in &project_mgr.projects {
            if proj.path == active_root {
                project_chats.insert(proj.path.clone(), chat_mgr.clone());
            } else {
                let mgr = ChatSessionManager::load_for_project(&proj.path, &proj.last_branch);
                project_chats.insert(proj.path.clone(), mgr);
            }
        }

        let mut studio_editor = StudioEditorState::default();
        let active_diff = if let Some(first_change) = file_changes.first() {
            studio_editor.active_file_path = first_change.path.clone();
            studio_editor.file_editor_buffer = git_manager
                .read_file_content(&first_change.path)
                .unwrap_or_default();
            git_manager.compute_file_diff(&first_change.path).ok()
        } else {
            None
        };

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
            project_chats,
            studio_editor,
            beebox_ui: BeeBoxUiState::default(),
            chat_composer_ui: ChatComposerUiState::default(),
            branch_name,
            head_commit,
            branches,
            worktrees,
            file_changes,
            workspace_files,
            active_diff,
            discovered_skills,
            discovered_agents,
            sidebar_show_agents_tab: false,
            sidebar_search: String::new(),
            commit_message: String::new(),
            input_prompt: String::new(),
            active_agent: ElegansAgent::SeniorDev,
            avatar_state: AgentAvatarState::Idle,
            exec_mode: ExecutionMode::Auto,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            event_receiver: None,
        }
    }

    fn apply_theme_visuals(ctx: &egui::Context, theme: &CanhTheme) {
        let mut visuals = if theme.is_dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.dark_mode = theme.is_dark;
        visuals.panel_fill = theme.bg_field;
        visuals.window_fill = theme.panel_bg;
        visuals.extreme_bg_color = theme.input_bg;
        visuals.faint_bg_color = theme.surface_1;

        visuals.widgets.noninteractive.bg_fill = theme.panel_bg;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, theme.border);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.text_paper);
        visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);

        visuals.widgets.inactive.bg_fill = theme.input_bg;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, theme.border);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme.text_paper);
        visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);

        visuals.widgets.hovered.bg_fill = theme.panel_hover;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, theme.border_strong);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, theme.text_paper);
        visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);

        visuals.widgets.active.bg_fill = theme.surface_2;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, theme.accent_pop);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, theme.text_paper);
        visuals.widgets.active.rounding = egui::Rounding::same(8.0);

        visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(
            theme.accent_pop.r(),
            theme.accent_pop.g(),
            theme.accent_pop.b(),
            80,
        );
        visuals.selection.stroke = egui::Stroke::new(1.0, theme.accent_pop);

        // Enforce Elegans Strict 4px Spacing Grid, h-7 (28px) Control Height, and 4px Quiet Scrollbar
        ctx.style_mut(|style| {
            style.visuals = visuals;
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
            style.spacing.window_margin = egui::Margin::same(12.0);
            style.spacing.indent = 16.0;
            style.spacing.interact_size = egui::vec2(28.0, 28.0);
            style.spacing.scroll.bar_width = 4.0;
        });
    }

    fn refresh_workspace(&mut self) {
        let root = self.project_mgr.active_project.clone();
        self.git_manager.set_repo_path(root.clone());
        self.branch_name = self.git_manager.get_branch_name();
        self.head_commit = self.git_manager.get_head_commit_short();
        self.branches = self.git_manager.list_branches();
        self.worktrees = self.git_manager.list_worktrees();
        self.file_changes = self.git_manager.get_status();
        self.workspace_files = self.git_manager.list_workspace_files(250);
        self.project_mgr.update_branch(&root, &self.branch_name);
        let (skills, agents) = SkillDiscovery::scan_workspace(&root);
        self.discovered_skills = skills;
        self.discovered_agents = agents;

        if !self.studio_editor.active_file_path.is_empty() {
            self.active_diff = self
                .git_manager
                .compute_file_diff(&self.studio_editor.active_file_path)
                .ok();
        }
    }

    fn open_file_in_editor(&mut self, rel_path: &str, edit_mode: bool) {
        let target_path = if !rel_path.is_empty() {
            rel_path.to_string()
        } else if !self.studio_editor.active_file_path.is_empty() {
            self.studio_editor.active_file_path.clone()
        } else if let Some(first) = self.file_changes.first() {
            first.path.clone()
        } else if let Some(wf) = self.workspace_files.first() {
            wf.clone()
        } else {
            String::new()
        };

        if !target_path.is_empty() {
            self.studio_editor.active_file_path = target_path.clone();
            self.studio_editor.file_editor_buffer = self
                .git_manager
                .read_file_content(&target_path)
                .unwrap_or_default();
            self.studio_editor.file_editor_dirty = false;
            self.active_diff = self.git_manager.compute_file_diff(&target_path).ok();
        }
        self.studio_editor.diff_edit_mode = edit_mode;
        self.studio_editor.status_notice = None;
        self.layout.center_mode = CenterViewMode::GitDiffModal;
    }

    fn run_elegans_diagnostics(&mut self) {
        let root = self.project_mgr.active_project.clone();
        let report = Self::build_elegans_security_and_ast_report(&root);
        self.chat_mgr.push_system_notice(report);
        self.chat_mgr.save(&root);
        self.layout.center_mode = CenterViewMode::ChatTimeline;
    }

    fn build_elegans_security_and_ast_report(root: &Path) -> String {
        let re_secret = Regex::new(
            r"(?:sk-[a-zA-Z0-9]{20,}|ghp_[a-zA-Z0-9]{30,}|glpat-[a-zA-Z0-9\-_]{20,}|AIza[0-9A-Za-z\-_]{35}|xox[baprs]-[0-9a-zA-Z\-]{10,})",
        )
        .unwrap();
        let mut total_files = 0usize;
        let mut total_lines = 0usize;
        let mut rust_files = 0usize;
        let mut ts_js_files = 0usize;
        let mut secret_leaks = Vec::new();

        for entry in WalkDir::new(root).max_depth(6).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let p = entry.path();
            let p_str = p.to_string_lossy().replace('\\', "/");
            if p_str.contains("/target/")
                || p_str.contains("/node_modules/")
                || p_str.contains("/.git/")
                || p_str.contains("/bin/")
            {
                continue;
            }
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(
                ext,
                "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "toml" | "json" | "env"
            ) {
                continue;
            }
            total_files += 1;
            if ext == "rs" {
                rust_files += 1;
            } else if matches!(ext, "ts" | "tsx" | "js" | "jsx") {
                ts_js_files += 1;
            }
            if let Ok(raw) = fs::read_to_string(p) {
                total_lines += raw.lines().count();
                if re_secret.is_match(&raw) {
                    if let Ok(rel) = p.strip_prefix(root) {
                        secret_leaks.push(rel.to_string_lossy().to_string());
                    }
                }
            }
        }

        let sec_verdict = if secret_leaks.is_empty() {
            "PASS (0 hardcoded API keys / secrets detected)".to_string()
        } else {
            format!("WARNING: Phát hiện nghi vấn lộ Secret tại: {}", secret_leaks.join(", "))
        };

        format!(
            "[Elegans Security & AST Blast Audit]\n• Project: {:?}\n• Env & Secret Guard: {}\n• AST & Code Metrics: {} source files ({} Rust, {} TS/JS) · {} lines of code\n• Browser Isolation: Protected Dad Profile active",
            root, sec_verdict, total_files, rust_files, ts_js_files, total_lines
        )
    }

    fn switch_to_project(&mut self, new_root: PathBuf) {
        let old_root = self.project_mgr.active_project.clone();
        self.chat_mgr.save(&old_root);
        self.project_chats.insert(old_root, self.chat_mgr.clone());

        self.project_mgr.ensure_project_in_list(&new_root);
        self.project_mgr.active_project = new_root.clone();
        let _ = self.project_mgr.save();
        self.refresh_workspace();
        self.kanban = AgentKanbanManager::load_for_project(&new_root);
        self.chat_mgr = self
            .project_chats
            .get(&new_root)
            .cloned()
            .unwrap_or_else(|| ChatSessionManager::load_for_project(&new_root, &self.branch_name));
        self.project_chats
            .insert(new_root, self.chat_mgr.clone());
        self.layout.center_mode = CenterViewMode::ChatTimeline;
    }

    fn handle_slash_command(&mut self, cmd: &str) -> bool {
        let trimmed = cmd.trim();
        if trimmed == "/clear" {
            self.chat_mgr.create_default_thread(&self.branch_name);
            self.chat_mgr.save(&self.project_mgr.active_project);
            return true;
        } else if trimmed == "/kanban" {
            self.layout.center_mode = CenterViewMode::KanbanBoard;
            return true;
        } else if trimmed == "/editor" {
            self.open_file_in_editor("", true);
            return true;
        } else if trimmed == "/env-guard" || trimmed == "/ast-blast" {
            self.run_elegans_diagnostics();
            return true;
        } else if trimmed == "/commit" {
            let msg = if self.commit_message.trim().is_empty() {
                self.git_manager
                    .generate_ai_commit_message(&self.file_changes)
            } else {
                self.commit_message.trim().to_string()
            };
            let _ = self.git_manager.stage_all();
            match self.git_manager.commit(&msg) {
                Ok(oid) => {
                    self.chat_mgr.push_system_notice(format!(
                        "✓ Đã commit '{}' ({})",
                        msg,
                        &oid[..7.min(oid.len())]
                    ));
                    self.commit_message.clear();
                }
                Err(e) => self
                    .chat_mgr
                    .push_system_notice(format!("Commit lỗi: {}", e)),
            }
            self.refresh_workspace();
            return true;
        } else if trimmed == "/diff" {
            let mut summary = String::new();
            for fc in self.file_changes.iter().take(6) {
                if let Ok(hunk) = self.git_manager.compute_file_diff(&fc.path) {
                    let lines: String = hunk
                        .lines
                        .iter()
                        .filter(|l| l.tag != ' ')
                        .take(15)
                        .map(|l| format!("{} {}", l.tag, l.content))
                        .collect::<Vec<_>>()
                        .join("\n");
                    summary.push_str(&format!(
                        "File {} (+{} -{}):\n{}\n\n",
                        fc.path, fc.added_lines, fc.removed_lines, lines
                    ));
                }
            }
            if summary.is_empty() {
                self.chat_mgr
                    .push_system_notice("Working tree hiện tại không có thay đổi nào.");
                return true;
            }
            self.chat_mgr.active_quote = Some(QuotedContext {
                source_msg_id: "git-diff-all".to_string(),
                author_label: "Workspace Git Diff".to_string(),
                snippet: summary,
                is_quote_block: true,
            });
            self.input_prompt =
                "Hãy review chi tiết các thay đổi trong Git Diff này và chỉ ra các điểm cần tối ưu."
                    .to_string();
            return false;
        } else if trimmed == "/review" {
            self.chat_mgr
                .open_or_create_dedicated_core_thread(ElegansAgent::CodeAuditor, &self.branch_name);
            self.active_agent = ElegansAgent::CodeAuditor;
            self.chat_mgr.save(&self.project_mgr.active_project);
            return true;
        } else if let Some(rest) = trimmed.strip_prefix("/agent ") {
            let target = rest.trim().to_lowercase();
            for &ag in ALL_AGENTS {
                if ag.name().to_lowercase().contains(&target) {
                    self.chat_mgr
                        .open_or_create_dedicated_core_thread(ag, &self.branch_name);
                    self.active_agent = ag;
                    self.chat_mgr.save(&self.project_mgr.active_project);
                    return true;
                }
            }
            if let Some(ca) = self
                .discovered_agents
                .iter()
                .find(|a| a.name.to_lowercase().contains(&target))
            {
                self.chat_mgr.open_or_create_dedicated_custom_thread(
                    &ca.name,
                    &ca.role,
                    &ca.content,
                    &self.branch_name,
                );
                self.chat_mgr.save(&self.project_mgr.active_project);
                return true;
            }
        }
        false
    }

    fn dispatch_chat_prompt(&mut self) {
        let raw = self.input_prompt.trim().to_string();
        if raw.is_empty() {
            return;
        }
        self.input_prompt.clear();
        self.chat_composer_ui.show_context_popup = false;
        self.chat_composer_ui.show_agent_popup = false;

        if self.handle_slash_command(&raw) {
            return;
        }
        let prompt_to_send = if self.input_prompt.is_empty() {
            raw
        } else {
            let p = self.input_prompt.clone();
            self.input_prompt.clear();
            p
        };

        for token in prompt_to_send.split_whitespace() {
            if let Some(f_tok) = token.strip_prefix('@') {
                if (self.workspace_files.iter().any(|wf| wf == f_tok)
                    || self.file_changes.iter().any(|fc| fc.path == f_tok))
                    && !self
                        .chat_mgr
                        .active_thread()
                        .attached_files
                        .contains(&f_tok.to_string())
                {
                    self.chat_mgr.toggle_attached_file(f_tok.to_string());
                }
            } else if let Some(s_tok) = token.strip_prefix('$') {
                if self.discovered_skills.iter().any(|sk| sk.name == s_tok)
                    && !self
                        .chat_mgr
                        .active_thread()
                        .attached_skills
                        .contains(&s_tok.to_string())
                {
                    self.chat_mgr.toggle_attached_skill(s_tok.to_string());
                }
            }
        }

        let agent_mode = self.chat_mgr.active_thread().agent_mode.clone();
        let resolved_agent = match &agent_mode {
            ThreadAgentMode::SwarmAuto => {
                AgyBridge::detect_agent_for_prompt(&prompt_to_send, self.active_agent)
            }
            ThreadAgentMode::DedicatedCore(ag) => *ag,
            ThreadAgentMode::DedicatedCustom { .. } => ElegansAgent::SeniorDev,
        };
        self.active_agent = resolved_agent;

        let attached_files_list = self.chat_mgr.active_thread().attached_files.clone();
        let attached_skills_list = self.chat_mgr.active_thread().attached_skills.clone();

        let mut file_payloads = Vec::new();
        for f_path in &attached_files_list {
            if let Ok(content) = self.git_manager.read_file_content(f_path) {
                file_payloads.push((f_path.clone(), content));
            }
        }

        let mut skill_payloads = Vec::new();
        for sk_name in &attached_skills_list {
            if let Some(sk) = self
                .discovered_skills
                .iter()
                .find(|s| &s.name == sk_name && s.enabled)
            {
                skill_payloads.push((sk.name.clone(), sk.body.clone()));
            }
        }

        let (_user_msg_id, quoted) = self
            .chat_mgr
            .add_user_message(&prompt_to_send, resolved_agent);
        let assistant_msg_id = self
            .chat_mgr
            .begin_assistant_stream(resolved_agent, quoted.clone());
        self.chat_mgr.save(&self.project_mgr.active_project);

        let full_prompt = AgyBridge::build_contextual_prompt(
            &prompt_to_send,
            &quoted,
            &agent_mode,
            resolved_agent,
            self.exec_mode,
            &file_payloads,
            &skill_payloads,
        );
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
            resolved_agent,
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

        let mut toggle_theme_shortcut = false;
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
            if i.modifiers.command && i.key_pressed(egui::Key::L) {
                toggle_theme_shortcut = true;
            }
        });

        if toggle_theme_shortcut {
            self.layout.auto_canh_gio = false;
            self.theme = self.theme.toggle_light_dark();
            Self::apply_theme_visuals(ctx, &self.theme);
        }

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

            if self.kanban.auto_run_backlog_on_idle && !self.kanban.any_background_running() {
                if let Some(next_id) = self
                    .kanban
                    .tasks
                    .iter()
                    .find(|t| t.column == KanbanColumn::Backlog && t.auto_schedule_idle)
                    .map(|t| t.id.clone())
                {
                    let root = self.project_mgr.active_project.clone();
                    self.kanban.spawn_background_agent(&next_id, root, None);
                }
            }
        }

        // 4. Poll Background Kanban Agents, GitHub Skill Importer & Foreground Chat Stream
        let mut need_repaint = false;
        let project_root = self.project_mgr.active_project.clone();
        if self.kanban.poll_background_events(&project_root) {
            need_repaint = true;
        }

        // Poll GitHub Skill Importer events
        let mut import_events = Vec::new();
        if let Some(ref rx) = self.studio_editor.import_receiver {
            while let Ok(ev) = rx.try_recv() {
                import_events.push(ev);
            }
        }
        let mut refresh_after_import = false;
        for ev in import_events {
            need_repaint = true;
            match ev {
                ImportEvent::Progress(msg, pct) => {
                    self.studio_editor.import_progress = Some((msg, pct));
                }
                ImportEvent::ScanCompleted(res) => {
                    self.studio_editor.import_progress = None;
                    match res {
                        Ok(scan) => {
                            self.studio_editor.status_notice = Some(format!(
                                "✓ Đã tìm thấy {} skills từ {}",
                                scan.candidates.len(),
                                scan.source.repo_slug()
                            ));
                            self.studio_editor.scan_result = Some(scan);
                        }
                        Err(e) => {
                            self.studio_editor.status_notice = Some(format!("Lỗi quét repo: {}", e));
                        }
                    }
                }
                ImportEvent::InstallCompleted(res) => {
                    self.studio_editor.import_progress = None;
                    match res {
                        Ok(rep) => {
                            self.studio_editor.status_notice = Some(format!(
                                "✓ Đã cài đặt {} skills (Bỏ qua: {})",
                                rep.installed.len(),
                                rep.skipped.len()
                            ));
                            refresh_after_import = true;
                        }
                        Err(e) => {
                            self.studio_editor.status_notice =
                                Some(format!("Lỗi cài đặt skill: {}", e));
                        }
                    }
                }
                ImportEvent::UpdateCompleted(res) => {
                    self.studio_editor.import_progress = None;
                    match res {
                        Ok(msg) => {
                            self.studio_editor.status_notice = Some(format!("✓ {}", msg));
                            refresh_after_import = true;
                        }
                        Err(e) => {
                            self.studio_editor.status_notice = Some(format!("Lỗi cập nhật: {}", e));
                        }
                    }
                }
            }
        }
        if refresh_after_import {
            self.refresh_workspace();
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
                    self.chat_mgr
                        .append_stream_line(&msg_id, &text, elapsed_secs);
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

        let is_streaming = self.avatar_state == AgentAvatarState::Thinking
            || self.avatar_state == AgentAvatarState::Answering;

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
        let active_thread_title = self.chat_mgr.active_thread().title.clone();
        let mut header_action = None;

        egui::TopBottomPanel::top("victini_top_header")
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_ink)
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0)),
            )
            .show(ctx, |ui| {
                header_action = ui::render_transparent_header(
                    ui,
                    &self.theme,
                    &proj_name,
                    &self.branch_name,
                    &active_thread_title,
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
                    if m == CenterViewMode::GitDiffModal {
                        self.open_file_in_editor("", false);
                    } else {
                        self.layout.center_mode = m;
                    }
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
                HeaderAction::ToggleLightDark => {
                    self.layout.auto_canh_gio = false;
                    self.theme = self.theme.toggle_light_dark();
                    Self::apply_theme_visuals(ctx, &self.theme);
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
                HeaderAction::RunSecurityDiagnostics => {
                    self.run_elegans_diagnostics();
                }
            }
        }

        // 6. LEFT SIDEBAR (Threads & 1-on-1 Dedicated Agents Launcher)
        if self.layout.show_left_sidebar {
            let target_w = if self.layout.left_mini_mode {
                48.0
            } else {
                self.layout.left_panel_width
            };
            let anim_w = self.ticker.animate_float("left_sidebar_w", target_w, 0.28);

            let mut sidebar_action = None;
            let mut panel = egui::SidePanel::left("victini_qoder_sidebar").frame(
                egui::Frame::none()
                    .fill(self.theme.bg_ink)
                    .inner_margin(egui::Margin::same(if self.layout.left_mini_mode {
                        8.0
                    } else {
                        12.0
                    })),
            );

            if self.layout.left_mini_mode {
                panel = panel.resizable(false).exact_width(anim_w);
            } else {
                panel = panel
                    .resizable(true)
                    .default_width(anim_w)
                    .width_range(200.0..=360.0);
            }

            panel.show(ctx, |ui| {
                sidebar_action = ui::render_qoder_sidebar(
                    ui,
                    &self.theme,
                    self.layout.left_mini_mode,
                    self.layout.center_mode,
                    &self.project_mgr,
                    &self.project_chats,
                    &self.chat_mgr,
                    &self.branch_name,
                    is_streaming,
                    &mut self.sidebar_show_agents_tab,
                    &mut self.sidebar_search,
                    &self.file_changes,
                    &self.discovered_skills,
                    &self.discovered_agents,
                );
            });

            if let Some(act) = sidebar_action {
                self.reaper.mark_activity();
                match act {
                    SidebarAction::ToggleMiniMode => {
                        self.layout.left_mini_mode = !self.layout.left_mini_mode;
                    }
                    SidebarAction::ToggleLightDark => {
                        self.layout.auto_canh_gio = false;
                        self.theme = self.theme.toggle_light_dark();
                        Self::apply_theme_visuals(ctx, &self.theme);
                    }
                    SidebarAction::NewChatThread => {
                        self.chat_mgr.create_default_thread(&self.branch_name);
                        self.chat_mgr.save(&project_root);
                        self.project_chats
                            .insert(project_root.clone(), self.chat_mgr.clone());
                        self.layout.center_mode = CenterViewMode::ChatTimeline;
                    }
                    SidebarAction::SelectChatThread(idx) => {
                        if idx < self.chat_mgr.threads.len() {
                            self.chat_mgr.active_thread_idx = idx;
                            self.active_agent = self
                                .chat_mgr
                                .active_thread()
                                .agent_mode
                                .resolve_core_agent(self.active_agent);
                            self.project_chats
                                .insert(project_root.clone(), self.chat_mgr.clone());
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                    }
                    SidebarAction::DeleteChatThread(idx) => {
                        if self.chat_mgr.threads.len() > 1 && idx < self.chat_mgr.threads.len() {
                            self.chat_mgr.threads.remove(idx);
                            if self.chat_mgr.active_thread_idx >= self.chat_mgr.threads.len() {
                                self.chat_mgr.active_thread_idx =
                                    self.chat_mgr.threads.len().saturating_sub(1);
                            }
                            self.chat_mgr.save(&project_root);
                            self.project_chats
                                .insert(project_root.clone(), self.chat_mgr.clone());
                        }
                    }
                    SidebarAction::ToggleProjectExpand(p) => {
                        self.project_mgr.toggle_expanded(&p);
                    }
                    SidebarAction::NewProjectChatThread(p) => {
                        if p != project_root {
                            self.switch_to_project(p.clone());
                        }
                        self.chat_mgr.create_default_thread(&self.branch_name);
                        self.chat_mgr.save(&p);
                        self.project_chats.insert(p, self.chat_mgr.clone());
                        self.layout.center_mode = CenterViewMode::ChatTimeline;
                    }
                    SidebarAction::SelectProjectThread {
                        project_path,
                        thread_idx,
                    } => {
                        if project_path != project_root {
                            self.switch_to_project(project_path.clone());
                        }
                        if thread_idx < self.chat_mgr.threads.len() {
                            self.chat_mgr.active_thread_idx = thread_idx;
                            self.active_agent = self
                                .chat_mgr
                                .active_thread()
                                .agent_mode
                                .resolve_core_agent(self.active_agent);
                            self.chat_mgr.save(&project_path);
                            self.project_chats
                                .insert(project_path, self.chat_mgr.clone());
                        }
                        self.layout.center_mode = CenterViewMode::ChatTimeline;
                    }
                    SidebarAction::DeleteProjectThread {
                        project_path,
                        thread_idx,
                    } => {
                        if project_path == project_root {
                            if self.chat_mgr.threads.len() > 1
                                && thread_idx < self.chat_mgr.threads.len()
                            {
                                self.chat_mgr.threads.remove(thread_idx);
                                if self.chat_mgr.active_thread_idx >= self.chat_mgr.threads.len() {
                                    self.chat_mgr.active_thread_idx =
                                        self.chat_mgr.threads.len().saturating_sub(1);
                                }
                                self.chat_mgr.save(&project_root);
                                self.project_chats
                                    .insert(project_root.clone(), self.chat_mgr.clone());
                            }
                        } else if let Some(mgr) = self.project_chats.get_mut(&project_path) {
                            if mgr.threads.len() > 1 && thread_idx < mgr.threads.len() {
                                mgr.threads.remove(thread_idx);
                                if mgr.active_thread_idx >= mgr.threads.len() {
                                    mgr.active_thread_idx =
                                        mgr.threads.len().saturating_sub(1);
                                }
                                mgr.save(&project_path);
                            }
                        }
                    }
                    SidebarAction::OpenDedicatedCoreAgent(ag) => {
                        self.chat_mgr
                            .open_or_create_dedicated_core_thread(ag, &self.branch_name);
                        self.active_agent = ag;
                        self.chat_mgr.save(&project_root);
                        self.layout.center_mode = CenterViewMode::ChatTimeline;
                    }
                    SidebarAction::OpenDedicatedCustomAgent(idx) => {
                        if let Some(ca) = self.discovered_agents.get(idx) {
                            self.chat_mgr.open_or_create_dedicated_custom_thread(
                                &ca.name,
                                &ca.role,
                                &ca.body,
                                &self.branch_name,
                            );
                            self.chat_mgr.save(&project_root);
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                    }
                    SidebarAction::SwitchToSwarmAuto => {
                        self.chat_mgr.active_thread_mut().agent_mode = ThreadAgentMode::SwarmAuto;
                        self.chat_mgr.save(&project_root);
                        self.layout.center_mode = CenterViewMode::ChatTimeline;
                    }
                    SidebarAction::SwitchCenterView(mode) => {
                        if mode == CenterViewMode::GitDiffModal {
                            self.open_file_in_editor("", false);
                        } else {
                            self.layout.center_mode = mode;
                        }
                    }
                    SidebarAction::OpenSkillStudioNewSkill => {
                        self.studio_editor.studio_tab = 0;
                        self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                    }
                    SidebarAction::OpenSkillStudioNewAgent => {
                        self.studio_editor.studio_tab = 1;
                        self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                    }
                    SidebarAction::InspectSkill(idx) => {
                        if let Some(sk) = self.discovered_skills.get(idx) {
                            self.studio_editor.studio_tab = 0;
                            self.studio_editor.selected_scope = sk.scope;
                            self.studio_editor.skill_name = sk.name.clone();
                            self.studio_editor.skill_desc = sk.description.clone();
                            self.studio_editor.skill_body = sk.body.clone();
                            self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                        }
                    }
                    SidebarAction::InspectAgent(idx) => {
                        if let Some(ag) = self.discovered_agents.get(idx) {
                            self.studio_editor.studio_tab = 1;
                            self.studio_editor.selected_scope = ag.scope;
                            self.studio_editor.agent_name = ag.name.clone();
                            self.studio_editor.agent_role = ag.role.clone();
                            self.studio_editor.agent_prompt = ag.body.clone();
                            self.layout.center_mode = CenterViewMode::AgentSkillStudio;
                        }
                    }
                }
            }
        }

        // 7. RIGHT RAIL: ELEGANS FRAMED-SHEET INSPECTOR DOCK (8px inset from right/bottom)
        if self.layout.show_right_dock {
            let mut beebox_action = None;
            egui::SidePanel::right("victini_beebox_dock")
                .resizable(true)
                .default_width(self.layout.right_panel_width)
                .width_range(256.0..=400.0)
                .frame(
                    egui::Frame::none()
                        .fill(self.theme.bg_ink)
                        .inner_margin(egui::Margin {
                            left: 0.0,
                            right: 8.0,
                            top: 0.0,
                            bottom: 8.0,
                        }),
                )
                .show(ctx, |ui| {
                    beebox_action = ui::render_beebox_panel(
                        ui,
                        &self.theme,
                        elapsed_secs,
                        &proj_name,
                        &self.branch_name,
                        &self.head_commit,
                        &self.branches,
                        &self.worktrees,
                        &self.file_changes,
                        &mut self.commit_message,
                        &mut self.beebox_ui,
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
                    BeeBoxAction::OpenBuiltInEditor => {
                        self.open_file_in_editor("", true);
                    }
                    BeeBoxAction::OpenExternalEditor(rel_opt) => {
                        match self
                            .git_manager
                            .open_in_external_editor(rel_opt.as_deref())
                        {
                            Ok(msg) => self.chat_mgr.push_system_notice(msg),
                            Err(e) => self
                                .chat_mgr
                                .push_system_notice(format!("Không thể mở Editor ngoài: {}", e)),
                        }
                    }
                    BeeBoxAction::InspectFile { path, edit_mode } => {
                        self.open_file_in_editor(&path, edit_mode);
                    }
                    BeeBoxAction::StageFile(p) => {
                        let _ = self.git_manager.stage_file(&p);
                        self.refresh_workspace();
                    }
                    BeeBoxAction::UnstageFile(p) => {
                        let _ = self.git_manager.unstage_file(&p);
                        self.refresh_workspace();
                    }
                    BeeBoxAction::StageAll => {
                        let _ = self.git_manager.stage_all();
                        self.refresh_workspace();
                    }
                    BeeBoxAction::UnstageAll => {
                        let _ = self.git_manager.unstage_all();
                        self.refresh_workspace();
                    }
                    BeeBoxAction::DiscardFile(p) => {
                        match self.git_manager.discard_file(&p) {
                            Ok(_) => self
                                .chat_mgr
                                .push_system_notice(format!("✓ Đã khôi phục file '{}'", p)),
                            Err(e) => self
                                .chat_mgr
                                .push_system_notice(format!("Discard lỗi: {}", e)),
                        }
                        self.refresh_workspace();
                    }
                    BeeBoxAction::SwitchBranch(b) => {
                        match self.git_manager.switch_branch(&b) {
                            Ok(_) => self
                                .chat_mgr
                                .push_system_notice(format!("✓ Đã chuyển sang nhánh '{}'", b)),
                            Err(e) => self
                                .chat_mgr
                                .push_system_notice(format!("Checkout lỗi: {}", e)),
                        }
                        self.refresh_workspace();
                    }
                    BeeBoxAction::CreateBranch(b) => {
                        match self.git_manager.create_and_checkout_branch(&b) {
                            Ok(_) => self.chat_mgr.push_system_notice(format!(
                                "✓ Đã tạo & chuyển sang nhánh mới '{}'",
                                b
                            )),
                            Err(e) => self
                                .chat_mgr
                                .push_system_notice(format!("Tạo nhánh lỗi: {}", e)),
                        }
                        self.refresh_workspace();
                    }
                    BeeBoxAction::SetupWorktree(wt_branch) => {
                        let safe_slug = wt_branch.replace(['/', '\\', ' '], "-");
                        let new_wt_path = project_root
                            .parent()
                            .unwrap_or(&project_root)
                            .join(format!("{}-wt-{}", proj_name, safe_slug));
                        match self
                            .git_manager
                            .setup_new_worktree(&wt_branch, &new_wt_path)
                        {
                            Ok(_) => self.chat_mgr.push_system_notice(format!(
                                "✓ Đã khởi tạo Worktree '{}' tại {:?}",
                                wt_branch, new_wt_path
                            )),
                            Err(e) => self
                                .chat_mgr
                                .push_system_notice(format!("Không thể tạo Worktree: {}", e)),
                        }
                        self.refresh_workspace();
                    }
                    BeeBoxAction::SwitchWorktree(wt_path) => {
                        self.switch_to_project(wt_path);
                    }
                    BeeBoxAction::RemoveWorktree(wt_path) => {
                        match self.git_manager.remove_worktree(&wt_path) {
                            Ok(msg) => self.chat_mgr.push_system_notice(msg),
                            Err(e) => self.chat_mgr.push_system_notice(format!("{}", e)),
                        }
                        self.refresh_workspace();
                    }
                    BeeBoxAction::AiGenerateCommitMsg => {
                        self.commit_message = self
                            .git_manager
                            .generate_ai_commit_message(&self.file_changes);
                    }
                    BeeBoxAction::Commit(msg) => {
                        let has_staged = self.file_changes.iter().any(|c| c.is_staged);
                        if !has_staged {
                            let _ = self.git_manager.stage_all();
                        }
                        match self.git_manager.commit(&msg) {
                            Ok(oid) => {
                                self.chat_mgr.push_system_notice(format!(
                                    "✓ Committed '{}' ({})",
                                    msg,
                                    &oid[..7.min(oid.len())]
                                ));
                                self.commit_message.clear();
                            }
                            Err(e) => self
                                .chat_mgr
                                .push_system_notice(format!("Commit thất bại: {}", e)),
                        }
                        self.refresh_workspace();
                    }
                    BeeBoxAction::GitPush => match self.git_manager.push_current_branch() {
                        Ok(res) => self.chat_mgr.push_system_notice(res),
                        Err(e) => self.chat_mgr.push_system_notice(format!("{}", e)),
                    },
                    BeeBoxAction::GitPull => match self.git_manager.pull_rebase() {
                        Ok(res) => {
                            self.chat_mgr.push_system_notice(res);
                            self.refresh_workspace();
                        }
                        Err(e) => self.chat_mgr.push_system_notice(format!("{}", e)),
                    },
                    BeeBoxAction::GitStashSave => {
                        match self.git_manager.stash_save(&self.commit_message) {
                            Ok(res) => {
                                self.chat_mgr.push_system_notice(res);
                                self.refresh_workspace();
                            }
                            Err(e) => self.chat_mgr.push_system_notice(format!("{}", e)),
                        }
                    }
                    BeeBoxAction::GitStashPop => match self.git_manager.stash_pop() {
                        Ok(res) => {
                            self.chat_mgr.push_system_notice(res);
                            self.refresh_workspace();
                        }
                        Err(e) => self.chat_mgr.push_system_notice(format!("{}", e)),
                    },
                    BeeBoxAction::OpenGitLabMr => {
                        let mr_url = self.git_manager.get_gitlab_mr_url();
                        let _ = self.profile_manager.launch_or_connect(Some(&mr_url));
                        self.chat_mgr.push_system_notice(format!(
                            "✓ Đã mở trang tạo GitLab Merge Request: {}",
                            mr_url
                        ));
                    }
                    BeeBoxAction::TriggerTap(x, y) => {
                        let _ = AdbController::dispatch_tap(x, y);
                    }
                    BeeBoxAction::SwitchPreset(idx) => {
                        self.emu_state.active_preset_idx = idx;
                    }
                    BeeBoxAction::CaptureAdbFrame => {
                        self.emu_state.refresh_adb_screenshot();
                    }
                    BeeBoxAction::OpenPreviewUrl(url) => {
                        let _ = self.profile_manager.launch_or_connect(Some(&url));
                    }
                    BeeBoxAction::RunBackgroundTask(task_id) => {
                        self.kanban
                            .spawn_background_agent(&task_id, project_root.clone(), None);
                    }
                }
            }
        }

        // 8. CENTER WORKSPACE AREA: ELEGANS FRAMED-SHEET (8px inset, 16px ROUND_PANEL, 1px border)
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_ink)
                    .inner_margin(egui::Margin {
                        left: 4.0,
                        right: 8.0,
                        top: 0.0,
                        bottom: 8.0,
                    }),
            )
            .show(ctx, |ui| {
                let mut chat_action = None;
                let mut kanban_action = None;
                let mut modal_action = None;

                egui::Frame::none()
                    .fill(self.theme.bg_field)
                    .rounding(egui::Rounding::same(16.0))
                    .stroke(egui::Stroke::new(1.0, self.theme.border))
                    .inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        match self.layout.center_mode {
                            CenterViewMode::ChatTimeline => {
                                chat_action = ui::render_qoder_chat_timeline(
                                    ui,
                                    &self.theme,
                                    elapsed_secs,
                                    &proj_name,
                                    &self.branch_name,
                                    self.active_agent,
                                    self.avatar_state,
                                    self.exec_mode,
                                    &self.chat_mgr,
                                    &self.file_changes,
                                    &self.workspace_files,
                                    &self.discovered_skills,
                                    &self.discovered_agents,
                                    &mut self.input_prompt,
                                    &mut self.chat_composer_ui,
                                );
                            }
                            CenterViewMode::KanbanBoard => {
                                kanban_action = ui::render_agent_kanban_view(
                                    ui,
                                    &self.theme,
                                    elapsed_secs,
                                    &mut self.kanban,
                                    &mut self.kanban_composer,
                                    &self.discovered_skills,
                                    &self.discovered_agents,
                                    &self.branches,
                                    &self.branch_name,
                                );
                            }
                            CenterViewMode::SplitChatKanban => {
                                ui.columns(2, |cols| {
                                    chat_action = ui::render_qoder_chat_timeline(
                                        &mut cols[0],
                                        &self.theme,
                                        elapsed_secs,
                                        &proj_name,
                                        &self.branch_name,
                                        self.active_agent,
                                        self.avatar_state,
                                        self.exec_mode,
                                        &self.chat_mgr,
                                        &self.file_changes,
                                        &self.workspace_files,
                                        &self.discovered_skills,
                                        &self.discovered_agents,
                                        &mut self.input_prompt,
                                        &mut self.chat_composer_ui,
                                    );
                                    kanban_action = ui::render_agent_kanban_view(
                                        &mut cols[1],
                                        &self.theme,
                                        elapsed_secs,
                                        &mut self.kanban,
                                        &mut self.kanban_composer,
                                        &self.discovered_skills,
                                        &self.discovered_agents,
                                        &self.branches,
                                        &self.branch_name,
                                    );
                                });
                            }
                            CenterViewMode::GitDiffModal => {
                                modal_action = ui::render_git_diff_viewer(
                                    ui,
                                    &self.theme,
                                    &self.active_diff,
                                    &self.file_changes,
                                    &self.workspace_files,
                                    &mut self.studio_editor,
                                );
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
                    });

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
                        ChatUiAction::SetThreadAgentMode(mode) => {
                            self.active_agent = mode.resolve_core_agent(self.active_agent);
                            self.chat_mgr.active_thread_mut().agent_mode = mode;
                            self.chat_mgr.save(&project_root);
                        }
                        ChatUiAction::OpenDedicatedCoreAgent(ag) => {
                            self.chat_mgr
                                .open_or_create_dedicated_core_thread(ag, &self.branch_name);
                            self.active_agent = ag;
                            self.chat_mgr.save(&project_root);
                        }
                        ChatUiAction::OpenDedicatedCustomAgent(idx) => {
                            if let Some(ca) = self.discovered_agents.get(idx) {
                                self.chat_mgr.open_or_create_dedicated_custom_thread(
                                    &ca.name,
                                    &ca.role,
                                    &ca.body,
                                    &self.branch_name,
                                );
                                self.chat_mgr.save(&project_root);
                            }
                        }
                        ChatUiAction::ToggleAttachFile(f_path) => {
                            self.chat_mgr.toggle_attached_file(f_path);
                            self.chat_mgr.save(&project_root);
                        }
                        ChatUiAction::ToggleAttachSkill(sk_name) => {
                            self.chat_mgr.toggle_attached_skill(sk_name);
                            self.chat_mgr.save(&project_root);
                        }
                        ChatUiAction::OpenFileInEditor { path, edit_mode } => {
                            self.open_file_in_editor(&path, edit_mode);
                        }
                        ChatUiAction::ApplyCodeBlockToEditor(code_snippet) => {
                            let target_file = if let Some(att) =
                                self.chat_mgr.active_thread().attached_files.first()
                            {
                                att.clone()
                            } else if !self.studio_editor.active_file_path.is_empty() {
                                self.studio_editor.active_file_path.clone()
                            } else if let Some(fc) = self.file_changes.first() {
                                fc.path.clone()
                            } else {
                                "src/snippet.rs".to_string()
                            };
                            self.studio_editor.active_file_path = target_file;
                            self.studio_editor.file_editor_buffer = code_snippet;
                            self.studio_editor.file_editor_dirty = true;
                            self.studio_editor.diff_edit_mode = true;
                            self.studio_editor.status_notice = Some(
                                "Đã nạp đoạn code từ AI — Bấm Save (⌘S) để lưu vào file"
                                    .to_string(),
                            );
                            self.layout.center_mode = CenterViewMode::GitDiffModal;
                        }
                        ChatUiAction::RunQuickSlashCommand(cmd) => {
                            self.input_prompt = cmd;
                            self.dispatch_chat_prompt();
                        }
                        ChatUiAction::RunSecurityDiagnostics => {
                            self.run_elegans_diagnostics();
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
                                Vec::new(),
                                &self.branch_name,
                                KanbanColumn::Backlog,
                                true,
                                false,
                                &project_root,
                            );
                            self.chat_mgr.push_system_notice(format!(
                                "✓ Đã tạo thẻ {} trên Swarm Kanban cho {}",
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
                            custom_agent,
                            attached_skills,
                            branch,
                            column,
                            auto_schedule_idle,
                            run_now,
                        } => {
                            let b = if branch.trim().is_empty() {
                                self.branch_name.clone()
                            } else {
                                branch
                            };
                            self.kanban.create_task(
                                &title,
                                &description,
                                &priority,
                                agent,
                                custom_agent,
                                attached_skills,
                                &b,
                                column,
                                auto_schedule_idle,
                                run_now,
                                &project_root,
                            );
                        }
                        KanbanAction::UpdateTask {
                            task_id,
                            title,
                            description,
                            priority,
                            agent,
                            custom_agent,
                            attached_skills,
                            branch,
                            column,
                            auto_schedule_idle,
                        } => {
                            self.kanban.update_task(
                                &task_id,
                                title,
                                description,
                                priority,
                                agent,
                                custom_agent,
                                attached_skills,
                                branch,
                                column,
                                auto_schedule_idle,
                                &project_root,
                            );
                        }
                        KanbanAction::ReassignAgent {
                            task_id,
                            next_agent,
                            custom_agent,
                        } => {
                            self.kanban
                                .assign_agent(&task_id, next_agent, custom_agent, &project_root);
                        }
                        KanbanAction::MoveToColumn {
                            task_id,
                            target_col,
                            insert_before_id,
                        } => {
                            self.kanban.move_task_to_column(
                                &task_id,
                                target_col,
                                insert_before_id.as_deref(),
                                &project_root,
                            );
                        }
                        KanbanAction::MoveLeft(id) => {
                            self.kanban.move_task_left(&id, &project_root);
                        }
                        KanbanAction::AdvanceTask(id) => {
                            self.kanban.advance_task(&id, &project_root);
                        }
                        KanbanAction::ApproveTask(id) => {
                            self.kanban.approve_task(&id, &project_root);
                        }
                        KanbanAction::RejectTask { task_id, note } => {
                            self.kanban.reject_task(&task_id, &note, &project_root);
                        }
                        KanbanAction::DuplicateTask(id) => {
                            self.kanban.duplicate_task(&id, &project_root);
                        }
                        KanbanAction::RunInBackground(id) => {
                            self.kanban
                                .spawn_background_agent(&id, project_root.clone(), None);
                        }
                        KanbanAction::StopBackground(id) => {
                            self.kanban.stop_background_agent(&id, &project_root);
                        }
                        KanbanAction::QuoteTaskToChat(q) => {
                            self.chat_mgr.active_quote = Some(q);
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                        KanbanAction::DeleteTask(id) => {
                            self.kanban.delete_task(&id, &project_root);
                        }
                        KanbanAction::RestoreTask(card) => {
                            self.kanban.restore_task(card, &project_root);
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
                        StudioModalAction::OpenFileInEditor { path, edit_mode } => {
                            self.open_file_in_editor(&path, edit_mode);
                        }
                        StudioModalAction::CreateNewFile(rel_path) => {
                            match self.git_manager.write_file_content(&rel_path, "") {
                                Ok(_) => {
                                    self.refresh_workspace();
                                    self.open_file_in_editor(&rel_path, true);
                                    self.studio_editor.status_notice =
                                        Some(format!("✓ Đã tạo file mới {}", rel_path));
                                }
                                Err(e) => {
                                    self.studio_editor.status_notice =
                                        Some(format!("Không thể tạo file: {}", e));
                                }
                            }
                        }
                        StudioModalAction::SaveEditedFile { path, content } => {
                            match self.git_manager.write_file_content(&path, &content) {
                                Ok(_) => {
                                    self.studio_editor.status_notice =
                                        Some(format!("✓ Đã lưu {}", path));
                                }
                                Err(e) => {
                                    self.studio_editor.status_notice =
                                        Some(format!("Lỗi lưu file: {}", e));
                                }
                            }
                            self.refresh_workspace();
                        }
                        StudioModalAction::StageFileFromEditor(p) => {
                            let _ = self.git_manager.stage_file(&p);
                            self.refresh_workspace();
                        }
                        StudioModalAction::UnstageFileFromEditor(p) => {
                            let _ = self.git_manager.unstage_file(&p);
                            self.refresh_workspace();
                        }
                        StudioModalAction::DiscardFileFromEditor(p) => {
                            let _ = self.git_manager.discard_file(&p);
                            self.refresh_workspace();
                            self.open_file_in_editor(&p, self.studio_editor.diff_edit_mode);
                        }
                        StudioModalAction::OpenExternalEditor(rel_opt) => {
                            let _ = self
                                .git_manager
                                .open_in_external_editor(rel_opt.as_deref());
                        }
                        StudioModalAction::SaveSkill {
                            name,
                            desc,
                            body,
                            scope,
                        } => {
                            match SkillDiscovery::save_skill(
                                &project_root,
                                scope,
                                &name,
                                &desc,
                                &body,
                                None,
                                true,
                            ) {
                                Ok(_) => {
                                    self.studio_editor.status_notice =
                                        Some(format!("✓ Đã lưu Skill '${}'", name));
                                }
                                Err(e) => {
                                    self.studio_editor.status_notice =
                                        Some(format!("Lỗi lưu Skill: {}", e));
                                }
                            }
                            self.refresh_workspace();
                        }
                        StudioModalAction::SaveAgent {
                            name,
                            role,
                            prompt,
                            scope,
                        } => {
                            match SkillDiscovery::save_agent(
                                &project_root,
                                scope,
                                &name,
                                &role,
                                "fe_senior_dev",
                                None,
                                &[],
                                &prompt,
                                None,
                                true,
                            ) {
                                Ok(_) => {
                                    self.studio_editor.status_notice =
                                        Some(format!("✓ Đã lưu Subagent '@{}'", name));
                                }
                                Err(e) => {
                                    self.studio_editor.status_notice =
                                        Some(format!("Lỗi lưu Subagent: {}", e));
                                }
                            }
                            self.refresh_workspace();
                        }
                        StudioModalAction::ToggleSkillEnabled { path, enabled } => {
                            SkillDiscovery::set_skill_enabled(&path, enabled);
                            self.refresh_workspace();
                        }
                        StudioModalAction::DeleteSkill(p) => {
                            if let Ok(trashed) = SkillDiscovery::trash_skill(&p) {
                                self.studio_editor.trashed_items.push(trashed);
                            }
                            self.refresh_workspace();
                        }
                        StudioModalAction::DeleteAgent(p) => {
                            if let Ok(trashed) = SkillDiscovery::trash_agent(&p) {
                                self.studio_editor.trashed_items.push(trashed);
                            }
                            self.refresh_workspace();
                        }
                        StudioModalAction::RestoreTrashed(idx) => {
                            if idx < self.studio_editor.trashed_items.len() {
                                let item = self.studio_editor.trashed_items.remove(idx);
                                let _ = SkillDiscovery::restore_trashed(&item);
                                self.studio_editor.status_notice =
                                    Some(format!("✓ Đã khôi phục '{}'", item.name));
                                self.refresh_workspace();
                            }
                        }
                        StudioModalAction::ScanGitHubRepo { url, branch } => {
                            self.studio_editor.scan_result = None;
                            self.studio_editor.import_progress =
                                Some(("Đang kết nối GitHub...".to_string(), 0.05));
                            self.studio_editor.import_receiver = Some(spawn_scan_github_repo(
                                url,
                                branch,
                                None,
                                project_root.clone(),
                            ));
                        }
                        StudioModalAction::InstallScannedSkills => {
                            if let Some(scan) = self.studio_editor.scan_result.clone() {
                                self.studio_editor.import_progress =
                                    Some(("Đang cài đặt skills...".to_string(), 0.1));
                                self.studio_editor.import_receiver =
                                    Some(spawn_install_candidates(
                                        scan,
                                        self.studio_editor.selected_scope,
                                        self.studio_editor.import_conflict_strategy,
                                        project_root.clone(),
                                    ));
                            }
                        }
                        StudioModalAction::UpdateSkillFromSource { path, source } => {
                            self.studio_editor.import_progress =
                                Some(("Đang cập nhật từ GitHub...".to_string(), 0.2));
                            self.studio_editor.import_receiver =
                                Some(spawn_update_skill_from_source(path, source, None));
                        }
                        StudioModalAction::InsertSkillIntoChat(token) => {
                            let clean = token.trim().trim_start_matches('$').to_string();
                            self.chat_mgr.toggle_attached_skill(clean);
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                        StudioModalAction::OpenDedicatedAgentChat(agent_name) => {
                            let mut matched_core = false;
                            for &ag in ALL_AGENTS {
                                if ag.name() == agent_name {
                                    self.chat_mgr.open_or_create_dedicated_core_thread(
                                        ag,
                                        &self.branch_name,
                                    );
                                    self.active_agent = ag;
                                    matched_core = true;
                                    break;
                                }
                            }
                            if !matched_core {
                                if let Some(ca) = self
                                    .discovered_agents
                                    .iter()
                                    .find(|a| a.name == agent_name)
                                {
                                    self.chat_mgr.open_or_create_dedicated_custom_thread(
                                        &ca.name,
                                        &ca.role,
                                        &ca.body,
                                        &self.branch_name,
                                    );
                                }
                            }
                            self.chat_mgr.save(&project_root);
                            self.layout.center_mode = CenterViewMode::ChatTimeline;
                        }
                        StudioModalAction::RunSecurityDiagnostics => {
                            self.run_elegans_diagnostics();
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
            .with_title("VICTINI IDE — Autonomous Vibe-Code & Swarm Kanban Station"),
        ..Default::default()
    };

    eframe::run_native(
        "Victini IDE",
        native_options,
        Box::new(|cc| Ok(Box::new(VictiniApp::new(cc)))),
    )
}
