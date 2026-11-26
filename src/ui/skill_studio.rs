use crate::ai::{AgentAvatarState, AvatarManager, ElegansAgent, ALL_AGENTS};
use crate::config::{
    spawn_install_candidates, spawn_scan_github_repo, spawn_update_skill_from_source,
    ConflictStrategy, DiscoveredAgent, DiscoveredSkill, ImportEvent, InstallReport,
    ScanRepoResult, SkillDiscovery, SkillScope, TrashedItem,
};
use crate::theme::{CanhTheme, Lucide};
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Sense, Stroke, Ui};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::time::Instant;

pub enum SkillStudioAction {
    CloseToChat,
    RefreshWorkspace,
    InsertIntoChat(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioMainTab {
    Skills,
    Agents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillDetailTab {
    Preview,
    Edit,
    Files,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeFilter {
    All,
    Project,
    Global,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedItem {
    None,
    SkillPath(PathBuf),
    NewSkill,
    CoreAgent(ElegansAgent),
    CustomAgentPath(PathBuf),
    NewAgent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardStep {
    Source,
    SelectSkills,
    Install,
    Done,
}

pub struct GitHubWizardState {
    pub open: bool,
    pub step: WizardStep,
    pub repo_url: String,
    pub branch_override: String,
    pub is_private: bool,
    pub token: String,
    pub filter_query: String,
    pub preview_candidate_idx: Option<usize>,
    pub target_scope: SkillScope,
    pub conflict_strategy: ConflictStrategy,

    pub is_busy: bool,
    pub progress_msg: String,
    pub progress_pct: f32,
    pub error_msg: Option<String>,
    pub scan_result: Option<ScanRepoResult>,
    pub install_report: Option<InstallReport>,
    pub rx: Option<std_mpsc::Receiver<ImportEvent>>,
}

impl Default for GitHubWizardState {
    fn default() -> Self {
        Self {
            open: false,
            step: WizardStep::Source,
            repo_url: String::new(),
            branch_override: String::new(),
            is_private: false,
            token: String::new(),
            filter_query: String::new(),
            preview_candidate_idx: None,
            target_scope: SkillScope::Project,
            conflict_strategy: ConflictStrategy::Skip,
            is_busy: false,
            progress_msg: String::new(),
            progress_pct: 0.0,
            error_msg: None,
            scan_result: None,
            install_report: None,
            rx: None,
        }
    }
}

impl GitHubWizardState {
    pub fn open_fresh(&mut self) {
        self.open = true;
        self.step = WizardStep::Source;
        self.error_msg = None;
        self.scan_result = None;
        self.install_report = None;
        self.is_busy = false;
        self.progress_msg.clear();
        self.progress_pct = 0.0;
        self.preview_candidate_idx = None;
        self.filter_query.clear();
    }
}

pub struct SkillDraft {
    pub original_path: Option<PathBuf>,
    pub scope: SkillScope,
    pub name: String,
    pub description: String,
    pub body: String,
    pub allow_overwrite: bool,
    pub dirty: bool,
}

impl Default for SkillDraft {
    fn default() -> Self {
        Self {
            original_path: None,
            scope: SkillScope::Project,
            name: String::new(),
            description: String::new(),
            body: "# Hướng dẫn cho Agent\n\n- Bước 1:\n- Bước 2:\n".to_string(),
            allow_overwrite: false,
            dirty: false,
        }
    }
}

pub struct AgentDraft {
    pub original_path: Option<PathBuf>,
    pub scope: SkillScope,
    pub name: String,
    pub role: String,
    pub avatar_key: String,
    pub model: String,
    pub default_skills: Vec<String>,
    pub body: String,
    pub allow_overwrite: bool,
    pub dirty: bool,
}

impl Default for AgentDraft {
    fn default() -> Self {
        Self {
            original_path: None,
            scope: SkillScope::Project,
            name: String::new(),
            role: String::new(),
            avatar_key: "fe_senior_dev".to_string(),
            model: String::new(),
            default_skills: Vec::new(),
            body: "Thực thi mã nguồn sạch, rõ ràng, có kiểm chứng đầy đủ.".to_string(),
            allow_overwrite: false,
            dirty: false,
        }
    }
}

pub struct SkillStudioState {
    pub main_tab: StudioMainTab,
    pub detail_tab: SkillDetailTab,
    pub scope_filter: ScopeFilter,
    pub search_query: String,
    pub selected: SelectedItem,
    pub pending_select_after_unsaved: Option<SelectedItem>,

    pub skill_draft: SkillDraft,
    pub agent_draft: AgentDraft,
    pub selected_file_rel: Option<String>,
    pub selected_file_content: String,

    pub confirm_delete: bool,
    pub undo_trashed: Option<(TrashedItem, Instant)>,
    pub status_banner: Option<(String, bool, Instant)>, // (msg, is_error, time)

    pub update_rx: Option<std_mpsc::Receiver<ImportEvent>>,
    pub is_updating_source: bool,

    pub wizard: GitHubWizardState,
}

impl Default for SkillStudioState {
    fn default() -> Self {
        Self {
            main_tab: StudioMainTab::Skills,
            detail_tab: SkillDetailTab::Preview,
            scope_filter: ScopeFilter::All,
            search_query: String::new(),
            selected: SelectedItem::None,
            pending_select_after_unsaved: None,
            skill_draft: SkillDraft::default(),
            agent_draft: AgentDraft::default(),
            selected_file_rel: None,
            selected_file_content: String::new(),
            confirm_delete: false,
            undo_trashed: None,
            status_banner: None,
            update_rx: None,
            is_updating_source: false,
            wizard: GitHubWizardState::default(),
        }
    }
}

impl SkillStudioState {
    pub fn open_new_skill(&mut self) {
        self.main_tab = StudioMainTab::Skills;
        self.detail_tab = SkillDetailTab::Edit;
        self.selected = SelectedItem::NewSkill;
        self.skill_draft = SkillDraft {
            original_path: None,
            scope: SkillScope::Project,
            name: String::new(),
            description: String::new(),
            body: "# Mục tiêu\n\nMô tả các bước và tiêu chuẩn khi kích hoạt skill này.\n".to_string(),
            allow_overwrite: false,
            dirty: true,
        };
        self.confirm_delete = false;
    }

    pub fn open_new_agent(&mut self) {
        self.main_tab = StudioMainTab::Agents;
        self.detail_tab = SkillDetailTab::Edit;
        self.selected = SelectedItem::NewAgent;
        self.agent_draft = AgentDraft {
            original_path: None,
            scope: SkillScope::Project,
            name: String::new(),
            role: String::new(),
            avatar_key: "fe_senior_dev".to_string(),
            model: String::new(),
            default_skills: Vec::new(),
            body: "Bạn là chuyên gia tập trung vào chất lượng, độ chính xác và khả năng bảo trì.".to_string(),
            allow_overwrite: false,
            dirty: true,
        };
        self.confirm_delete = false;
    }

    pub fn open_skill(&mut self, sk: &DiscoveredSkill) {
        self.main_tab = StudioMainTab::Skills;
        self.detail_tab = SkillDetailTab::Preview;
        self.selected = SelectedItem::SkillPath(sk.path.clone());
        self.load_skill_into_draft(sk);
        self.confirm_delete = false;
    }

    pub fn open_agent(&mut self, ag: &DiscoveredAgent) {
        self.main_tab = StudioMainTab::Agents;
        self.detail_tab = SkillDetailTab::Preview;
        self.selected = SelectedItem::CustomAgentPath(ag.path.clone());
        self.load_agent_into_draft(ag);
        self.confirm_delete = false;
    }

    fn load_skill_into_draft(&mut self, sk: &DiscoveredSkill) {
        self.skill_draft = SkillDraft {
            original_path: Some(sk.path.clone()),
            scope: sk.scope,
            name: sk.name.clone(),
            description: sk.description.clone(),
            body: sk.body.clone(),
            allow_overwrite: false,
            dirty: false,
        };
        self.selected_file_rel = None;
        self.selected_file_content.clear();
    }

    fn load_agent_into_draft(&mut self, ag: &DiscoveredAgent) {
        self.agent_draft = AgentDraft {
            original_path: Some(ag.path.clone()),
            scope: ag.scope,
            name: ag.name.clone(),
            role: ag.role.clone(),
            avatar_key: ag.avatar_key.clone(),
            model: ag.model.clone().unwrap_or_default(),
            default_skills: ag.default_skills.clone(),
            body: ag.body.clone(),
            allow_overwrite: false,
            dirty: false,
        };
    }

    fn is_current_dirty(&self) -> bool {
        match self.selected {
            SelectedItem::SkillPath(_) | SelectedItem::NewSkill => self.skill_draft.dirty,
            SelectedItem::CustomAgentPath(_) | SelectedItem::NewAgent => self.agent_draft.dirty,
            _ => false,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_banner = Some((msg.into(), is_error, Instant::now()));
    }

    /// Polls async GitHub import and update-from-source background channels.
    /// Returns true if workspace needs a rescan.
    pub fn poll_async_events(&mut self) -> bool {
        let mut need_rescan = false;

        // 1. Poll Wizard events
        let mut wizard_events = Vec::new();
        if let Some(ref rx) = self.wizard.rx {
            while let Ok(ev) = rx.try_recv() {
                wizard_events.push(ev);
            }
        }
        for ev in wizard_events {
            match ev {
                ImportEvent::Progress(msg, pct) => {
                    self.wizard.progress_msg = msg;
                    self.wizard.progress_pct = pct;
                }
                ImportEvent::ScanCompleted(res) => {
                    self.wizard.is_busy = false;
                    self.wizard.rx = None;
                    match res {
                        Ok(scan) => {
                            self.wizard.scan_result = Some(scan);
                            self.wizard.error_msg = None;
                            self.wizard.step = WizardStep::SelectSkills;
                        }
                        Err(err) => {
                            self.wizard.error_msg = Some(err);
                        }
                    }
                }
                ImportEvent::InstallCompleted(res) => {
                    self.wizard.is_busy = false;
                    self.wizard.rx = None;
                    match res {
                        Ok(report) => {
                            let count = report.installed.len();
                            self.wizard.install_report = Some(report);
                            self.wizard.error_msg = None;
                            self.wizard.step = WizardStep::Done;
                            self.set_status(
                                format!("Đã nạp thành công {} skill từ GitHub.", count),
                                false,
                            );
                            need_rescan = true;
                        }
                        Err(err) => {
                            self.wizard.error_msg = Some(err);
                        }
                    }
                }
                ImportEvent::UpdateCompleted(_) => {}
            }
        }

        // 2. Poll Update-from-source events
        let mut upd_events = Vec::new();
        if let Some(ref rx) = self.update_rx {
            while let Ok(ev) = rx.try_recv() {
                upd_events.push(ev);
            }
        }
        for ev in upd_events {
            match ev {
                ImportEvent::Progress(msg, _) => {
                    self.set_status(msg, false);
                }
                ImportEvent::UpdateCompleted(res) => {
                    self.is_updating_source = false;
                    self.update_rx = None;
                    match res {
                        Ok(msg) => {
                            self.set_status(msg, false);
                            need_rescan = true;
                        }
                        Err(err) => {
                            self.set_status(err, true);
                        }
                    }
                }
                _ => {}
            }
        }

        need_rescan
    }
}

fn avatar_agent_from_key(key: &str) -> ElegansAgent {
    ALL_AGENTS
        .iter()
        .copied()
        .find(|a| a.name() == key)
        .unwrap_or(ElegansAgent::SeniorDev)
}

#[allow(clippy::too_many_arguments)]
pub fn render_skill_agent_studio(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    project_root: &Path,
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action: Option<SkillStudioAction> = None;

    if state.poll_async_events() {
        action = Some(SkillStudioAction::RefreshWorkspace);
    }
    if state.wizard.is_busy || state.is_updating_source {
        ui.ctx().request_repaint();
    }

    // Expire undo & status banners
    if let Some((_, ts)) = &state.undo_trashed {
        if ts.elapsed().as_secs_f32() > 6.5 {
            state.undo_trashed = None;
        }
    }
    if let Some((_, _, ts)) = &state.status_banner {
        if ts.elapsed().as_secs_f32() > 6.0 && !state.is_updating_source {
            state.status_banner = None;
        }
    }

    // Auto-select first skill/agent if nothing selected yet
    if state.selected == SelectedItem::None {
        match state.main_tab {
            StudioMainTab::Skills => {
                if let Some(first) = skills.first() {
                    state.open_skill(first);
                }
            }
            StudioMainTab::Agents => {
                state.selected = SelectedItem::CoreAgent(ElegansAgent::SeniorDev);
            }
        }
    }

    ui.vertical(|ui| {
        // 1. TOP HEADER BAR
        Frame::none()
            .fill(theme.surface_1)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::symmetric(10.0, 7.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{} Skills & Agents", Lucide::LAYERS))
                            .color(theme.text_paper)
                            .strong()
                            .size(13.0),
                    );
                    ui.add_space(10.0);

                    // Segmented Tabs
                    for (tab, label, count) in [
                        (StudioMainTab::Skills, "Skills", skills.len()),
                        (StudioMainTab::Agents, "Agents", ALL_AGENTS.len() + agents.len()),
                    ] {
                        let is_act = state.main_tab == tab;
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} ({})", label, count))
                                        .color(if is_act {
                                            theme.text_paper
                                        } else {
                                            theme.text_silver
                                        })
                                        .strong()
                                        .size(11.0),
                                )
                                .fill(if is_act {
                                    theme.surface_3
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .stroke(Stroke::new(
                                    1.0,
                                    if is_act {
                                        theme.border_strong
                                    } else {
                                        Color32::TRANSPARENT
                                    },
                                ))
                                .rounding(Rounding::same(5.0)),
                            )
                            .clicked()
                        {
                            state.main_tab = tab;
                            state.confirm_delete = false;
                            match tab {
                                StudioMainTab::Skills => {
                                    if let Some(first) = skills.first() {
                                        state.open_skill(first);
                                    } else {
                                        state.selected = SelectedItem::None;
                                    }
                                }
                                StudioMainTab::Agents => {
                                    state.selected = SelectedItem::CoreAgent(ElegansAgent::SeniorDev);
                                }
                            }
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} Đóng", Lucide::X))
                                        .color(theme.text_silver)
                                        .size(11.0),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            action = Some(SkillStudioAction::CloseToChat);
                        }

                        // Primary CTA: Create new
                        let create_lbl = match state.main_tab {
                            StudioMainTab::Skills => format!("{} Tạo Skill", Lucide::PLUS),
                            StudioMainTab::Agents => format!("{} Tạo Agent", Lucide::PLUS),
                        };
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(create_lbl)
                                        .color(Color32::BLACK)
                                        .strong()
                                        .size(11.0),
                                )
                                .fill(theme.accent_pop)
                                .rounding(Rounding::same(5.0)),
                            )
                            .clicked()
                        {
                            match state.main_tab {
                                StudioMainTab::Skills => state.open_new_skill(),
                                StudioMainTab::Agents => state.open_new_agent(),
                            }
                        }

                        // GitHub Import Button
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{} Nạp từ GitHub",
                                        Lucide::DOWNLOAD
                                    ))
                                    .color(theme.text_paper)
                                    .size(11.0),
                                )
                                .fill(theme.surface_2)
                                .stroke(Stroke::new(1.0, theme.border_strong))
                                .rounding(Rounding::same(5.0)),
                            )
                            .clicked()
                        {
                            state.wizard.open_fresh();
                        }
                    });
                });
            });

        // 2. STATUS / UNDO BANNERS
        if let Some((trashed, ts)) = state.undo_trashed.clone() {
            let rem = (6.5 - ts.elapsed().as_secs_f32()).max(0.0);
            ui.add_space(6.0);
            Frame::none()
                .fill(theme.surface_2)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border_strong))
                .inner_margin(Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Đã chuyển {} '{}' vào thùng rác · Hoàn tác trong {:.0}s",
                                Lucide::TRASH,
                                trashed.kind_label,
                                trashed.name,
                                rem.ceil()
                            ))
                            .color(theme.text_paper)
                            .size(11.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!("{} Hoàn tác", Lucide::UNDO))
                                            .color(Color32::BLACK)
                                            .strong()
                                            .size(10.5),
                                    )
                                    .fill(theme.accent_pop)
                                    .rounding(Rounding::same(4.0)),
                                )
                                .clicked()
                            {
                                match SkillDiscovery::restore_trashed(&trashed) {
                                    Ok(_) => {
                                        state.undo_trashed = None;
                                        state.set_status(
                                            format!("Đã khôi phục '{}'.", trashed.name),
                                            false,
                                        );
                                        action = Some(SkillStudioAction::RefreshWorkspace);
                                    }
                                    Err(e) => state.set_status(format!("Lỗi khôi phục: {}", e), true),
                                }
                            }
                        });
                    });
                });
            ui.ctx().request_repaint();
        }

        if let Some((msg, is_err, _)) = state.status_banner.clone() {
            ui.add_space(5.0);
            let col = if is_err { theme.danger } else { theme.success };
            Frame::none()
                .fill(Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 22))
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, col))
                .inner_margin(Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(msg).color(col).size(11.0));
                });
        }

        // Unsaved changes guard prompt
        if let Some(pending) = state.pending_select_after_unsaved.clone() {
            ui.add_space(5.0);
            Frame::none()
                .fill(Color32::from_rgba_unmultiplied(
                    theme.warning.r(),
                    theme.warning.g(),
                    theme.warning.b(),
                    24,
                ))
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.warning))
                .inner_margin(Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Bạn có thay đổi chưa lưu. Bạn muốn bỏ thay đổi và chuyển mục?",
                                Lucide::ALERT_TRIANGLE
                            ))
                            .color(theme.warning)
                            .size(11.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Bỏ thay đổi")
                                            .color(Color32::WHITE)
                                            .size(10.5),
                                    )
                                    .fill(theme.danger)
                                    .rounding(Rounding::same(4.0)),
                                )
                                .clicked()
                            {
                                state.skill_draft.dirty = false;
                                state.agent_draft.dirty = false;
                                state.pending_select_after_unsaved = None;
                                apply_selection(state, pending, skills, agents);
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Ở lại")
                                            .color(theme.text_paper)
                                            .size(10.5),
                                    )
                                    .fill(theme.surface_2)
                                    .rounding(Rounding::same(4.0)),
                                )
                                .clicked()
                            {
                                state.pending_select_after_unsaved = None;
                            }
                        });
                    });
                });
        }

        ui.add_space(8.0);

        // 3. TWO-PANE BODY: LEFT LIST + RIGHT INSPECTOR/EDITOR
        let total_w = ui.available_width();
        let body_h = (ui.available_height() - 4.0).max(260.0);
        let left_w = (total_w * 0.30).clamp(220.0, 310.0);
        let right_w = (total_w - left_w - 10.0).max(320.0);

        ui.horizontal_top(|ui| {
            // LEFT PANE
            ui.allocate_ui_with_layout(
                vec2(left_w, body_h),
                Layout::top_down(Align::Min),
                |ui| {
                    render_left_pane(ui, theme, elapsed_secs, skills, agents, state);
                },
            );

            ui.add_space(8.0);

            // RIGHT PANE
            ui.allocate_ui_with_layout(
                vec2(right_w, body_h),
                Layout::top_down(Align::Min),
                |ui| {
                    if let Some(act) = render_right_pane(
                        ui,
                        theme,
                        elapsed_secs,
                        project_root,
                        skills,
                        agents,
                        state,
                    ) {
                        action = Some(act);
                    }
                },
            );
        });
    });

    // 4. GITHUB IMPORT WIZARD MODAL
    if state.wizard.open {
        if let Some(w_act) = render_github_wizard_modal(ui.ctx(), theme, project_root, state) {
            action = Some(w_act);
        }
    }

    action
}

fn request_selection(
    state: &mut SkillStudioState,
    target: SelectedItem,
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
) {
    if state.selected == target {
        return;
    }
    if state.is_current_dirty() {
        state.pending_select_after_unsaved = Some(target);
    } else {
        apply_selection(state, target, skills, agents);
    }
}

fn apply_selection(
    state: &mut SkillStudioState,
    target: SelectedItem,
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
) {
    state.confirm_delete = false;
    state.selected = target.clone();
    match target {
        SelectedItem::SkillPath(ref p) => {
            if let Some(sk) = skills.iter().find(|s| &s.path == p) {
                state.load_skill_into_draft(sk);
            }
            if state.detail_tab == SkillDetailTab::Edit {
                state.detail_tab = SkillDetailTab::Preview;
            }
        }
        SelectedItem::CustomAgentPath(ref p) => {
            if let Some(ag) = agents.iter().find(|a| &a.path == p) {
                state.load_agent_into_draft(ag);
            }
            state.detail_tab = SkillDetailTab::Preview;
        }
        SelectedItem::NewSkill => state.open_new_skill(),
        SelectedItem::NewAgent => state.open_new_agent(),
        _ => {}
    }
}

fn render_left_pane(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
    state: &mut SkillStudioState,
) {
    Frame::none()
        .fill(theme.surface_1)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            // Search Box
            Frame::none()
                .fill(theme.bg_field)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border_strong))
                .inner_margin(Margin::symmetric(7.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::SEARCH)
                                .color(theme.text_silver)
                                .size(11.0),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut state.search_query)
                                .desired_width(ui.available_width() - 16.0)
                                .hint_text(match state.main_tab {
                                    StudioMainTab::Skills => "Tìm skill...",
                                    StudioMainTab::Agents => "Tìm agent...",
                                })
                                .frame(false)
                                .text_color(theme.text_paper),
                        );
                    });
                });

            ui.add_space(6.0);

            // Scope Filter
            ui.horizontal(|ui| {
                for (sf, lbl) in [
                    (ScopeFilter::All, "Tất cả"),
                    (ScopeFilter::Project, "Project"),
                    (ScopeFilter::Global, "Global"),
                ] {
                    let sel = state.scope_filter == sf;
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(lbl)
                                    .color(if sel {
                                        theme.text_paper
                                    } else {
                                        theme.text_silver
                                    })
                                    .size(10.0),
                            )
                            .fill(if sel {
                                theme.surface_3
                            } else {
                                Color32::TRANSPARENT
                            })
                            .rounding(Rounding::same(4.0)),
                        )
                        .clicked()
                    {
                        state.scope_filter = sf;
                    }
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let q = state.search_query.trim().to_lowercase();

            ScrollArea::vertical()
                .id_salt("studio_left_list_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| match state.main_tab {
                    StudioMainTab::Skills => {
                        let filtered: Vec<&DiscoveredSkill> = skills
                            .iter()
                            .filter(|s| {
                                let scope_ok = match state.scope_filter {
                                    ScopeFilter::All => true,
                                    ScopeFilter::Project => s.scope == SkillScope::Project,
                                    ScopeFilter::Global => s.scope == SkillScope::Global,
                                };
                                let q_ok = q.is_empty()
                                    || s.name.to_lowercase().contains(&q)
                                    || s.description.to_lowercase().contains(&q);
                                scope_ok && q_ok
                            })
                            .collect();

                        if filtered.is_empty() {
                            ui.add_space(20.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new("Chưa có skill phù hợp")
                                        .color(theme.text_silver)
                                        .size(11.0),
                                );
                                ui.add_space(8.0);
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(format!(
                                                "{} Nạp từ GitHub",
                                                Lucide::DOWNLOAD
                                            ))
                                            .color(theme.text_paper)
                                            .size(10.5),
                                        )
                                        .fill(theme.surface_2)
                                        .rounding(Rounding::same(5.0)),
                                    )
                                    .clicked()
                                {
                                    state.wizard.open_fresh();
                                }
                            });
                            return;
                        }

                        for scope_group in [SkillScope::Project, SkillScope::Global] {
                            let group_items: Vec<&&DiscoveredSkill> = filtered
                                .iter()
                                .filter(|s| s.scope == scope_group)
                                .collect();
                            if group_items.is_empty() {
                                continue;
                            }
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} ({})",
                                    scope_group.label().to_uppercase(),
                                    group_items.len()
                                ))
                                .color(theme.text_silver)
                                .strong()
                                .size(9.5),
                            );
                            ui.add_space(3.0);

                            for sk in group_items {
                                let is_sel = matches!(
                                    &state.selected,
                                    SelectedItem::SkillPath(p) if p == &sk.path
                                );
                                let resp = Frame::none()
                                    .fill(if is_sel {
                                        theme.surface_3
                                    } else {
                                        Color32::TRANSPARENT
                                    })
                                    .rounding(Rounding::same(6.0))
                                    .stroke(Stroke::new(
                                        1.0,
                                        if is_sel {
                                            theme.accent_pop
                                        } else {
                                            Color32::TRANSPARENT
                                        },
                                    ))
                                    .inner_margin(Margin::symmetric(7.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let (dot, _) = ui.allocate_exact_size(
                                                vec2(7.0, 7.0),
                                                Sense::hover(),
                                            );
                                            if sk.enabled {
                                                ui.painter().circle_filled(
                                                    dot.center(),
                                                    3.0,
                                                    theme.success,
                                                );
                                            } else {
                                                ui.painter().circle_stroke(
                                                    dot.center(),
                                                    3.0,
                                                    Stroke::new(1.0, theme.text_silver),
                                                );
                                            }

                                            ui.label(
                                                egui::RichText::new(&sk.name)
                                                    .color(if sk.enabled {
                                                        theme.text_paper
                                                    } else {
                                                        theme.text_silver
                                                    })
                                                    .strong()
                                                    .size(11.2),
                                            );

                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    if sk.source.is_some() {
                                                        Frame::none()
                                                            .fill(theme.surface_2)
                                                            .rounding(Rounding::same(3.0))
                                                            .inner_margin(Margin::symmetric(
                                                                4.0, 0.5,
                                                            ))
                                                            .show(ui, |ui| {
                                                                ui.label(
                                                                    egui::RichText::new("GH")
                                                                        .color(theme.info)
                                                                        .strong()
                                                                        .size(8.8),
                                                                );
                                                            });
                                                    }
                                                    if sk.has_scripts {
                                                        ui.label(
                                                            egui::RichText::new(Lucide::CODE)
                                                                .color(theme.warning)
                                                                .size(9.5),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                        let short_desc: String =
                                            sk.description.chars().take(54).collect();
                                        ui.label(
                                            egui::RichText::new(short_desc)
                                                .color(theme.text_silver)
                                                .size(9.8),
                                        );
                                    });
                                if resp.response.interact(Sense::click()).clicked() {
                                    request_selection(
                                        state,
                                        SelectedItem::SkillPath(sk.path.clone()),
                                        skills,
                                        agents,
                                    );
                                }
                                ui.add_space(3.0);
                            }
                            ui.add_space(6.0);
                        }
                    }
                    StudioMainTab::Agents => {
                        // Core Agents (Read-only)
                        ui.label(
                            egui::RichText::new(format!("CORE AGENTS ({})", ALL_AGENTS.len()))
                                .color(theme.text_silver)
                                .strong()
                                .size(9.5),
                        );
                        ui.add_space(3.0);
                        for &ag in ALL_AGENTS {
                            if !q.is_empty()
                                && !ag.name().to_lowercase().contains(&q)
                                && !ag.title().to_lowercase().contains(&q)
                            {
                                continue;
                            }
                            let is_sel = state.selected == SelectedItem::CoreAgent(ag);
                            let resp = Frame::none()
                                .fill(if is_sel {
                                    theme.surface_3
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .rounding(Rounding::same(6.0))
                                .stroke(Stroke::new(
                                    1.0,
                                    if is_sel {
                                        theme.accent_pop
                                    } else {
                                        Color32::TRANSPARENT
                                    },
                                ))
                                .inner_margin(Margin::symmetric(7.0, 5.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        AvatarManager::render_animated_avatar(
                                            ui,
                                            ag,
                                            AgentAvatarState::Idle,
                                            20.0,
                                            elapsed_secs,
                                            theme.accent_pop,
                                        );
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(ag.name())
                                                    .color(theme.text_paper)
                                                    .strong()
                                                    .size(11.0),
                                            );
                                            ui.label(
                                                egui::RichText::new(ag.title())
                                                    .color(theme.text_silver)
                                                    .size(9.6),
                                            );
                                        });
                                    });
                                });
                            if resp.response.interact(Sense::click()).clicked() {
                                request_selection(
                                    state,
                                    SelectedItem::CoreAgent(ag),
                                    skills,
                                    agents,
                                );
                            }
                            ui.add_space(3.0);
                        }

                        ui.add_space(8.0);
                        // Custom Agents
                        let filtered_custom: Vec<&DiscoveredAgent> = agents
                            .iter()
                            .filter(|a| {
                                let scope_ok = match state.scope_filter {
                                    ScopeFilter::All => true,
                                    ScopeFilter::Project => a.scope == SkillScope::Project,
                                    ScopeFilter::Global => a.scope == SkillScope::Global,
                                };
                                let q_ok = q.is_empty()
                                    || a.name.to_lowercase().contains(&q)
                                    || a.role.to_lowercase().contains(&q);
                                scope_ok && q_ok
                            })
                            .collect();

                        ui.label(
                            egui::RichText::new(format!(
                                "CUSTOM AGENTS ({})",
                                filtered_custom.len()
                            ))
                            .color(theme.text_silver)
                            .strong()
                            .size(9.5),
                        );
                        ui.add_space(3.0);

                        for cag in filtered_custom {
                            let is_sel = matches!(
                                &state.selected,
                                SelectedItem::CustomAgentPath(p) if p == &cag.path
                            );
                            let resp = Frame::none()
                                .fill(if is_sel {
                                    theme.surface_3
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .rounding(Rounding::same(6.0))
                                .stroke(Stroke::new(
                                    1.0,
                                    if is_sel {
                                        theme.accent_pop
                                    } else {
                                        Color32::TRANSPARENT
                                    },
                                ))
                                .inner_margin(Margin::symmetric(7.0, 5.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        AvatarManager::render_animated_avatar(
                                            ui,
                                            avatar_agent_from_key(&cag.avatar_key),
                                            AgentAvatarState::Idle,
                                            20.0,
                                            elapsed_secs,
                                            theme.accent_pop,
                                        );
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(format!("@{}", cag.name))
                                                    .color(theme.text_paper)
                                                    .strong()
                                                    .size(11.0),
                                            );
                                            ui.label(
                                                egui::RichText::new(&cag.role)
                                                    .color(theme.text_silver)
                                                    .size(9.6),
                                            );
                                        });
                                    });
                                });
                            if resp.response.interact(Sense::click()).clicked() {
                                request_selection(
                                    state,
                                    SelectedItem::CustomAgentPath(cag.path.clone()),
                                    skills,
                                    agents,
                                );
                            }
                            ui.add_space(3.0);
                        }
                    }
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn render_right_pane(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    project_root: &Path,
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action: Option<SkillStudioAction> = None;

    Frame::none()
        .fill(theme.surface_1)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| match state.selected.clone() {
            SelectedItem::None => {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "Chọn một Skill hoặc Agent ở cột trái, hoặc bấm Tạo mới / Nạp từ GitHub.",
                        )
                        .color(theme.text_silver)
                        .size(12.0),
                    );
                });
            }
            SelectedItem::SkillPath(path) => {
                if let Some(sk) = skills.iter().find(|s| s.path == path) {
                    action = render_skill_detail(ui, theme, project_root, sk, skills, state);
                } else {
                    state.selected = SelectedItem::None;
                }
            }
            SelectedItem::NewSkill => {
                action = render_skill_editor_form(ui, theme, project_root, None, skills, state);
            }
            SelectedItem::CoreAgent(ag) => {
                action = render_core_agent_detail(ui, theme, elapsed_secs, ag, state);
            }
            SelectedItem::CustomAgentPath(path) => {
                if let Some(ag) = agents.iter().find(|a| a.path == path) {
                    action = render_custom_agent_detail(
                        ui,
                        theme,
                        elapsed_secs,
                        project_root,
                        ag,
                        skills,
                        agents,
                        state,
                    );
                } else {
                    state.selected = SelectedItem::CoreAgent(ElegansAgent::SeniorDev);
                }
            }
            SelectedItem::NewAgent => {
                action = render_agent_editor_form(
                    ui,
                    theme,
                    elapsed_secs,
                    project_root,
                    None,
                    skills,
                    agents,
                    state,
                );
            }
        });

    action
}

fn render_skill_detail(
    ui: &mut Ui,
    theme: &CanhTheme,
    project_root: &Path,
    sk: &DiscoveredSkill,
    all_skills: &[DiscoveredSkill],
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action = None;

    // Header row: Name + Scope Badge + Enable Toggle
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("${}", sk.name))
                .color(theme.text_paper)
                .strong()
                .size(14.5),
        );
        Frame::none()
            .fill(theme.surface_2)
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(6.0, 2.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(sk.scope.label())
                        .color(theme.text_silver)
                        .size(10.0),
                );
            });
        if sk.has_scripts {
            Frame::none()
                .fill(Color32::from_rgba_unmultiplied(
                    theme.warning.r(),
                    theme.warning.g(),
                    theme.warning.b(),
                    28,
                ))
                .rounding(Rounding::same(4.0))
                .inner_margin(Margin::symmetric(6.0, 2.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} Có scripts", Lucide::SHIELD_ALERT))
                            .color(theme.warning)
                            .size(10.0),
                    );
                });
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let mut enabled = sk.enabled;
            let toggle_txt = if enabled { "Đang bật" } else { "Đang tắt" };
            let toggle_col = if enabled {
                theme.success
            } else {
                theme.text_silver
            };
            if ui
                .checkbox(
                    &mut enabled,
                    egui::RichText::new(toggle_txt).color(toggle_col).size(11.0),
                )
                .changed()
            {
                SkillDiscovery::set_skill_enabled(&sk.path, enabled);
                action = Some(SkillStudioAction::RefreshWorkspace);
            }
        });
    });

    ui.add_space(3.0);
    ui.label(
        egui::RichText::new(&sk.description)
            .color(theme.text_silver)
            .size(11.5),
    );

    if let Some(ref src) = sk.source {
        let short_sha = &src.commit_sha[..7.min(src.commit_sha.len())];
        ui.add_space(3.0);
        ui.label(
            egui::RichText::new(format!(
                "{} Nguồn: {} ({}@{}) · cài lúc {}",
                Lucide::GIT_BRANCH,
                src.repo,
                src.git_ref,
                short_sha,
                src.installed_at
            ))
            .color(theme.info)
            .size(10.2),
        );
    }

    ui.add_space(8.0);

    // Detail Sub-tabs: Xem | Sửa | File
    ui.horizontal(|ui| {
        for (t, icon, lbl) in [
            (SkillDetailTab::Preview, Lucide::EYE, "Xem"),
            (SkillDetailTab::Edit, Lucide::PENCIL, "Sửa"),
            (SkillDetailTab::Files, Lucide::FOLDER_OPEN, "File"),
        ] {
            let sel = state.detail_tab == t;
            let suffix = if t == SkillDetailTab::Edit && state.skill_draft.dirty {
                " ●"
            } else if t == SkillDetailTab::Files {
                ""
            } else {
                ""
            };
            let full_lbl = if t == SkillDetailTab::Files {
                format!("{} {} ({})", icon, lbl, sk.files.len())
            } else {
                format!("{} {}{}", icon, lbl, suffix)
            };
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(full_lbl)
                            .color(if sel {
                                theme.text_paper
                            } else {
                                theme.text_silver
                            })
                            .strong()
                            .size(11.0),
                    )
                    .fill(if sel {
                        theme.surface_3
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(5.0)),
                )
                .clicked()
            {
                state.detail_tab = t;
            }
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);

    // Body area
    let body_h = (ui.available_height() - 48.0).max(150.0);
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), body_h),
        Layout::top_down(Align::Min),
        |ui| match state.detail_tab {
            SkillDetailTab::Preview => {
                render_markdown_preview(ui, theme, &sk.body);
            }
            SkillDetailTab::Edit => {
                if let Some(a) = render_skill_editor_form(
                    ui,
                    theme,
                    project_root,
                    Some(sk),
                    all_skills,
                    state,
                ) {
                    action = Some(a);
                }
            }
            SkillDetailTab::Files => {
                render_skill_files_tab(ui, theme, sk, state);
            }
        },
    );

    // Delete confirmation bar
    if state.confirm_delete {
        ui.add_space(4.0);
        Frame::none()
            .fill(Color32::from_rgba_unmultiplied(
                theme.danger.r(),
                theme.danger.g(),
                theme.danger.b(),
                26,
            ))
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, theme.danger))
            .inner_margin(Margin::symmetric(8.0, 5.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Chuyển '{}' ({} file tại {}) vào thùng rác?",
                            sk.name,
                            sk.files.len(),
                            sk.path.to_string_lossy()
                        ))
                        .color(theme.danger)
                        .size(10.5),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Xác nhận xóa")
                                        .color(Color32::WHITE)
                                        .strong()
                                        .size(10.5),
                                )
                                .fill(theme.danger)
                                .rounding(Rounding::same(4.0)),
                            )
                            .clicked()
                        {
                            match SkillDiscovery::trash_skill(&sk.path) {
                                Ok(trashed) => {
                                    state.undo_trashed = Some((trashed, Instant::now()));
                                    state.confirm_delete = false;
                                    state.selected = SelectedItem::None;
                                    action = Some(SkillStudioAction::RefreshWorkspace);
                                }
                                Err(e) => state.set_status(format!("Lỗi xóa: {}", e), true),
                            }
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Hủy")
                                        .color(theme.text_silver)
                                        .size(10.5),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            state.confirm_delete = false;
                        }
                    });
                });
            });
    }

    // Bottom Footer Bar
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if let Some(ref src) = sk.source {
            let btn_lbl = if state.is_updating_source {
                format!("{} Đang kiểm tra...", Lucide::REFRESH)
            } else {
                format!("{} Cập nhật từ nguồn", Lucide::REFRESH)
            };
            if ui
                .add_enabled(
                    !state.is_updating_source,
                    egui::Button::new(
                        egui::RichText::new(btn_lbl)
                            .color(theme.info)
                            .size(10.8),
                    )
                    .fill(theme.surface_2)
                    .rounding(Rounding::same(5.0)),
                )
                .clicked()
            {
                state.is_updating_source = true;
                state.update_rx = Some(spawn_update_skill_from_source(
                    sk.path.clone(),
                    src.clone(),
                    None,
                ));
            }
        }

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(format!("{} Dùng trong Chat", Lucide::SPARKLES))
                        .color(Color32::BLACK)
                        .strong()
                        .size(10.8),
                )
                .fill(theme.accent_pop)
                .rounding(Rounding::same(5.0)),
            )
            .clicked()
        {
            action = Some(SkillStudioAction::InsertIntoChat(format!("${} ", sk.name)));
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} Xóa", Lucide::TRASH))
                            .color(theme.danger)
                            .size(10.8),
                    )
                    .fill(Color32::TRANSPARENT),
                )
                .clicked()
            {
                state.confirm_delete = !state.confirm_delete;
            }
        });
    });

    action
}

fn render_skill_editor_form(
    ui: &mut Ui,
    theme: &CanhTheme,
    project_root: &Path,
    existing: Option<&DiscoveredSkill>,
    all_skills: &[DiscoveredSkill],
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action = None;
    let draft = &mut state.skill_draft;

    if existing.is_none() {
        ui.label(
            egui::RichText::new(format!("{} Tạo Skill mới", Lucide::PLUS))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
        );
        ui.add_space(6.0);
    }

    // Scope selector
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Phạm vi lưu:")
                .color(theme.text_silver)
                .size(10.8),
        );
        for sc in [SkillScope::Project, SkillScope::Global] {
            if ui
                .radio_value(&mut draft.scope, sc, sc.label())
                .changed()
            {
                draft.dirty = true;
            }
        }
        let target_hint = match draft.scope {
            SkillScope::Project => "skills/<slug>/SKILL.md",
            SkillScope::Global => "~/.gemini/config/skills/<slug>/SKILL.md",
        };
        ui.label(
            egui::RichText::new(target_hint)
                .color(theme.text_silver)
                .monospace()
                .size(9.8),
        );
    });

    ui.add_space(6.0);

    // Name + Realtime Slug & Collision Validation
    let slug = SkillDiscovery::sanitize_skill_slug(&draft.name);
    let collision = !slug.is_empty()
        && all_skills.iter().any(|s| {
            s.scope == draft.scope
                && s.name == slug
                && draft.original_path.as_ref() != Some(&s.path)
        });

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Tên Skill:")
                .color(theme.text_silver)
                .size(10.8),
        );
        if ui
            .add(
                egui::TextEdit::singleline(&mut draft.name)
                    .desired_width(220.0)
                    .hint_text("vd: api-security-audit"),
            )
            .changed()
        {
            draft.dirty = true;
        }
        if !slug.is_empty() && slug != draft.name.trim() {
            ui.label(
                egui::RichText::new(format!("→ slug: {}", slug))
                    .color(theme.text_silver)
                    .monospace()
                    .size(10.0),
            );
        }
    });

    if draft.name.trim().is_empty() || slug.is_empty() {
        ui.label(
            egui::RichText::new("Tên skill không được để trống (dùng chữ cái, số, dấu gạch ngang).")
                .color(theme.danger)
                .size(10.0),
        );
    } else if collision {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} Skill '{}' đã tồn tại trong {}.",
                    Lucide::ALERT_TRIANGLE,
                    slug,
                    draft.scope.label()
                ))
                .color(theme.warning)
                .size(10.2),
            );
            ui.checkbox(&mut draft.allow_overwrite, "Cho phép ghi đè");
        });
    }

    ui.add_space(6.0);

    // Description with char counter
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Mô tả ngắn (khi nào dùng skill):")
                .color(theme.text_silver)
                .size(10.8),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{} ký tự", draft.description.chars().count()))
                    .color(theme.text_silver)
                    .size(9.8),
            );
        });
    });
    if ui
        .add(
            egui::TextEdit::singleline(&mut draft.description)
                .desired_width(ui.available_width())
                .hint_text("VD: Dùng khi cần kiểm tra lỗ hổng bảo mật REST API..."),
        )
        .changed()
    {
        draft.dirty = true;
    }

    ui.add_space(6.0);
    ui.label(
        egui::RichText::new("Nội dung Markdown (phần thân SKILL.md — frontmatter tự sinh):")
            .color(theme.text_silver)
            .size(10.8),
    );
    ui.add_space(3.0);

    let editor_h = (ui.available_height() - 42.0).max(120.0);
    ScrollArea::vertical()
        .id_salt("skill_body_editor_scroll")
        .max_height(editor_h)
        .show(ui, |ui| {
            if ui
                .add(
                    egui::TextEdit::multiline(&mut draft.body)
                        .desired_rows(12)
                        .desired_width(ui.available_width())
                        .code_editor(),
                )
                .changed()
            {
                draft.dirty = true;
            }
        });

    ui.add_space(8.0);
    let can_save = !slug.is_empty() && (!collision || draft.allow_overwrite) && draft.dirty;

    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("Hủy thay đổi")
                        .color(theme.text_silver)
                        .size(10.8),
                )
                .fill(theme.surface_2)
                .rounding(Rounding::same(5.0)),
            )
            .clicked()
        {
            if let Some(sk) = existing {
                state.load_skill_into_draft(sk);
                state.detail_tab = SkillDetailTab::Preview;
            } else {
                state.skill_draft.dirty = false;
                state.selected = SelectedItem::None;
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add_enabled(
                    can_save,
                    egui::Button::new(
                        egui::RichText::new(format!("{} Lưu Skill", Lucide::SAVE))
                            .color(Color32::BLACK)
                            .strong()
                            .size(11.0),
                    )
                    .fill(theme.accent_pop)
                    .rounding(Rounding::same(5.0)),
                )
                .clicked()
            {
                let res = SkillDiscovery::save_skill(
                    project_root,
                    state.skill_draft.scope,
                    &state.skill_draft.name,
                    &state.skill_draft.description,
                    &state.skill_draft.body,
                    state.skill_draft.original_path.as_deref(),
                    state.skill_draft.allow_overwrite,
                );
                match res {
                    Ok(saved_path) => {
                        state.skill_draft.original_path = Some(saved_path.clone());
                        state.skill_draft.dirty = false;
                        state.skill_draft.allow_overwrite = false;
                        state.selected = SelectedItem::SkillPath(saved_path);
                        state.detail_tab = SkillDetailTab::Preview;
                        state.set_status(format!("Đã lưu skill '{}'.", slug), false);
                        action = Some(SkillStudioAction::RefreshWorkspace);
                    }
                    Err(e) => {
                        state.set_status(format!("Không thể lưu: {}", e), true);
                    }
                }
            }
        });
    });

    action
}

fn render_skill_files_tab(
    ui: &mut Ui,
    theme: &CanhTheme,
    sk: &DiscoveredSkill,
    state: &mut SkillStudioState,
) {
    ui.columns(2, |cols| {
        cols[0].vertical(|ui| {
            ui.label(
                egui::RichText::new(format!("Thư mục: {}", sk.path.to_string_lossy()))
                    .color(theme.text_silver)
                    .size(9.8),
            );
            ui.add_space(4.0);
            ScrollArea::vertical()
                .id_salt("skill_files_list")
                .show(ui, |ui| {
                    for rel in &sk.files {
                        let is_exec = rel.starts_with("scripts/")
                            || rel.ends_with(".sh")
                            || rel.ends_with(".py")
                            || rel.ends_with(".js")
                            || rel.ends_with(".ts");
                        let is_sel = state.selected_file_rel.as_deref() == Some(rel.as_str());
                        let label = if is_exec {
                            format!("{} {} (script)", Lucide::FILE_CODE, rel)
                        } else {
                            format!("{} {}", Lucide::FILE_TEXT, rel)
                        };
                        if ui
                            .selectable_label(
                                is_sel,
                                egui::RichText::new(label)
                                    .color(if is_exec {
                                        theme.warning
                                    } else {
                                        theme.text_paper
                                    })
                                    .size(10.8),
                            )
                            .clicked()
                        {
                            state.selected_file_rel = Some(rel.clone());
                            state.selected_file_content = fs::read_to_string(sk.path.join(rel))
                                .unwrap_or_else(|_| "[Không thể đọc file nhị phân]".to_string());
                        }
                    }
                });
        });

        cols[1].vertical(|ui| {
            if let Some(ref rel) = state.selected_file_rel {
                ui.label(
                    egui::RichText::new(rel)
                        .color(theme.accent_pop)
                        .strong()
                        .size(11.0),
                );
                ui.add_space(4.0);
                Frame::none()
                    .fill(theme.bg_field)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .id_salt("skill_file_content_scroll")
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&state.selected_file_content)
                                        .color(theme.text_paper)
                                        .monospace()
                                        .size(10.2),
                                );
                            });
                    });
            } else {
                ui.label(
                    egui::RichText::new("Chọn một file bên trái để xem nội dung.")
                        .color(theme.text_silver)
                        .size(11.0),
                );
            }
        });
    });
}

fn render_core_agent_detail(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    ag: ElegansAgent,
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        AvatarManager::render_animated_avatar(
            ui,
            ag,
            AgentAvatarState::Answering,
            44.0,
            elapsed_secs,
            theme.accent_pop,
        );
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} {}", ag.icon(), ag.name()))
                        .color(theme.text_paper)
                        .strong()
                        .size(15.0),
                );
                Frame::none()
                    .fill(theme.surface_2)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} Core (chỉ đọc)", Lucide::LOCK))
                                .color(theme.text_silver)
                                .size(9.8),
                        );
                    });
            });
            ui.label(
                egui::RichText::new(ag.title())
                    .color(theme.accent_pop)
                    .size(12.0),
            );
            ui.label(
                egui::RichText::new(format!("Tier: {}", ag.ponytail_tier()))
                    .color(theme.text_silver)
                    .monospace()
                    .size(10.2),
            );
        });
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(10.0);

    ui.label(
        egui::RichText::new(
            "Agent tích hợp sẵn trong Victini Swarm. Bạn có thể nhân bản thành Custom Agent để tùy biến vai trò, system prompt và danh sách skill mặc định.",
        )
        .color(theme.text_silver)
        .size(11.5),
    );

    ui.add_space(14.0);
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(format!(
                        "{} Nhân bản thành Custom Agent",
                        Lucide::COPY_PLUS
                    ))
                    .color(Color32::BLACK)
                    .strong()
                    .size(11.0),
                )
                .fill(theme.accent_pop)
                .rounding(Rounding::same(5.0)),
            )
            .clicked()
        {
            state.open_new_agent();
            state.agent_draft.name = format!("{}_custom", ag.name());
            state.agent_draft.role = ag.title().to_string();
            state.agent_draft.avatar_key = ag.name().to_string();
            state.agent_draft.body = format!(
                "# Custom {}\n\nKế thừa vai trò: {} ({}).\n",
                ag.name(),
                ag.title(),
                ag.ponytail_tier()
            );
        }

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(format!("{} Triệu hồi trong Chat", Lucide::SPARKLES))
                        .color(theme.text_paper)
                        .size(11.0),
                )
                .fill(theme.surface_2)
                .rounding(Rounding::same(5.0)),
            )
            .clicked()
        {
            action = Some(SkillStudioAction::InsertIntoChat(format!("@{} ", ag.name())));
        }
    });

    action
}

#[allow(clippy::too_many_arguments)]
fn render_custom_agent_detail(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    project_root: &Path,
    ag: &DiscoveredAgent,
    skills: &[DiscoveredSkill],
    all_agents: &[DiscoveredAgent],
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        AvatarManager::render_animated_avatar(
            ui,
            avatar_agent_from_key(&ag.avatar_key),
            AgentAvatarState::Idle,
            34.0,
            elapsed_secs,
            theme.accent_pop,
        );
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("@{}", ag.name))
                        .color(theme.text_paper)
                        .strong()
                        .size(14.5),
                );
                Frame::none()
                    .fill(theme.surface_2)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(ag.scope.label())
                                .color(theme.text_silver)
                                .size(9.8),
                        );
                    });
            });
            ui.label(
                egui::RichText::new(&ag.role)
                    .color(theme.text_silver)
                    .size(11.5),
            );
        });
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        for (t, icon, lbl) in [
            (SkillDetailTab::Preview, Lucide::EYE, "Xem"),
            (SkillDetailTab::Edit, Lucide::PENCIL, "Sửa"),
        ] {
            let sel = state.detail_tab == t;
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} {}", icon, lbl))
                            .color(if sel {
                                theme.text_paper
                            } else {
                                theme.text_silver
                            })
                            .strong()
                            .size(11.0),
                    )
                    .fill(if sel {
                        theme.surface_3
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(5.0)),
                )
                .clicked()
            {
                state.detail_tab = t;
            }
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);

    let body_h = (ui.available_height() - 48.0).max(150.0);
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), body_h),
        Layout::top_down(Align::Min),
        |ui| match state.detail_tab {
            SkillDetailTab::Preview | SkillDetailTab::Files => {
                if let Some(ref m) = ag.model {
                    ui.label(
                        egui::RichText::new(format!("Model ưu tiên: {}", m))
                            .color(theme.info)
                            .size(10.8),
                    );
                    ui.add_space(4.0);
                }
                if !ag.default_skills.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("Skills mặc định:")
                                .color(theme.text_silver)
                                .size(10.5),
                        );
                        for s in &ag.default_skills {
                            Frame::none()
                                .fill(theme.surface_2)
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::symmetric(5.0, 2.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("${}", s))
                                            .color(theme.accent_pop)
                                            .size(10.0),
                                    );
                                });
                        }
                    });
                    ui.add_space(6.0);
                }
                render_markdown_preview(ui, theme, &ag.body);
            }
            SkillDetailTab::Edit => {
                if let Some(a) = render_agent_editor_form(
                    ui,
                    theme,
                    elapsed_secs,
                    project_root,
                    Some(ag),
                    skills,
                    all_agents,
                    state,
                ) {
                    action = Some(a);
                }
            }
        },
    );

    if state.confirm_delete {
        ui.add_space(4.0);
        Frame::none()
            .fill(Color32::from_rgba_unmultiplied(
                theme.danger.r(),
                theme.danger.g(),
                theme.danger.b(),
                26,
            ))
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, theme.danger))
            .inner_margin(Margin::symmetric(8.0, 5.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Chuyển agent '@{}' vào thùng rác?", ag.name))
                            .color(theme.danger)
                            .size(10.5),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Xác nhận xóa")
                                        .color(Color32::WHITE)
                                        .strong()
                                        .size(10.5),
                                )
                                .fill(theme.danger)
                                .rounding(Rounding::same(4.0)),
                            )
                            .clicked()
                        {
                            match SkillDiscovery::trash_agent(&ag.path) {
                                Ok(trashed) => {
                                    state.undo_trashed = Some((trashed, Instant::now()));
                                    state.confirm_delete = false;
                                    state.selected =
                                        SelectedItem::CoreAgent(ElegansAgent::SeniorDev);
                                    action = Some(SkillStudioAction::RefreshWorkspace);
                                }
                                Err(e) => state.set_status(format!("Lỗi xóa: {}", e), true),
                            }
                        }
                        if ui.button("Hủy").clicked() {
                            state.confirm_delete = false;
                        }
                    });
                });
            });
    }

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(format!("{} Triệu hồi trong Chat", Lucide::SPARKLES))
                        .color(Color32::BLACK)
                        .strong()
                        .size(10.8),
                )
                .fill(theme.accent_pop)
                .rounding(Rounding::same(5.0)),
            )
            .clicked()
        {
            action = Some(SkillStudioAction::InsertIntoChat(format!("@{} ", ag.name)));
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} Xóa", Lucide::TRASH))
                            .color(theme.danger)
                            .size(10.8),
                    )
                    .fill(Color32::TRANSPARENT),
                )
                .clicked()
            {
                state.confirm_delete = !state.confirm_delete;
            }
        });
    });

    action
}

#[allow(clippy::too_many_arguments)]
fn render_agent_editor_form(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    project_root: &Path,
    existing: Option<&DiscoveredAgent>,
    skills: &[DiscoveredSkill],
    all_agents: &[DiscoveredAgent],
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action = None;
    let draft = &mut state.agent_draft;

    if existing.is_none() {
        ui.label(
            egui::RichText::new(format!("{} Tạo Custom Agent mới", Lucide::BOT))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
        );
        ui.add_space(6.0);
    }

    let slug = SkillDiscovery::sanitize_agent_slug(&draft.name);
    let collision = !slug.is_empty()
        && all_agents.iter().any(|a| {
            a.scope == draft.scope
                && a.name == slug
                && draft.original_path.as_ref() != Some(&a.path)
        });

    ScrollArea::vertical()
        .id_salt("agent_editor_scroll")
        .max_height((ui.available_height() - 40.0).max(140.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Phạm vi:");
                for sc in [SkillScope::Project, SkillScope::Global] {
                    if ui.radio_value(&mut draft.scope, sc, sc.label()).changed() {
                        draft.dirty = true;
                    }
                }
            });
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Tên Agent:");
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut draft.name)
                            .desired_width(180.0)
                            .hint_text("vd: fe_perf_expert"),
                    )
                    .changed()
                {
                    draft.dirty = true;
                }
                ui.label("Model (tùy chọn):");
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut draft.model)
                            .desired_width(140.0)
                            .hint_text("vd: claude-sonnet"),
                    )
                    .changed()
                {
                    draft.dirty = true;
                }
            });

            if collision {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} Agent '{}' đã tồn tại.",
                            Lucide::ALERT_TRIANGLE,
                            slug
                        ))
                        .color(theme.warning)
                        .size(10.2),
                    );
                    ui.checkbox(&mut draft.allow_overwrite, "Cho phép ghi đè");
                });
            }

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Vai trò (Role):");
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut draft.role)
                            .desired_width(ui.available_width())
                            .hint_text("Chuyên gia tối ưu hiệu năng..."),
                    )
                    .changed()
                {
                    draft.dirty = true;
                }
            });

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Avatar Notionists:")
                    .color(theme.text_silver)
                    .size(10.5),
            );
            ui.horizontal_wrapped(|ui| {
                for &ag in ALL_AGENTS {
                    let sel = draft.avatar_key == ag.name();
                    let resp = Frame::none()
                        .fill(if sel {
                            theme.accent_surface
                        } else {
                            theme.surface_2
                        })
                        .rounding(Rounding::same(6.0))
                        .stroke(Stroke::new(
                            1.0,
                            if sel { theme.accent_pop } else { theme.border },
                        ))
                        .inner_margin(Margin::same(4.0))
                        .show(ui, |ui| {
                            AvatarManager::render_animated_avatar(
                                ui,
                                ag,
                                if sel {
                                    AgentAvatarState::Answering
                                } else {
                                    AgentAvatarState::Idle
                                },
                                24.0,
                                elapsed_secs,
                                theme.accent_pop,
                            );
                        });
                    if resp.response.interact(Sense::click()).clicked() {
                        draft.avatar_key = ag.name().to_string();
                        draft.dirty = true;
                    }
                }
            });

            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Skills mặc định:")
                        .color(theme.text_silver)
                        .size(10.5),
                );
                let mut rem = None;
                for (i, s) in draft.default_skills.iter().enumerate() {
                    Frame::none()
                        .fill(theme.surface_2)
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("${}", s))
                                        .color(theme.accent_pop)
                                        .size(10.0),
                                );
                                if ui.small_button(Lucide::X).clicked() {
                                    rem = Some(i);
                                }
                            });
                        });
                }
                if let Some(i) = rem {
                    draft.default_skills.remove(i);
                    draft.dirty = true;
                }
                ui.menu_button(format!("{} Thêm skill", Lucide::PLUS), |ui| {
                    for sk in skills.iter().filter(|s| s.enabled) {
                        if !draft.default_skills.contains(&sk.name)
                            && ui.button(format!("${}", sk.name)).clicked()
                        {
                            draft.default_skills.push(sk.name.clone());
                            draft.dirty = true;
                            ui.close_menu();
                        }
                    }
                });
            });

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("System Prompt:")
                    .color(theme.text_silver)
                    .size(10.5),
            );
            if ui
                .add(
                    egui::TextEdit::multiline(&mut draft.body)
                        .desired_rows(9)
                        .desired_width(ui.available_width())
                        .code_editor(),
                )
                .changed()
            {
                draft.dirty = true;
            }
        });

    ui.add_space(8.0);
    let can_save = !slug.is_empty() && (!collision || draft.allow_overwrite) && draft.dirty;

    ui.horizontal(|ui| {
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("Hủy")
                        .color(theme.text_silver)
                        .size(10.8),
                )
                .fill(theme.surface_2)
                .rounding(Rounding::same(5.0)),
            )
            .clicked()
        {
            if let Some(ag) = existing {
                state.load_agent_into_draft(ag);
                state.detail_tab = SkillDetailTab::Preview;
            } else {
                state.agent_draft.dirty = false;
                state.selected = SelectedItem::CoreAgent(ElegansAgent::SeniorDev);
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add_enabled(
                    can_save,
                    egui::Button::new(
                        egui::RichText::new(format!("{} Lưu Agent", Lucide::SAVE))
                            .color(Color32::BLACK)
                            .strong()
                            .size(11.0),
                    )
                    .fill(theme.accent_pop)
                    .rounding(Rounding::same(5.0)),
                )
                .clicked()
            {
                let res = SkillDiscovery::save_agent(
                    project_root,
                    state.agent_draft.scope,
                    &state.agent_draft.name,
                    &state.agent_draft.role,
                    &state.agent_draft.avatar_key,
                    Some(&state.agent_draft.model),
                    &state.agent_draft.default_skills,
                    &state.agent_draft.body,
                    state.agent_draft.original_path.as_deref(),
                    state.agent_draft.allow_overwrite,
                );
                match res {
                    Ok(saved_path) => {
                        state.agent_draft.original_path = Some(saved_path.clone());
                        state.agent_draft.dirty = false;
                        state.agent_draft.allow_overwrite = false;
                        state.selected = SelectedItem::CustomAgentPath(saved_path);
                        state.detail_tab = SkillDetailTab::Preview;
                        state.set_status(format!("Đã lưu agent '@{}'.", slug), false);
                        action = Some(SkillStudioAction::RefreshWorkspace);
                    }
                    Err(e) => state.set_status(format!("Lỗi lưu agent: {}", e), true),
                }
            }
        });
    });

    action
}

fn render_markdown_preview(ui: &mut Ui, theme: &CanhTheme, md: &str) {
    ScrollArea::vertical()
        .id_salt("studio_md_preview_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let mut in_code_block = false;
            let mut code_buf = String::new();

            for line in md.lines() {
                if line.trim_start().starts_with("```") {
                    if in_code_block {
                        Frame::none()
                            .fill(theme.bg_field)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(code_buf.trim_end())
                                        .color(theme.text_paper)
                                        .monospace()
                                        .size(10.5),
                                );
                            });
                        ui.add_space(4.0);
                        code_buf.clear();
                        in_code_block = false;
                    } else {
                        in_code_block = true;
                    }
                    continue;
                }

                if in_code_block {
                    code_buf.push_str(line);
                    code_buf.push('\n');
                    continue;
                }

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    ui.add_space(5.0);
                } else if let Some(h1) = trimmed.strip_prefix("# ") {
                    ui.label(
                        egui::RichText::new(h1)
                            .color(theme.text_paper)
                            .strong()
                            .size(14.5),
                    );
                    ui.add_space(3.0);
                } else if let Some(h2) = trimmed.strip_prefix("## ") {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(h2)
                            .color(theme.accent_pop)
                            .strong()
                            .size(12.5),
                    );
                } else if let Some(h3) = trimmed.strip_prefix("### ") {
                    ui.add_space(3.0);
                    ui.label(
                        egui::RichText::new(h3)
                            .color(theme.text_paper)
                            .strong()
                            .size(11.8),
                    );
                } else if let Some(bullet) = trimmed
                    .strip_prefix("- ")
                    .or_else(|| trimmed.strip_prefix("* "))
                {
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("•").color(theme.accent_pop).size(11.5));
                        ui.label(
                            egui::RichText::new(bullet)
                                .color(theme.text_paper)
                                .size(11.4),
                        );
                    });
                } else {
                    ui.label(
                        egui::RichText::new(line)
                            .color(theme.text_paper)
                            .size(11.4),
                    );
                }
            }

            if in_code_block && !code_buf.is_empty() {
                Frame::none()
                    .fill(theme.bg_field)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(code_buf.trim_end())
                                .color(theme.text_paper)
                                .monospace()
                                .size(10.5),
                        );
                    });
            }
        });
}

fn render_github_wizard_modal(
    ctx: &egui::Context,
    theme: &CanhTheme,
    project_root: &Path,
    state: &mut SkillStudioState,
) -> Option<SkillStudioAction> {
    let mut action = None;
    let mut open = state.wizard.open;

    egui::Window::new(format!("{} Nạp Skill từ GitHub", Lucide::DOWNLOAD))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .frame(
            Frame::none()
                .fill(theme.panel_bg)
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(1.0, theme.border_strong))
                .inner_margin(Margin::same(14.0)),
        )
        .show(ctx, |ui| {
            // Stepper Header
            ui.horizontal(|ui| {
                for (step, num, lbl) in [
                    (WizardStep::Source, "1", "Nguồn"),
                    (WizardStep::SelectSkills, "2", "Chọn skill"),
                    (WizardStep::Install, "3", "Cài đặt"),
                ] {
                    let active = state.wizard.step == step;
                    let done = (state.wizard.step as u8) > (step as u8);
                    let col = if active {
                        theme.accent_pop
                    } else if done {
                        theme.success
                    } else {
                        theme.text_silver
                    };
                    Frame::none()
                        .fill(if active {
                            theme.accent_surface
                        } else {
                            theme.surface_2
                        })
                        .rounding(Rounding::same(10.0))
                        .inner_margin(Margin::symmetric(7.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("{}. {}", num, lbl))
                                    .color(col)
                                    .strong()
                                    .size(10.5),
                            );
                        });
                    if step != WizardStep::Install {
                        ui.label(
                            egui::RichText::new(Lucide::CHEVRON_RIGHT)
                                .color(theme.text_silver)
                                .size(10.0),
                        );
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Error Banner
            if let Some(ref err) = state.wizard.error_msg {
                Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(
                        theme.danger.r(),
                        theme.danger.g(),
                        theme.danger.b(),
                        26,
                    ))
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0, theme.danger))
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(err).color(theme.danger).size(10.8));
                    });
                ui.add_space(8.0);
            }

            // Busy Progress Indicator
            if state.wizard.is_busy {
                ui.label(
                    egui::RichText::new(&state.wizard.progress_msg)
                        .color(theme.accent_pop)
                        .size(11.0),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::ProgressBar::new(state.wizard.progress_pct)
                        .desired_height(6.0)
                        .fill(theme.accent_pop),
                );
                ui.add_space(10.0);
            }

            match state.wizard.step {
                WizardStep::Source => {
                    ui.label(
                        egui::RichText::new("URL repository GitHub")
                            .color(theme.text_paper)
                            .strong()
                            .size(11.2),
                    );
                    ui.add_space(3.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut state.wizard.repo_url)
                            .desired_width(ui.available_width())
                            .hint_text("https://github.com/anthropics/skills hoặc owner/repo"),
                    );
                    ui.add_space(3.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("Hỗ trợ: owner/repo · URL /tree/<branch>/<subpath> ·")
                                .color(theme.text_silver)
                                .size(9.8),
                        );
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Thử anthropics/skills")
                                        .color(theme.info)
                                        .size(9.8),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            state.wizard.repo_url = "https://github.com/anthropics/skills".to_string();
                        }
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Nhánh / Tag (tùy chọn):")
                                .color(theme.text_silver)
                                .size(10.8),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut state.wizard.branch_override)
                                .desired_width(140.0)
                                .hint_text("mặc định HEAD"),
                        );
                        ui.checkbox(&mut state.wizard.is_private, "Repo private (dùng token)");
                    });

                    if state.wizard.is_private {
                        ui.add_space(6.0);
                        let has_env = std::env::var("GITHUB_TOKEN")
                            .map(|v| !v.trim().is_empty())
                            .unwrap_or(false);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("GitHub Token:")
                                    .color(theme.text_silver)
                                    .size(10.5),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut state.wizard.token)
                                    .password(true)
                                    .desired_width(280.0)
                                    .hint_text(if has_env {
                                        "Để trống sẽ dùng biến môi trường GITHUB_TOKEN"
                                    } else {
                                        "ghp_... (chỉ giữ trên RAM, không lưu ra đĩa)"
                                    }),
                            );
                        });
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Hủy").clicked() {
                            state.wizard.open = false;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let can_scan =
                                !state.wizard.is_busy && !state.wizard.repo_url.trim().is_empty();
                            if ui
                                .add_enabled(
                                    can_scan,
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "Quét repo {}",
                                            Lucide::ARROW_RIGHT
                                        ))
                                        .color(Color32::BLACK)
                                        .strong()
                                        .size(11.0),
                                    )
                                    .fill(theme.accent_pop)
                                    .rounding(Rounding::same(5.0)),
                                )
                                .clicked()
                            {
                                state.wizard.is_busy = true;
                                state.wizard.error_msg = None;
                                state.wizard.progress_msg = "Đang kết nối GitHub...".to_string();
                                state.wizard.progress_pct = 0.1;
                                let tok = if state.wizard.is_private
                                    && !state.wizard.token.trim().is_empty()
                                {
                                    Some(state.wizard.token.trim().to_string())
                                } else {
                                    None
                                };
                                state.wizard.rx = Some(spawn_scan_github_repo(
                                    state.wizard.repo_url.clone(),
                                    state.wizard.branch_override.clone(),
                                    tok,
                                    project_root.to_path_buf(),
                                ));
                            }
                        });
                    });
                }

                WizardStep::SelectSkills => {
                    if let Some(ref mut scan) = state.wizard.scan_result {
                        let short_sha = &scan.commit_sha[..7.min(scan.commit_sha.len())];
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Tìm thấy {} skill trong {} ({}@{})",
                                    scan.candidates.len(),
                                    scan.source.repo_slug(),
                                    scan.resolved_ref,
                                    short_sha
                                ))
                                .color(theme.text_paper)
                                .strong()
                                .size(11.2),
                            );
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            let all_selected = scan.candidates.iter().all(|c| c.selected);
                            let mut sel_all = all_selected;
                            if ui.checkbox(&mut sel_all, "Chọn tất cả").changed() {
                                for c in &mut scan.candidates {
                                    c.selected = sel_all;
                                }
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.wizard.filter_query)
                                        .desired_width(170.0)
                                        .hint_text("Lọc theo tên..."),
                                );
                            });
                        });

                        ui.add_space(6.0);
                        let fq = state.wizard.filter_query.trim().to_lowercase();

                        ScrollArea::vertical()
                            .id_salt("wizard_candidates_scroll")
                            .max_height(260.0)
                            .show(ui, |ui| {
                                for (idx, cand) in scan.candidates.iter_mut().enumerate() {
                                    if !fq.is_empty()
                                        && !cand.name.to_lowercase().contains(&fq)
                                        && !cand.description.to_lowercase().contains(&fq)
                                    {
                                        continue;
                                    }

                                    Frame::none()
                                        .fill(theme.surface_2)
                                        .rounding(Rounding::same(6.0))
                                        .stroke(Stroke::new(
                                            1.0,
                                            if cand.selected {
                                                theme.border_strong
                                            } else {
                                                theme.border
                                            },
                                        ))
                                        .inner_margin(Margin::symmetric(8.0, 6.0))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.checkbox(&mut cand.selected, "");
                                                ui.label(
                                                    egui::RichText::new(&cand.name)
                                                        .color(theme.text_paper)
                                                        .strong()
                                                        .size(11.5),
                                                );
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{} file",
                                                        cand.file_count
                                                    ))
                                                    .color(theme.text_silver)
                                                    .size(9.8),
                                                );
                                                if cand.has_scripts {
                                                    Frame::none()
                                                        .fill(Color32::from_rgba_unmultiplied(
                                                            theme.warning.r(),
                                                            theme.warning.g(),
                                                            theme.warning.b(),
                                                            28,
                                                        ))
                                                        .rounding(Rounding::same(3.0))
                                                        .inner_margin(Margin::symmetric(4.0, 1.0))
                                                        .show(ui, |ui| {
                                                            ui.label(
                                                                egui::RichText::new(format!(
                                                                    "{} scripts/",
                                                                    Lucide::SHIELD_ALERT
                                                                ))
                                                                .color(theme.warning)
                                                                .size(9.4),
                                                            );
                                                        });
                                                }
                                                if cand.already_exists_project
                                                    || cand.already_exists_global
                                                {
                                                    ui.label(
                                                        egui::RichText::new("(đã có)")
                                                            .color(theme.warning)
                                                            .size(9.8),
                                                    );
                                                }

                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        let is_prev =
                                                            state.wizard.preview_candidate_idx
                                                                == Some(idx);
                                                        if ui
                                                            .small_button(if is_prev {
                                                                "Ẩn xem trước"
                                                            } else {
                                                                "Xem trước"
                                                            })
                                                            .clicked()
                                                        {
                                                            state.wizard.preview_candidate_idx =
                                                                if is_prev { None } else { Some(idx) };
                                                        }
                                                    },
                                                );
                                            });

                                            ui.label(
                                                egui::RichText::new(&cand.description)
                                                    .color(theme.text_silver)
                                                    .size(10.3),
                                            );

                                            if state.wizard.preview_candidate_idx == Some(idx) {
                                                ui.add_space(4.0);
                                                Frame::none()
                                                    .fill(theme.bg_field)
                                                    .rounding(Rounding::same(4.0))
                                                    .inner_margin(Margin::same(6.0))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            egui::RichText::new(&cand.preview_md)
                                                                .color(theme.text_paper)
                                                                .monospace()
                                                                .size(9.8),
                                                        );
                                                    });
                                            }
                                        });
                                    ui.add_space(5.0);
                                }
                            });

                        let selected_count = scan.candidates.iter().filter(|c| c.selected).count();
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button(format!("{} Quay lại", Lucide::ARROW_LEFT)).clicked() {
                                state.wizard.step = WizardStep::Source;
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .add_enabled(
                                        selected_count > 0,
                                        egui::Button::new(
                                            egui::RichText::new(format!(
                                                "Tiếp tục ({}) {}",
                                                selected_count,
                                                Lucide::ARROW_RIGHT
                                            ))
                                            .color(Color32::BLACK)
                                            .strong()
                                            .size(11.0),
                                        )
                                        .fill(theme.accent_pop)
                                        .rounding(Rounding::same(5.0)),
                                    )
                                    .clicked()
                                {
                                    state.wizard.step = WizardStep::Install;
                                }
                            });
                        });
                    }
                }

                WizardStep::Install => {
                    let (selected_count, script_count, conflict_count) =
                        if let Some(ref scan) = state.wizard.scan_result {
                            let sel: Vec<_> =
                                scan.candidates.iter().filter(|c| c.selected).collect();
                            let sc = sel.iter().filter(|c| c.has_scripts).count();
                            let cc = sel
                                .iter()
                                .filter(|c| match state.wizard.target_scope {
                                    SkillScope::Project => c.already_exists_project,
                                    SkillScope::Global => c.already_exists_global,
                                })
                                .count();
                            (sel.len(), sc, cc)
                        } else {
                            (0, 0, 0)
                        };

                    ui.label(
                        egui::RichText::new("Nơi cài đặt")
                            .color(theme.text_paper)
                            .strong()
                            .size(11.2),
                    );
                    ui.add_space(4.0);
                    ui.radio_value(
                        &mut state.wizard.target_scope,
                        SkillScope::Project,
                        format!(
                            "Project hiện tại ({})",
                            project_root.join("skills").to_string_lossy()
                        ),
                    );
                    ui.radio_value(
                        &mut state.wizard.target_scope,
                        SkillScope::Global,
                        format!(
                            "Global dùng chung cho mọi project ({})",
                            SkillDiscovery::global_skills_dir().to_string_lossy()
                        ),
                    );

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Khi trùng tên skill:")
                                .color(theme.text_paper)
                                .size(11.0),
                        );
                        egui::ComboBox::from_id_salt("wizard_conflict_cb")
                            .selected_text(state.wizard.conflict_strategy.label())
                            .width(230.0)
                            .show_ui(ui, |ui| {
                                for strat in [
                                    ConflictStrategy::Skip,
                                    ConflictStrategy::Overwrite,
                                    ConflictStrategy::Rename,
                                ] {
                                    ui.selectable_value(
                                        &mut state.wizard.conflict_strategy,
                                        strat,
                                        strat.label(),
                                    );
                                }
                            });
                    });

                    if conflict_count > 0 {
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "Có {} skill trùng tên trong đích đã chọn.",
                                conflict_count
                            ))
                            .color(theme.warning)
                            .size(10.5),
                        );
                    }

                    if script_count > 0 {
                        ui.add_space(8.0);
                        Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(
                                theme.warning.r(),
                                theme.warning.g(),
                                theme.warning.b(),
                                24,
                            ))
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.warning))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} {} skill chứa file script thực thi. Victini không bao giờ tự chạy script khi cài — bạn nên kiểm tra lại trong tab File sau khi nạp.",
                                        Lucide::SHIELD_ALERT,
                                        script_count
                                    ))
                                    .color(theme.warning)
                                    .size(10.5),
                                );
                            });
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !state.wizard.is_busy,
                                egui::Button::new(format!("{} Quay lại", Lucide::ARROW_LEFT)),
                            )
                            .clicked()
                        {
                            state.wizard.step = WizardStep::SelectSkills;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add_enabled(
                                    !state.wizard.is_busy && selected_count > 0,
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} Cài đặt {} skill",
                                            Lucide::DOWNLOAD,
                                            selected_count
                                        ))
                                        .color(Color32::BLACK)
                                        .strong()
                                        .size(11.0),
                                    )
                                    .fill(theme.accent_pop)
                                    .rounding(Rounding::same(5.0)),
                                )
                                .clicked()
                            {
                                if let Some(scan) = state.wizard.scan_result.clone() {
                                    state.wizard.is_busy = true;
                                    state.wizard.error_msg = None;
                                    state.wizard.progress_msg = "Đang sao chép skill...".to_string();
                                    state.wizard.progress_pct = 0.2;
                                    state.wizard.rx = Some(spawn_install_candidates(
                                        scan,
                                        state.wizard.target_scope,
                                        state.wizard.conflict_strategy,
                                        project_root.to_path_buf(),
                                    ));
                                }
                            }
                        });
                    });
                }

                WizardStep::Done => {
                    if let Some(ref rep) = state.wizard.install_report {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Hoàn tất nạp {} skill từ GitHub",
                                Lucide::CIRCLE_CHECK,
                                rep.installed.len()
                            ))
                            .color(theme.success)
                            .strong()
                            .size(13.0),
                        );
                        ui.add_space(6.0);
                        if !rep.installed.is_empty() {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Đã cài: {}",
                                    rep.installed.join(", ")
                                ))
                                .color(theme.text_paper)
                                .size(11.0),
                            );
                        }
                        if !rep.renamed.is_empty() {
                            let pairs: Vec<String> = rep
                                .renamed
                                .iter()
                                .map(|(a, b)| format!("{} → {}", a, b))
                                .collect();
                            ui.label(
                                egui::RichText::new(format!("Đổi tên: {}", pairs.join(", ")))
                                    .color(theme.info)
                                    .size(10.8),
                            );
                        }
                        if !rep.skipped.is_empty() {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Đã bỏ qua (trùng tên): {}",
                                    rep.skipped.join(", ")
                                ))
                                .color(theme.text_silver)
                                .size(10.8),
                            );
                        }
                    }

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Mở trong Studio")
                                            .color(Color32::BLACK)
                                            .strong()
                                            .size(11.0),
                                    )
                                    .fill(theme.accent_pop)
                                    .rounding(Rounding::same(5.0)),
                                )
                                .clicked()
                            {
                                state.wizard.open = false;
                                action = Some(SkillStudioAction::RefreshWorkspace);
                            }
                        });
                    });
                }
            }
        });

    if !open {
        state.wizard.open = false;
    }

    action
}
