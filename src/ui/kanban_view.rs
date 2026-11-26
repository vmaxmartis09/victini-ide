use crate::ai::{
    AgentAvatarState, AgentKanbanManager, AgentTaskCard, AvatarManager, ElegansAgent, KanbanColumn,
    QuotedContext, ALL_AGENTS,
};
use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::theme::{CanhTheme, Lucide};
use egui::{
    vec2, Align, Color32, Frame, Id, Layout, Margin, Rect, Rounding, ScrollArea, Sense, Stroke, Ui,
};
use std::collections::HashMap;
use std::time::Instant;

pub enum KanbanAction {
    CreateTask {
        title: String,
        description: String,
        priority: String,
        agent: ElegansAgent,
        custom_agent: Option<String>,
        attached_skills: Vec<String>,
        branch: String,
        column: KanbanColumn,
        auto_schedule_idle: bool,
        run_now: bool,
    },
    UpdateTask {
        task_id: String,
        title: String,
        description: String,
        priority: String,
        agent: ElegansAgent,
        custom_agent: Option<String>,
        attached_skills: Vec<String>,
        branch: String,
        column: KanbanColumn,
        auto_schedule_idle: bool,
    },
    ReassignAgent {
        task_id: String,
        next_agent: ElegansAgent,
        custom_agent: Option<String>,
    },
    MoveToColumn {
        task_id: String,
        target_col: KanbanColumn,
        insert_before_id: Option<String>,
    },
    MoveLeft(String),
    AdvanceTask(String),
    ApproveTask(String),
    RejectTask {
        task_id: String,
        note: String,
    },
    DuplicateTask(String),
    RunInBackground(String),
    StopBackground(String),
    QuoteTaskToChat(QuotedContext),
    DeleteTask(String),
    RestoreTask(AgentTaskCard),
}

pub struct KanbanDrawerDraft {
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub agent: ElegansAgent,
    pub custom_agent: Option<String>,
    pub attached_skills: Vec<String>,
    pub branch: String,
    pub column: KanbanColumn,
    pub auto_schedule_idle: bool,
    pub reject_note: String,
    pub auto_scroll_logs: bool,
    pub confirm_delete: bool,
    pub dirty: bool,
}

impl KanbanDrawerDraft {
    pub fn from_card(card: &AgentTaskCard) -> Self {
        Self {
            task_id: card.id.clone(),
            title: card.title.clone(),
            description: card.description.clone(),
            priority: card.priority.clone(),
            agent: card.assigned_agent,
            custom_agent: card.custom_agent_name.clone(),
            attached_skills: card.attached_skills.clone(),
            branch: card.worktree_branch.clone(),
            column: card.column,
            auto_schedule_idle: card.auto_schedule_idle,
            reject_note: card.review_note.clone().unwrap_or_default(),
            auto_scroll_logs: true,
            confirm_delete: false,
            dirty: false,
        }
    }
}

pub struct KanbanComposerState {
    // Modal Create Task state
    pub show_create_modal: bool,
    pub new_title: String,
    pub new_desc: String,
    pub selected_priority: String,
    pub selected_agent: ElegansAgent,
    pub selected_custom_agent: Option<String>,
    pub selected_skills: Vec<String>,
    pub selected_branch: String,
    pub selected_column: KanbanColumn,
    pub auto_schedule_idle: bool,

    // Filters & Search
    pub search_query: String,
    pub filter_priority: Option<String>,
    pub filter_agent: Option<String>,

    // Per-column inline quick add
    pub quick_add_col: Option<KanbanColumn>,
    pub quick_add_text: String,

    // Done column collapse & compact view accordions
    pub done_show_all: bool,
    pub compact_collapsed: HashMap<KanbanColumn, bool>,

    // Right-side Task Drawer
    pub selected_task_id: Option<String>,
    pub drawer_draft: Option<KanbanDrawerDraft>,

    // Custom Drag & Drop state (avoids egui::dnd_drop_zone white background & horizontal layout bug)
    pub dragging_task_id: Option<String>,

    // Delete confirmation (card menu) & 5s Undo Toast
    pub confirm_delete_card_id: Option<String>,
    pub undo_deleted: Option<(AgentTaskCard, Instant)>,

    // Geometry telemetry for layout unit tests
    pub last_column_rects: Vec<Rect>,
    pub last_card_rects: Vec<(String, Rect)>,
    pub expanded_task_id: Option<String>,
}

impl Default for KanbanComposerState {
    fn default() -> Self {
        Self {
            show_create_modal: false,
            new_title: String::new(),
            new_desc: String::new(),
            selected_priority: "P1".to_string(),
            selected_agent: ElegansAgent::SeniorDev,
            selected_custom_agent: None,
            selected_skills: Vec::new(),
            selected_branch: "main".to_string(),
            selected_column: KanbanColumn::Backlog,
            auto_schedule_idle: true,
            search_query: String::new(),
            filter_priority: None,
            filter_agent: None,
            quick_add_col: None,
            quick_add_text: String::new(),
            done_show_all: false,
            compact_collapsed: HashMap::new(),
            selected_task_id: None,
            drawer_draft: None,
            dragging_task_id: None,
            confirm_delete_card_id: None,
            undo_deleted: None,
            last_column_rects: Vec::new(),
            last_card_rects: Vec::new(),
            expanded_task_id: None,
        }
    }
}

impl KanbanComposerState {
    pub fn open_create_modal_prefilled(
        &mut self,
        title: String,
        desc: String,
        agent: ElegansAgent,
        branch: &str,
    ) {
        self.new_title = title;
        self.new_desc = desc;
        self.selected_priority = "P1".to_string();
        self.selected_agent = agent;
        self.selected_custom_agent = None;
        self.selected_skills.clear();
        self.selected_branch = branch.to_string();
        self.selected_column = KanbanColumn::Backlog;
        self.auto_schedule_idle = true;
        self.show_create_modal = true;
    }

    pub fn open_drawer_for(&mut self, card: &AgentTaskCard) {
        self.selected_task_id = Some(card.id.clone());
        self.drawer_draft = Some(KanbanDrawerDraft::from_card(card));
    }

    pub fn record_deleted_for_undo(&mut self, card: AgentTaskCard) {
        if self.selected_task_id.as_deref() == Some(card.id.as_str()) {
            self.selected_task_id = None;
            self.drawer_draft = None;
        }
        self.undo_deleted = Some((card, Instant::now()));
    }
}

fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }
}

fn priority_color(prio: &str, theme: &CanhTheme) -> Color32 {
    match prio {
        "P0" => theme.danger,
        "P1" => theme.accent_pop,
        _ => theme.info,
    }
}

fn column_accent(col: KanbanColumn, theme: &CanhTheme) -> Color32 {
    match col {
        KanbanColumn::Backlog => theme.text_silver,
        KanbanColumn::InProgress => theme.accent_pop,
        KanbanColumn::Verifying => theme.info,
        KanbanColumn::Done => theme.success,
    }
}

fn matches_filters(task: &AgentTaskCard, composer: &KanbanComposerState) -> bool {
    if let Some(ref prio) = composer.filter_priority {
        if &task.priority != prio {
            return false;
        }
    }
    if let Some(ref ag) = composer.filter_agent {
        if task.display_agent_name() != ag && task.assigned_agent.name() != ag {
            return false;
        }
    }
    let q = composer.search_query.trim().to_lowercase();
    if !q.is_empty() {
        let hit = task.id.to_lowercase().contains(&q)
            || task.title.to_lowercase().contains(&q)
            || task.description.to_lowercase().contains(&q)
            || task.display_agent_name().to_lowercase().contains(&q)
            || task.worktree_branch.to_lowercase().contains(&q);
        if !hit {
            return false;
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
pub fn render_agent_kanban_view(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    kanban: &mut AgentKanbanManager,
    composer: &mut KanbanComposerState,
    skills: &[DiscoveredSkill],
    custom_agents: &[DiscoveredAgent],
    branches: &[String],
    current_branch: &str,
) -> Option<KanbanAction> {
    let mut action: Option<KanbanAction> = None;
    composer.last_column_rects.clear();
    composer.last_card_rects.clear();

    // Expire 5-second undo toast
    if let Some((_, ts)) = &composer.undo_deleted {
        if ts.elapsed().as_secs_f32() > 5.5 {
            composer.undo_deleted = None;
        }
    }

    // Keyboard shortcuts inside Kanban view
    ui.ctx().input(|i| {
        if i.modifiers.command && i.key_pressed(egui::Key::N) && !composer.show_create_modal {
            composer.open_create_modal_prefilled(
                String::new(),
                String::new(),
                ElegansAgent::SeniorDev,
                current_branch,
            );
        }
        if i.key_pressed(egui::Key::Escape) {
            if composer.show_create_modal {
                composer.show_create_modal = false;
            } else if composer.selected_task_id.is_some() {
                composer.selected_task_id = None;
                composer.drawer_draft = None;
            } else if composer.quick_add_col.is_some() {
                composer.quick_add_col = None;
            }
        }
    });

    // Keep drawer synced if selected task was deleted or updated externally
    if let Some(ref sel_id) = composer.selected_task_id.clone() {
        if let Some(card) = kanban.tasks.iter().find(|t| &t.id == sel_id) {
            let need_init = composer
                .drawer_draft
                .as_ref()
                .map(|d| d.task_id != card.id || !d.dirty)
                .unwrap_or(true);
            if need_init {
                let prev_auto = composer
                    .drawer_draft
                    .as_ref()
                    .map(|d| d.auto_scroll_logs)
                    .unwrap_or(true);
                let prev_confirm = composer
                    .drawer_draft
                    .as_ref()
                    .map(|d| d.confirm_delete)
                    .unwrap_or(false);
                let mut draft = KanbanDrawerDraft::from_card(card);
                draft.auto_scroll_logs = prev_auto;
                draft.confirm_delete = prev_confirm;
                composer.drawer_draft = Some(draft);
            }
        } else {
            composer.selected_task_id = None;
            composer.drawer_draft = None;
        }
    }

    ui.vertical(|ui| {
        // 1. TOP ORCHESTRATOR & FILTER BAR
        render_top_bar(
            ui,
            theme,
            kanban,
            composer,
            custom_agents,
            current_branch,
        );

        // Optional Undo Delete Toast Banner
        if let Some((deleted_card, ts)) = composer.undo_deleted.clone() {
            let remaining = (5.5 - ts.elapsed().as_secs_f32()).max(0.0);
            ui.add_space(6.0);
            Frame::none()
                .fill(theme.surface_2)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border_strong))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Đã xóa {} ({}) · Hoàn tác trong {:.0}s",
                                Lucide::TRASH,
                                deleted_card.id,
                                deleted_card.title,
                                remaining.ceil()
                            ))
                            .color(theme.text_paper)
                            .size(11.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!("{} Hoàn tác", Lucide::UNDO))
                                            .color(theme.text_on_accent())
                                            .strong()
                                            .size(10.5),
                                    )
                                    .fill(theme.accent_pop)
                                    .rounding(Rounding::same(4.0)),
                                )
                                .clicked()
                            {
                                composer.undo_deleted = None;
                                action = Some(KanbanAction::RestoreTask(deleted_card));
                            }
                        });
                    });
                });
            ui.ctx().request_repaint();
        }

        ui.add_space(8.0);

        // 2. MAIN WORKSPACE: BOARD (LEFT) + OPTIONAL TASK DRAWER (RIGHT)
        let total_w = ui.available_width();
        let drawer_open = composer.selected_task_id.is_some();
        let drawer_w = if drawer_open {
            (total_w * 0.35).clamp(320.0, 410.0)
        } else {
            0.0
        };
        let board_w = (total_w - if drawer_open { drawer_w + 10.0 } else { 0.0 }).max(260.0);
        let board_h = (ui.available_height() - 4.0).max(240.0);

        ui.horizontal_top(|ui| {
            // Left: Kanban Board (either Compact Vertical or 4-Column Board)
            ui.allocate_ui_with_layout(
                vec2(board_w, board_h),
                Layout::top_down(Align::Min),
                |ui| {
                    if board_w < 780.0 {
                        if let Some(act) = render_compact_board(
                            ui,
                            theme,
                            elapsed_secs,
                            kanban,
                            composer,
                            custom_agents,
                            current_branch,
                        ) {
                            action = Some(act);
                        }
                    } else if let Some(act) = render_four_column_board(
                        ui,
                        theme,
                        elapsed_secs,
                        kanban,
                        composer,
                        custom_agents,
                        current_branch,
                        board_w,
                        board_h,
                    ) {
                        action = Some(act);
                    }
                },
            );

            // Right: Task Detail Drawer
            if drawer_open {
                ui.add_space(8.0);
                ui.allocate_ui_with_layout(
                    vec2(drawer_w, board_h),
                    Layout::top_down(Align::Min),
                    |ui| {
                        if let Some(act) = render_task_drawer(
                            ui,
                            theme,
                            elapsed_secs,
                            kanban,
                            composer,
                            skills,
                            custom_agents,
                            branches,
                        ) {
                            action = Some(act);
                        }
                    },
                );
            }
        });
    });

    // Clear dragging state when pointer is released
    if composer.dragging_task_id.is_some() {
        let pointer_down = ui.input(|i| i.pointer.any_down());
        if let Some(ref drag_id) = composer.dragging_task_id {
            egui::show_tooltip_at_pointer(
                ui.ctx(),
                egui::LayerId::new(egui::Order::Tooltip, Id::new("kanban_drag_badge")),
                Id::new("kanban_drag_preview"),
                |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} Đang kéo {}", Lucide::KANBAN, drag_id))
                            .color(theme.accent_pop)
                            .strong()
                            .size(11.0),
                    );
                },
            );
        }
        if !pointer_down {
            composer.dragging_task_id = None;
        }
    }

    // 3. CREATE TASK MODAL
    if composer.show_create_modal {
        if let Some(act) = render_create_task_modal(
            ui.ctx(),
            theme,
            composer,
            skills,
            custom_agents,
            branches,
            current_branch,
        ) {
            action = Some(act);
        }
    }

    action
}

fn render_top_bar(
    ui: &mut Ui,
    theme: &CanhTheme,
    kanban: &mut AgentKanbanManager,
    composer: &mut KanbanComposerState,
    custom_agents: &[DiscoveredAgent],
    current_branch: &str,
) {
    let running_count = kanban.tasks.iter().filter(|t| t.is_running_bg).count();

    Frame::none()
        .fill(theme.surface_1)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                // Title + Running counter
                ui.label(
                    egui::RichText::new(format!("{} Swarm Kanban", Lucide::KANBAN))
                        .color(theme.text_paper)
                        .strong()
                        .size(13.5),
                );
                if running_count > 0 {
                    Frame::none()
                        .fill(theme.accent_surface)
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("{} đang chạy", running_count))
                                    .color(theme.accent_pop)
                                    .strong()
                                    .size(10.5),
                            );
                        });
                }

                ui.add_space(6.0);

                // Search Box
                Frame::none()
                    .fill(theme.input_bg)
                    .rounding(Rounding::same(6.0))
                    .stroke(Stroke::new(1.0, theme.border_strong))
                    .inner_margin(Margin::symmetric(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(Lucide::SEARCH)
                                    .color(theme.text_silver)
                                    .size(11.0),
                            );
                            ui.add_sized(
                                vec2(140.0, 18.0),
                                egui::TextEdit::singleline(&mut composer.search_query)
                                    .hint_text("Tìm task...")
                                    .frame(false)
                                    .text_color(theme.text_paper),
                            );
                            if !composer.search_query.is_empty()
                                && ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(Lucide::X)
                                                .color(theme.text_silver)
                                                .size(10.0),
                                        )
                                        .fill(Color32::TRANSPARENT),
                                    )
                                    .clicked()
                            {
                                composer.search_query.clear();
                            }
                        });
                    });

                // Agent Filter Menu Button (consistent warm surface, no OS ComboBox white glitch)
                let current_ag_label = composer
                    .filter_agent
                    .as_deref()
                    .unwrap_or("Tất cả Agent");
                ui.menu_button(
                    egui::RichText::new(format!(
                        "{} {} {}",
                        Lucide::BOT,
                        current_ag_label,
                        Lucide::CHEVRON_DOWN
                    ))
                    .color(theme.text_paper)
                    .size(10.8),
                    |ui| {
                        if ui
                            .selectable_label(composer.filter_agent.is_none(), "Tất cả Agent")
                            .clicked()
                        {
                            composer.filter_agent = None;
                            ui.close_menu();
                        }
                        ui.separator();
                        for &ag in ALL_AGENTS {
                            let selected = composer.filter_agent.as_deref() == Some(ag.name());
                            if ui
                                .selectable_label(
                                    selected,
                                    format!("{} {} — {}", ag.icon(), ag.name(), ag.title()),
                                )
                                .clicked()
                            {
                                composer.filter_agent = Some(ag.name().to_string());
                                ui.close_menu();
                            }
                        }
                        if !custom_agents.is_empty() {
                            ui.separator();
                            for cag in custom_agents {
                                let selected =
                                    composer.filter_agent.as_deref() == Some(cag.name.as_str());
                                if ui
                                    .selectable_label(selected, format!("@{}", cag.name))
                                    .clicked()
                                {
                                    composer.filter_agent = Some(cag.name.clone());
                                    ui.close_menu();
                                }
                            }
                        }
                    },
                );

                // Priority Filter Pills
                for prio in [None, Some("P0"), Some("P1"), Some("P2")] {
                    let label = prio.unwrap_or("Tất cả");
                    let is_sel = composer.filter_priority.as_deref() == prio;
                    let fill = if is_sel {
                        theme.surface_3
                    } else {
                        Color32::TRANSPARENT
                    };
                    let fg = if is_sel {
                        prio.map(|p| priority_color(p, theme))
                            .unwrap_or(theme.text_paper)
                    } else {
                        theme.text_silver
                    };
                    if ui
                        .add(
                            egui::Button::new(egui::RichText::new(label).color(fg).size(10.5))
                                .fill(fill)
                                .stroke(Stroke::new(
                                    1.0,
                                    if is_sel {
                                        theme.border_strong
                                    } else {
                                        Color32::TRANSPARENT
                                    },
                                ))
                                .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        composer.filter_priority = prio.map(|s| s.to_string());
                    }
                }

                // Auto-run Idle Toggle
                ui.checkbox(
                    &mut kanban.auto_run_backlog_on_idle,
                    egui::RichText::new(format!("{} Tự chạy khi rảnh", Lucide::CLOCK))
                        .color(theme.text_silver)
                        .size(10.5),
                )
                .on_hover_text("Tự động chạy 1 task Backlog (có bật Tự chạy) khi IDE rảnh > 30s");

                // Right-aligned Primary Action Button
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!("{} Task mới", Lucide::PLUS))
                                    .color(theme.text_on_accent())
                                    .strong()
                                    .size(11.0),
                            )
                            .fill(theme.accent_pop)
                            .rounding(Rounding::same(6.0)),
                        )
                        .on_hover_text("Tạo task mới (⌘N)")
                        .clicked()
                    {
                        composer.open_create_modal_prefilled(
                            String::new(),
                            String::new(),
                            ElegansAgent::SeniorDev,
                            current_branch,
                        );
                    }
                });
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn render_four_column_board(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    kanban: &AgentKanbanManager,
    composer: &mut KanbanComposerState,
    custom_agents: &[DiscoveredAgent],
    current_branch: &str,
    board_w: f32,
    board_h: f32,
) -> Option<KanbanAction> {
    let mut action: Option<KanbanAction> = None;
    let gap = 8.0;
    let col_width = ((board_w - gap * 3.0 - 4.0) / 4.0).max(220.0);
    let col_inner_w = (col_width - 18.0).max(195.0);
    let col_h = (board_h - 8.0).max(220.0);
    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let pointer_released = ui.input(|i| i.pointer.any_released());

    ScrollArea::horizontal()
        .id_salt("kanban_4col_hscroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;

                for col_type in KanbanColumn::all() {
                    let header_col = column_accent(col_type, theme);
                    let col_tasks: Vec<&AgentTaskCard> = kanban
                        .tasks
                        .iter()
                        .filter(|t| t.column == col_type && matches_filters(t, composer))
                        .collect();
                    let total_in_col = col_tasks.len();

                    // Allocate a strict vertical column UI so children NEVER inherit horizontal layout
                    let col_alloc = ui.allocate_ui_with_layout(
                        vec2(col_width, col_h),
                        Layout::top_down(Align::Min),
                        |ui| {
                            let is_drag_hover = composer.dragging_task_id.is_some()
                                && pointer_pos
                                    .map(|p| ui.max_rect().contains(p))
                                    .unwrap_or(false);

                            let col_frame = Frame::none()
                                .fill(theme.surface_1)
                                .rounding(Rounding::same(10.0))
                                .stroke(Stroke::new(
                                    if is_drag_hover { 1.8 } else { 1.0 },
                                    if is_drag_hover {
                                        theme.accent_pop
                                    } else {
                                        theme.border
                                    },
                                ))
                                .inner_margin(Margin::same(9.0));

                            col_frame.show(ui, |ui| {
                                ui.set_width(col_inner_w);
                                ui.set_min_height((col_h - 20.0).max(200.0));

                                ui.vertical(|ui| {
                                    ui.set_width(col_inner_w);

                                    // Column Header
                                    ui.horizontal(|ui| {
                                        let (dot_r, _) =
                                            ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                                        ui.painter()
                                            .circle_filled(dot_r.center(), 3.5, header_col);

                                        ui.label(
                                            egui::RichText::new(col_type.title())
                                                .color(theme.text_paper)
                                                .strong()
                                                .size(11.2),
                                        );
                                        Frame::none()
                                            .fill(theme.surface_2)
                                            .rounding(Rounding::same(4.0))
                                            .inner_margin(Margin::symmetric(6.0, 1.5))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{}",
                                                        total_in_col
                                                    ))
                                                    .color(theme.text_silver)
                                                    .strong()
                                                    .size(10.0),
                                                );
                                            });

                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new(Lucide::PLUS)
                                                                .color(theme.text_silver)
                                                                .size(11.5),
                                                        )
                                                        .fill(Color32::TRANSPARENT),
                                                    )
                                                    .on_hover_text("Thêm nhanh task vào cột này")
                                                    .clicked()
                                                {
                                                    if composer.quick_add_col == Some(col_type) {
                                                        composer.quick_add_col = None;
                                                    } else {
                                                        composer.quick_add_col = Some(col_type);
                                                        composer.quick_add_text.clear();
                                                    }
                                                }
                                                if col_type == KanbanColumn::Done
                                                    && total_in_col > 5
                                                {
                                                    let lbl = if composer.done_show_all {
                                                        "Thu gọn"
                                                    } else {
                                                        "Tất cả"
                                                    };
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                egui::RichText::new(lbl)
                                                                    .color(theme.text_silver)
                                                                    .size(10.0),
                                                            )
                                                            .fill(Color32::TRANSPARENT),
                                                        )
                                                        .clicked()
                                                    {
                                                        composer.done_show_all =
                                                            !composer.done_show_all;
                                                    }
                                                }
                                            },
                                        );
                                    });

                                    ui.add_space(5.0);
                                    ui.separator();
                                    ui.add_space(6.0);

                                    // Inline Quick Add box
                                    if composer.quick_add_col == Some(col_type) {
                                        if let Some(qa_act) = render_inline_quick_add(
                                            ui,
                                            theme,
                                            col_type,
                                            composer,
                                            current_branch,
                                            col_inner_w,
                                        ) {
                                            action = Some(qa_act);
                                        }
                                        ui.add_space(6.0);
                                    }

                                    // Card List inside Vertical ScrollArea
                                    let visible_limit = if col_type == KanbanColumn::Done
                                        && !composer.done_show_all
                                    {
                                        5
                                    } else {
                                        usize::MAX
                                    };

                                    ScrollArea::vertical()
                                        .id_salt(format!("kanban_col_v4_{:?}", col_type))
                                        .auto_shrink([false, false])
                                        .max_height((col_h - 68.0).max(160.0))
                                        .show(ui, |ui| {
                                            ui.set_width(col_inner_w);
                                            if col_tasks.is_empty() {
                                                Frame::none()
                                                    .fill(Color32::TRANSPARENT)
                                                    .rounding(Rounding::same(6.0))
                                                    .stroke(Stroke::new(1.0, theme.border))
                                                    .inner_margin(Margin::symmetric(8.0, 18.0))
                                                    .show(ui, |ui| {
                                                        ui.set_width(col_inner_w - 16.0);
                                                        ui.vertical_centered(|ui| {
                                                            ui.label(
                                                                egui::RichText::new(
                                                                    "Chưa có thẻ — kéo thả hoặc bấm +",
                                                                )
                                                                .color(theme.text_silver)
                                                                .size(10.5),
                                                            );
                                                        });
                                                    });
                                            } else {
                                                for task in col_tasks.iter().take(visible_limit) {
                                                    if let Some(act) = render_task_card(
                                                        ui,
                                                        theme,
                                                        elapsed_secs,
                                                        task,
                                                        composer,
                                                        custom_agents,
                                                        col_inner_w,
                                                    ) {
                                                        action = Some(act);
                                                    }
                                                    ui.add_space(7.0);
                                                }
                                                if col_type == KanbanColumn::Done
                                                    && !composer.done_show_all
                                                    && total_in_col > 5
                                                {
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                egui::RichText::new(format!(
                                                                    "Xem thêm {} task đã xong",
                                                                    total_in_col - 5
                                                                ))
                                                                .color(theme.text_silver)
                                                                .size(10.5),
                                                            )
                                                            .fill(theme.surface_2)
                                                            .rounding(Rounding::same(5.0)),
                                                        )
                                                        .clicked()
                                                    {
                                                        composer.done_show_all = true;
                                                    }
                                                }
                                            }
                                        });
                                });
                            });
                        },
                    );

                    let col_rect = col_alloc.response.rect;
                    composer.last_column_rects.push(col_rect);

                    // Handle dropping into this column (if not already handled by dropping onto a specific card)
                    if pointer_released && action.is_none() {
                        if let (Some(pos), Some(dragged_id)) =
                            (pointer_pos, composer.dragging_task_id.clone())
                        {
                            if col_rect.contains(pos) {
                                action = Some(KanbanAction::MoveToColumn {
                                    task_id: dragged_id,
                                    target_col: col_type,
                                    insert_before_id: None,
                                });
                                composer.dragging_task_id = None;
                            }
                        }
                    }
                }
            });
        });

    action
}

#[allow(clippy::too_many_arguments)]
fn render_compact_board(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    kanban: &AgentKanbanManager,
    composer: &mut KanbanComposerState,
    custom_agents: &[DiscoveredAgent],
    current_branch: &str,
) -> Option<KanbanAction> {
    let mut action: Option<KanbanAction> = None;
    let avail_w = (ui.available_width() - 20.0).max(220.0);

    ScrollArea::vertical()
        .id_salt("kanban_compact_vscroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for col_type in KanbanColumn::all() {
                let header_col = column_accent(col_type, theme);
                let col_tasks: Vec<&AgentTaskCard> = kanban
                    .tasks
                    .iter()
                    .filter(|t| t.column == col_type && matches_filters(t, composer))
                    .collect();
                let is_collapsed = composer
                    .compact_collapsed
                    .get(&col_type)
                    .copied()
                    .unwrap_or(col_type == KanbanColumn::Done);

                Frame::none()
                    .fill(theme.surface_1)
                    .rounding(Rounding::same(8.0))
                    .stroke(Stroke::new(1.0, theme.border))
                    .inner_margin(Margin::same(9.0))
                    .show(ui, |ui| {
                        ui.set_width(avail_w);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let chev = if is_collapsed {
                                    Lucide::CHEVRON_RIGHT
                                } else {
                                    Lucide::CHEVRON_DOWN
                                };
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(format!(
                                                "{} {} ({})",
                                                chev,
                                                col_type.title(),
                                                col_tasks.len()
                                            ))
                                            .color(header_col)
                                            .strong()
                                            .size(11.2),
                                        )
                                        .fill(Color32::TRANSPARENT),
                                    )
                                    .clicked()
                                {
                                    composer.compact_collapsed.insert(col_type, !is_collapsed);
                                }

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(Lucide::PLUS)
                                                    .color(theme.text_silver)
                                                    .size(11.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        composer.compact_collapsed.insert(col_type, false);
                                        composer.quick_add_col = Some(col_type);
                                        composer.quick_add_text.clear();
                                    }
                                });
                            });

                            if !is_collapsed {
                                ui.add_space(6.0);
                                if composer.quick_add_col == Some(col_type) {
                                    if let Some(qa) = render_inline_quick_add(
                                        ui,
                                        theme,
                                        col_type,
                                        composer,
                                        current_branch,
                                        avail_w - 8.0,
                                    ) {
                                        action = Some(qa);
                                    }
                                    ui.add_space(6.0);
                                }
                                for task in col_tasks {
                                    if let Some(act) = render_task_card(
                                        ui,
                                        theme,
                                        elapsed_secs,
                                        task,
                                        composer,
                                        custom_agents,
                                        avail_w - 8.0,
                                    ) {
                                        action = Some(act);
                                    }
                                    ui.add_space(6.0);
                                }
                            }
                        });
                    });
                ui.add_space(7.0);
            }
        });

    action
}

fn render_inline_quick_add(
    ui: &mut Ui,
    theme: &CanhTheme,
    col_type: KanbanColumn,
    composer: &mut KanbanComposerState,
    current_branch: &str,
    width: f32,
) -> Option<KanbanAction> {
    let mut act = None;
    Frame::none()
        .fill(theme.surface_2)
        .rounding(Rounding::same(6.0))
        .stroke(Stroke::new(1.0, theme.accent_pop))
        .inner_margin(Margin::same(7.0))
        .show(ui, |ui| {
            ui.set_width((width - 14.0).max(160.0));
            ui.vertical(|ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut composer.quick_add_text)
                        .hint_text("Tên task... (Enter để tạo)")
                        .desired_width(ui.available_width())
                        .text_color(theme.text_paper),
                );
                resp.request_focus();
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if (enter
                        || ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Thêm")
                                        .color(theme.text_on_accent())
                                        .strong()
                                        .size(10.2),
                                )
                                .fill(theme.accent_pop)
                                .rounding(Rounding::same(4.0)),
                            )
                            .clicked())
                        && !composer.quick_add_text.trim().is_empty()
                    {
                        act = Some(KanbanAction::CreateTask {
                            title: composer.quick_add_text.trim().to_string(),
                            description: String::new(),
                            priority: "P1".to_string(),
                            agent: ElegansAgent::SeniorDev,
                            custom_agent: None,
                            attached_skills: Vec::new(),
                            branch: current_branch.to_string(),
                            column: col_type,
                            auto_schedule_idle: true,
                            run_now: false,
                        });
                        composer.quick_add_text.clear();
                        composer.quick_add_col = None;
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Hủy")
                                    .color(theme.text_silver)
                                    .size(10.2),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .clicked()
                    {
                        composer.quick_add_col = None;
                    }
                });
            });
        });
    act
}

fn render_task_card(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    task: &AgentTaskCard,
    composer: &mut KanbanComposerState,
    custom_agents: &[DiscoveredAgent],
    card_outer_width: f32,
) -> Option<KanbanAction> {
    let mut card_action: Option<KanbanAction> = None;
    let is_selected = composer.selected_task_id.as_deref() == Some(task.id.as_str());
    let card_inner_w = (card_outer_width - 20.0).max(175.0);

    let border_col = if task.run_failed {
        theme.danger
    } else if task.is_running_bg {
        let pulse = ((elapsed_secs * 3.0).sin() * 0.5) + 0.5;
        Color32::from_rgba_unmultiplied(
            theme.accent_pop.r(),
            theme.accent_pop.g(),
            theme.accent_pop.b(),
            (150.0 + pulse * 105.0) as u8,
        )
    } else if is_selected {
        theme.accent_pop
    } else {
        theme.border_strong
    };

    let bg_fill = if is_selected {
        theme.surface_3
    } else {
        theme.surface_2
    };

    let card_frame = Frame::none()
        .fill(bg_fill)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(
            if task.is_running_bg || task.run_failed || is_selected {
                1.5
            } else {
                1.0
            },
            border_col,
        ))
        .inner_margin(Margin::same(10.0));

    let inner_resp = card_frame.show(ui, |ui| {
        ui.set_width(card_inner_w);
        ui.vertical(|ui| {
            ui.set_width(card_inner_w);

            // Row 1: Drag Handle + Priority chip + Task ID + Status badge + Avatar
            ui.horizontal(|ui| {
                let (grip_rect, grip_resp) =
                    ui.allocate_exact_size(vec2(14.0, 18.0), Sense::drag());
                ui.painter().text(
                    grip_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    Lucide::GRIP_VERTICAL,
                    egui::FontId::proportional(11.0),
                    theme.text_silver,
                );
                if grip_resp.drag_started() || grip_resp.dragged() {
                    composer.dragging_task_id = Some(task.id.clone());
                }
                grip_resp.on_hover_text("Kéo để chuyển cột hoặc sắp xếp");

                // Priority pill
                let pcol = priority_color(&task.priority, theme);
                Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(pcol.r(), pcol.g(), pcol.b(), 36))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(5.0, 1.5))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&task.priority)
                                .color(pcol)
                                .strong()
                                .size(9.8),
                        );
                    });

                ui.label(
                    egui::RichText::new(&task.id)
                        .color(theme.text_silver)
                        .monospace()
                        .strong()
                        .size(10.0),
                );

                if task.run_failed {
                    Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(
                            theme.danger.r(),
                            theme.danger.g(),
                            theme.danger.b(),
                            38,
                        ))
                        .rounding(Rounding::same(3.0))
                        .inner_margin(Margin::symmetric(4.0, 1.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("Lỗi")
                                    .color(theme.danger)
                                    .strong()
                                    .size(9.5),
                            );
                        });
                } else if task.is_running_bg {
                    let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot.center(), 3.2, theme.accent_pop);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let av_state = if task.is_running_bg {
                        AgentAvatarState::Thinking
                    } else {
                        AgentAvatarState::Idle
                    };
                    AvatarManager::render_animated_avatar(
                        ui,
                        task.assigned_agent,
                        av_state,
                        20.0,
                        elapsed_secs,
                        theme.accent_pop,
                    );
                });
            });

            ui.add_space(6.0);

            // Row 2: Full-width Clickable Title (wraps normally across full card_inner_w!)
            let title_resp = ui.add_sized(
                vec2(card_inner_w, 0.0),
                egui::Label::new(
                    egui::RichText::new(&task.title)
                        .color(theme.text_paper)
                        .strong()
                        .size(12.0),
                )
                .wrap()
                .sense(Sense::click()),
            );
            if title_resp.clicked() {
                composer.open_drawer_for(task);
            }

            // Optional short description snippet
            if !task.description.trim().is_empty() {
                ui.add_space(3.0);
                let short_desc: String = task.description.chars().take(90).collect();
                let desc_txt = if task.description.chars().count() > 90 {
                    format!("{}…", short_desc)
                } else {
                    short_desc
                };
                let desc_resp = ui.add_sized(
                    vec2(card_inner_w, 0.0),
                    egui::Label::new(
                        egui::RichText::new(desc_txt)
                            .color(theme.text_silver)
                            .size(10.5),
                    )
                    .wrap()
                    .sense(Sense::click()),
                );
                if desc_resp.clicked() {
                    composer.open_drawer_for(task);
                }
            }

            // Row 3: Subtle metadata (Agent · Branch · Elapsed · Skills)
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} {}", task.assigned_agent.icon(), task.display_agent_name()))
                        .color(theme.text_silver)
                        .size(10.0),
                );
                ui.label(egui::RichText::new("·").color(theme.border_strong).size(10.0));
                ui.label(
                    egui::RichText::new(format!(
                        "{} {}",
                        Lucide::GIT_BRANCH,
                        task.worktree_branch
                    ))
                    .color(theme.text_silver)
                    .size(10.0),
                );
                if task.time_spent_secs > 0 {
                    ui.label(egui::RichText::new("·").color(theme.border_strong).size(10.0));
                    ui.label(
                        egui::RichText::new(format!(
                            "{} {}",
                            Lucide::CLOCK,
                            format_duration(task.time_spent_secs)
                        ))
                        .color(theme.text_silver)
                        .size(10.0),
                    );
                }
                if !task.attached_skills.is_empty() {
                    Frame::none()
                        .fill(theme.surface_3)
                        .rounding(Rounding::same(3.0))
                        .inner_margin(Margin::symmetric(4.0, 0.5))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("${}", task.attached_skills.len()))
                                    .color(theme.accent_pop)
                                    .size(9.5),
                            );
                        });
                }
            });

            // Running / Failed / Live Output snippet
            if task.is_running_bg || task.run_failed {
                ui.add_space(6.0);
                if task.is_running_bg {
                    let bar = egui::ProgressBar::new(task.progress_pct)
                        .desired_height(4.0)
                        .fill(theme.accent_pop);
                    ui.add(bar);
                    ui.add_space(4.0);
                }
                Frame::none()
                    .fill(theme.code_bg)
                    .rounding(Rounding::same(5.0))
                    .inner_margin(Margin::symmetric(7.0, 5.0))
                    .show(ui, |ui| {
                        ui.set_width((card_inner_w - 14.0).max(140.0));
                        let snippet: String = task.live_output.chars().take(110).collect();
                        ui.label(
                            egui::RichText::new(snippet)
                                .color(if task.run_failed {
                                    theme.danger
                                } else {
                                    theme.accent_pop
                                })
                                .monospace()
                                .size(9.8),
                        );
                    });
            }

            // Inline Delete Confirmation Bar
            if composer.confirm_delete_card_id.as_deref() == Some(task.id.as_str()) {
                ui.add_space(6.0);
                Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(
                        theme.danger.r(),
                        theme.danger.g(),
                        theme.danger.b(),
                        28,
                    ))
                    .rounding(Rounding::same(5.0))
                    .inner_margin(Margin::symmetric(6.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Xóa thẻ này?")
                                    .color(theme.danger)
                                    .strong()
                                    .size(10.2),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("Xóa")
                                                .color(Color32::WHITE)
                                                .strong()
                                                .size(10.0),
                                        )
                                        .fill(theme.danger)
                                        .rounding(Rounding::same(4.0)),
                                    )
                                    .clicked()
                                {
                                    composer.confirm_delete_card_id = None;
                                    card_action = Some(KanbanAction::DeleteTask(task.id.clone()));
                                }
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("Hủy")
                                                .color(theme.text_silver)
                                                .size(10.0),
                                        )
                                        .fill(Color32::TRANSPARENT),
                                    )
                                    .clicked()
                                {
                                    composer.confirm_delete_card_id = None;
                                }
                            });
                        });
                    });
            }

            ui.add_space(7.0);

            // Row 4: Clean Primary Action + Overflow Menu `...`
            ui.horizontal(|ui| {
                if task.is_running_bg {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!("{} Dừng", Lucide::STOP))
                                    .color(Color32::WHITE)
                                    .strong()
                                    .size(10.0),
                            )
                            .fill(theme.danger)
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        card_action = Some(KanbanAction::StopBackground(task.id.clone()));
                    }
                } else if task.run_failed {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!("{} Chạy lại", Lucide::ROTATE_CCW))
                                    .color(theme.text_on_accent())
                                    .strong()
                                    .size(10.0),
                            )
                            .fill(theme.accent_pop)
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        card_action = Some(KanbanAction::RunInBackground(task.id.clone()));
                    }
                } else if task.column == KanbanColumn::Verifying {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!("{} Duyệt", Lucide::CHECK))
                                    .color(Color32::WHITE)
                                    .strong()
                                    .size(10.0),
                            )
                            .fill(theme.success)
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        card_action = Some(KanbanAction::ApproveTask(task.id.clone()));
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Trả lại")
                                    .color(theme.text_paper)
                                    .size(10.0),
                            )
                            .fill(theme.surface_3)
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        card_action = Some(KanbanAction::RejectTask {
                            task_id: task.id.clone(),
                            note: String::new(),
                        });
                    }
                } else if task.column != KanbanColumn::Done {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!("{} Chạy", Lucide::PLAY))
                                    .color(theme.text_on_accent())
                                    .strong()
                                    .size(10.0),
                            )
                            .fill(theme.accent_pop)
                            .rounding(Rounding::same(5.0)),
                        )
                        .on_hover_text("Giao Agent chạy nền task này qua AGY")
                        .clicked()
                    {
                        card_action = Some(KanbanAction::RunInBackground(task.id.clone()));
                    }
                } else {
                    ui.label(
                        egui::RichText::new(format!("{} Hoàn tất", Lucide::CIRCLE_CHECK))
                            .color(theme.success)
                            .strong()
                            .size(10.0),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Overflow `...` Menu
                    ui.menu_button(
                        egui::RichText::new(Lucide::ELLIPSIS)
                            .color(theme.text_silver)
                            .size(12.0),
                        |ui| {
                            if ui
                                .button(format!("{} Mở chi tiết & Sửa", Lucide::PENCIL))
                                .clicked()
                            {
                                composer.open_drawer_for(task);
                                ui.close_menu();
                            }
                            ui.menu_button(format!("{} Đổi Agent", Lucide::BOT), |ui| {
                                for &ag in ALL_AGENTS {
                                    if ui.button(format!("{} {}", ag.icon(), ag.name())).clicked() {
                                        card_action = Some(KanbanAction::ReassignAgent {
                                            task_id: task.id.clone(),
                                            next_agent: ag,
                                            custom_agent: None,
                                        });
                                        ui.close_menu();
                                    }
                                }
                                if !custom_agents.is_empty() {
                                    ui.separator();
                                    for cag in custom_agents {
                                        if ui.button(format!("@{}", cag.name)).clicked() {
                                            card_action = Some(KanbanAction::ReassignAgent {
                                                task_id: task.id.clone(),
                                                next_agent: ElegansAgent::SeniorDev,
                                                custom_agent: Some(cag.name.clone()),
                                            });
                                            ui.close_menu();
                                        }
                                    }
                                }
                            });
                            ui.menu_button(format!("{} Chuyển cột", Lucide::ARROW_RIGHT), |ui| {
                                for col in KanbanColumn::all() {
                                    if col != task.column && ui.button(col.title()).clicked() {
                                        card_action = Some(KanbanAction::MoveToColumn {
                                            task_id: task.id.clone(),
                                            target_col: col,
                                            insert_before_id: None,
                                        });
                                        ui.close_menu();
                                    }
                                }
                            });
                            if ui
                                .button(format!("{} Nhân bản", Lucide::COPY_PLUS))
                                .clicked()
                            {
                                card_action = Some(KanbanAction::DuplicateTask(task.id.clone()));
                                ui.close_menu();
                            }
                            if !task.full_logs.is_empty()
                                && ui
                                    .button(format!("{} Copy logs", Lucide::COPY))
                                    .clicked()
                            {
                                ui.ctx().copy_text(task.full_logs.join("\n"));
                                ui.close_menu();
                            }
                            ui.separator();
                            if ui
                                .button(
                                    egui::RichText::new(format!("{} Xóa task", Lucide::TRASH))
                                        .color(theme.danger),
                                )
                                .clicked()
                            {
                                composer.confirm_delete_card_id = Some(task.id.clone());
                                ui.close_menu();
                            }
                        },
                    );

                    // Quote to Chat icon button
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(Lucide::MESSAGE_SQUARE)
                                    .color(theme.text_silver)
                                    .size(11.0),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text("Trích dẫn task này sang Chat")
                        .clicked()
                    {
                        card_action = Some(KanbanAction::QuoteTaskToChat(QuotedContext {
                            source_msg_id: task.id.clone(),
                            author_label: format!("Kanban {}", task.display_agent_name()),
                            snippet: format!("{}\nKết quả: {}", task.title, task.live_output),
                            is_quote_block: true,
                        }));
                    }

                    // Detail drawer quick open icon
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(Lucide::PENCIL)
                                    .color(theme.text_silver)
                                    .size(11.0),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text("Mở khung Sửa & Xem Log")
                        .clicked()
                    {
                        composer.open_drawer_for(task);
                    }
                });
            });
        });
    });

    let card_rect = inner_resp.response.rect;
    composer
        .last_card_rects
        .push((task.id.clone(), card_rect));

    // Handle drop onto this card (reorder before this card)
    if let Some(ref dragged_id) = composer.dragging_task_id.clone() {
        if dragged_id != &task.id {
            let pointer_pos = ui.input(|i| i.pointer.interact_pos());
            if let Some(pos) = pointer_pos {
                if card_rect.contains(pos) {
                    ui.painter().line_segment(
                        [card_rect.left_top(), card_rect.right_top()],
                        Stroke::new(2.5, theme.accent_pop),
                    );
                    if ui.input(|i| i.pointer.any_released()) {
                        card_action = Some(KanbanAction::MoveToColumn {
                            task_id: dragged_id.clone(),
                            target_col: task.column,
                            insert_before_id: Some(task.id.clone()),
                        });
                        composer.dragging_task_id = None;
                    }
                }
            }
        }
    }

    card_action
}

#[allow(clippy::too_many_arguments)]
fn render_task_drawer(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    kanban: &AgentKanbanManager,
    composer: &mut KanbanComposerState,
    skills: &[DiscoveredSkill],
    custom_agents: &[DiscoveredAgent],
    branches: &[String],
) -> Option<KanbanAction> {
    let sel_id = composer.selected_task_id.clone()?;
    let live_card = kanban.tasks.iter().find(|t| t.id == sel_id)?;
    let is_running = live_card.is_running_bg;
    let run_failed = live_card.run_failed;
    let time_spent = live_card.time_spent_secs;
    let logs = live_card.full_logs.clone();
    let live_output = live_card.live_output.clone();

    let mut action: Option<KanbanAction> = None;
    let mut close_drawer = false;

    Frame::none()
        .fill(theme.surface_1)
        .rounding(Rounding::same(10.0))
        .stroke(Stroke::new(1.0, theme.border_strong))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let Some(ref mut draft) = composer.drawer_draft else {
                return;
            };

            // Drawer Header
            ui.horizontal(|ui| {
                AvatarManager::render_animated_avatar(
                    ui,
                    draft.agent,
                    if is_running {
                        AgentAvatarState::Thinking
                    } else {
                        AgentAvatarState::Idle
                    },
                    24.0,
                    elapsed_secs,
                    theme.accent_pop,
                );
                ui.label(
                    egui::RichText::new(&draft.task_id)
                        .color(theme.accent_pop)
                        .monospace()
                        .strong()
                        .size(12.0),
                );
                if draft.dirty {
                    ui.label(
                        egui::RichText::new("● chưa lưu")
                            .color(theme.warning)
                            .size(10.0),
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(Lucide::X)
                                    .color(theme.text_silver)
                                    .size(13.0),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text("Đóng (Esc)")
                        .clicked()
                    {
                        close_drawer = true;
                    }
                    if draft.dirty
                        && ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} Lưu", Lucide::SAVE))
                                        .color(theme.text_on_accent())
                                        .strong()
                                        .size(10.5),
                                )
                                .fill(theme.accent_pop)
                                .rounding(Rounding::same(5.0)),
                            )
                            .clicked()
                    {
                        draft.dirty = false;
                        action = Some(KanbanAction::UpdateTask {
                            task_id: draft.task_id.clone(),
                            title: draft.title.clone(),
                            description: draft.description.clone(),
                            priority: draft.priority.clone(),
                            agent: draft.agent,
                            custom_agent: draft.custom_agent.clone(),
                            attached_skills: draft.attached_skills.clone(),
                            branch: draft.branch.clone(),
                            column: draft.column,
                            auto_schedule_idle: draft.auto_schedule_idle,
                        });
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ScrollArea::vertical()
                .id_salt("kanban_drawer_body_scroll")
                .auto_shrink([false, false])
                .max_height((ui.available_height() - 46.0).max(150.0))
                .show(ui, |ui| {
                    // Editable Title
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut draft.title)
                                .desired_width(ui.available_width())
                                .font(egui::FontId::proportional(13.0))
                                .text_color(theme.text_paper),
                        )
                        .changed()
                    {
                        draft.dirty = true;
                    }

                    ui.add_space(8.0);

                    // Metadata Grid (Status, Priority, Agent, Branch, Auto-run)
                    egui::Grid::new("kanban_drawer_meta_grid")
                        .num_columns(2)
                        .spacing(vec2(10.0, 6.0))
                        .show(ui, |ui| {
                            // Status
                            ui.label(
                                egui::RichText::new("Trạng thái")
                                    .color(theme.text_silver)
                                    .size(10.8),
                            );
                            ui.menu_button(
                                format!("{} {}", draft.column.title(), Lucide::CHEVRON_DOWN),
                                |ui| {
                                    for col in KanbanColumn::all() {
                                        if ui
                                            .selectable_value(&mut draft.column, col, col.title())
                                            .changed()
                                        {
                                            draft.dirty = true;
                                            ui.close_menu();
                                        }
                                    }
                                },
                            );
                            ui.end_row();

                            // Priority
                            ui.label(
                                egui::RichText::new("Ưu tiên")
                                    .color(theme.text_silver)
                                    .size(10.8),
                            );
                            ui.horizontal(|ui| {
                                for p in ["P0", "P1", "P2"] {
                                    let is_p = draft.priority == p;
                                    let pcol = priority_color(p, theme);
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(p)
                                                    .color(if is_p {
                                                        Color32::WHITE
                                                    } else {
                                                        pcol
                                                    })
                                                    .strong()
                                                    .size(10.2),
                                            )
                                            .fill(if is_p { pcol } else { theme.surface_2 })
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .clicked()
                                    {
                                        draft.priority = p.to_string();
                                        draft.dirty = true;
                                    }
                                }
                            });
                            ui.end_row();

                            // Agent (Core + Custom)
                            ui.label(
                                egui::RichText::new("Agent")
                                    .color(theme.text_silver)
                                    .size(10.8),
                            );
                            let cur_ag = draft
                                .custom_agent
                                .as_deref()
                                .map(|c| format!("@{}", c))
                                .unwrap_or_else(|| draft.agent.name().to_string());
                            ui.menu_button(
                                format!("{} {}", cur_ag, Lucide::CHEVRON_DOWN),
                                |ui| {
                                    for &ag in ALL_AGENTS {
                                        let sel =
                                            draft.custom_agent.is_none() && draft.agent == ag;
                                        if ui
                                            .selectable_label(
                                                sel,
                                                format!(
                                                    "{} {} — {}",
                                                    ag.icon(),
                                                    ag.name(),
                                                    ag.title()
                                                ),
                                            )
                                            .clicked()
                                        {
                                            draft.agent = ag;
                                            draft.custom_agent = None;
                                            draft.dirty = true;
                                            ui.close_menu();
                                        }
                                    }
                                    if !custom_agents.is_empty() {
                                        ui.separator();
                                        for cag in custom_agents {
                                            let sel = draft.custom_agent.as_deref()
                                                == Some(cag.name.as_str());
                                            if ui
                                                .selectable_label(
                                                    sel,
                                                    format!("@{} — {}", cag.name, cag.role),
                                                )
                                                .clicked()
                                            {
                                                draft.custom_agent = Some(cag.name.clone());
                                                draft.dirty = true;
                                                ui.close_menu();
                                            }
                                        }
                                    }
                                },
                            );
                            ui.end_row();

                            // Branch
                            ui.label(
                                egui::RichText::new("Branch")
                                    .color(theme.text_silver)
                                    .size(10.8),
                            );
                            ui.menu_button(
                                format!(
                                    "{} {} {}",
                                    Lucide::GIT_BRANCH,
                                    draft.branch,
                                    Lucide::CHEVRON_DOWN
                                ),
                                |ui| {
                                    for b in branches {
                                        if ui
                                            .selectable_value(
                                                &mut draft.branch,
                                                b.clone(),
                                                b.as_str(),
                                            )
                                            .changed()
                                        {
                                            draft.dirty = true;
                                            ui.close_menu();
                                        }
                                    }
                                },
                            );
                            ui.end_row();

                            // Auto-run idle
                            ui.label(
                                egui::RichText::new("Tự động")
                                    .color(theme.text_silver)
                                    .size(10.8),
                            );
                            if ui
                                .checkbox(&mut draft.auto_schedule_idle, "Chạy khi máy nhàn rỗi")
                                .changed()
                            {
                                draft.dirty = true;
                            }
                            ui.end_row();
                        });

                    ui.add_space(8.0);

                    // Attached Skills Section
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("Skills:")
                                .color(theme.text_silver)
                                .size(10.8),
                        );
                        let mut remove_idx = None;
                        for (idx, sk_name) in draft.attached_skills.iter().enumerate() {
                            Frame::none()
                                .fill(theme.surface_2)
                                .rounding(Rounding::same(4.0))
                                .stroke(Stroke::new(1.0, theme.border_strong))
                                .inner_margin(Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("${}", sk_name))
                                                .color(theme.accent_pop)
                                                .size(10.2),
                                        );
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(Lucide::X)
                                                        .color(theme.text_silver)
                                                        .size(9.0),
                                                )
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .clicked()
                                        {
                                            remove_idx = Some(idx);
                                        }
                                    });
                                });
                        }
                        if let Some(i) = remove_idx {
                            draft.attached_skills.remove(i);
                            draft.dirty = true;
                        }

                        ui.menu_button(
                            egui::RichText::new(format!("{} Gắn Skill", Lucide::PLUS))
                                .color(theme.text_silver)
                                .size(10.2),
                            |ui| {
                                let enabled_skills: Vec<&DiscoveredSkill> =
                                    skills.iter().filter(|s| s.enabled).collect();
                                if enabled_skills.is_empty() {
                                    ui.label("Chưa có skill nào");
                                }
                                for sk in enabled_skills {
                                    if !draft.attached_skills.contains(&sk.name)
                                        && ui.button(format!("${}", sk.name)).clicked()
                                    {
                                        draft.attached_skills.push(sk.name.clone());
                                        draft.dirty = true;
                                        ui.close_menu();
                                    }
                                }
                            },
                        );
                    });

                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Mô tả / Prompt cho Agent")
                            .color(theme.text_silver)
                            .strong()
                            .size(10.8),
                    );
                    ui.add_space(3.0);
                    if ui
                        .add(
                            egui::TextEdit::multiline(&mut draft.description)
                                .desired_rows(4)
                                .desired_width(ui.available_width())
                                .hint_text("Nhập yêu cầu chi tiết, tiêu chí nghiệm thu...")
                                .text_color(theme.text_paper),
                        )
                        .changed()
                    {
                        draft.dirty = true;
                    }

                    // Verifying Approve / Reject Section
                    if draft.column == KanbanColumn::Verifying {
                        ui.add_space(8.0);
                        Frame::none()
                            .fill(theme.surface_2)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.info))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("Nghiệm thu kết quả")
                                        .color(theme.info)
                                        .strong()
                                        .size(10.8),
                                );
                                ui.add_space(4.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut draft.reject_note)
                                        .desired_width(ui.available_width())
                                        .hint_text("Ghi chú nếu cần trả lại Backlog..."),
                                );
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Duyệt → Done",
                                                    Lucide::CHECK
                                                ))
                                                .color(Color32::WHITE)
                                                .strong()
                                                .size(10.5),
                                            )
                                            .fill(theme.success)
                                            .rounding(Rounding::same(5.0)),
                                        )
                                        .clicked()
                                    {
                                        draft.dirty = false;
                                        action =
                                            Some(KanbanAction::ApproveTask(draft.task_id.clone()));
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Trả lại",
                                                    Lucide::ROTATE_CCW
                                                ))
                                                .color(theme.text_paper)
                                                .size(10.5),
                                            )
                                            .fill(theme.surface_3)
                                            .rounding(Rounding::same(5.0)),
                                        )
                                        .clicked()
                                    {
                                        draft.dirty = false;
                                        action = Some(KanbanAction::RejectTask {
                                            task_id: draft.task_id.clone(),
                                            note: draft.reject_note.clone(),
                                        });
                                    }
                                });
                            });
                    }

                    // Full Background Execution Logs
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Execution Logs ({} dòng · {})",
                                Lucide::TERMINAL,
                                logs.len(),
                                format_duration(time_spent)
                            ))
                            .color(if run_failed {
                                theme.danger
                            } else {
                                theme.text_silver
                            })
                            .strong()
                            .size(10.8),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if !logs.is_empty()
                                && ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(format!("{} Copy", Lucide::COPY))
                                                .color(theme.text_silver)
                                                .size(10.0),
                                        )
                                        .fill(theme.surface_2)
                                        .rounding(Rounding::same(4.0)),
                                    )
                                    .clicked()
                            {
                                ui.ctx().copy_text(logs.join("\n"));
                            }
                            ui.checkbox(&mut draft.auto_scroll_logs, "Cuộn cuối");
                        });
                    });
                    ui.add_space(4.0);
                    Frame::none()
                        .fill(theme.code_bg)
                        .rounding(Rounding::same(6.0))
                        .stroke(Stroke::new(1.0, theme.border))
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            ScrollArea::vertical()
                                .id_salt("drawer_full_logs_scroll")
                                .max_height(180.0)
                                .stick_to_bottom(draft.auto_scroll_logs)
                                .show(ui, |ui| {
                                    if logs.is_empty() {
                                        ui.label(
                                            egui::RichText::new(&live_output)
                                                .color(theme.text_silver)
                                                .monospace()
                                                .size(10.2),
                                        );
                                    } else {
                                        for line in &logs {
                                            let lcol = if line.contains("[stderr]")
                                                || line.contains("Lỗi:")
                                            {
                                                theme.danger
                                            } else {
                                                theme.text_paper
                                            };
                                            ui.label(
                                                egui::RichText::new(line)
                                                    .color(lcol)
                                                    .monospace()
                                                    .size(10.2),
                                            );
                                        }
                                    }
                                });
                        });
                });

            // Drawer Footer Action Row
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if is_running {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!("{} Dừng", Lucide::STOP))
                                    .color(Color32::WHITE)
                                    .strong()
                                    .size(10.5),
                            )
                            .fill(theme.danger)
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        action = Some(KanbanAction::StopBackground(draft.task_id.clone()));
                    }
                } else if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!("{} Chạy nền", Lucide::PLAY))
                                .color(theme.text_on_accent())
                                .strong()
                                .size(10.5),
                        )
                        .fill(theme.accent_pop)
                        .rounding(Rounding::same(5.0)),
                    )
                    .clicked()
                {
                    action = Some(KanbanAction::RunInBackground(draft.task_id.clone()));
                }

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!("{} Chat", Lucide::MESSAGE_SQUARE))
                                .color(theme.text_paper)
                                .size(10.5),
                        )
                        .fill(theme.surface_2)
                        .rounding(Rounding::same(5.0)),
                    )
                    .clicked()
                {
                    action = Some(KanbanAction::QuoteTaskToChat(QuotedContext {
                        source_msg_id: draft.task_id.clone(),
                        author_label: format!("Kanban {}", draft.task_id),
                        snippet: format!("{}\n{}", draft.title, live_output),
                        is_quote_block: true,
                    }));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if draft.confirm_delete {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Xác nhận xóa")
                                        .color(Color32::WHITE)
                                        .strong()
                                        .size(10.2),
                                )
                                .fill(theme.danger)
                                .rounding(Rounding::same(4.0)),
                            )
                            .clicked()
                        {
                            action = Some(KanbanAction::DeleteTask(draft.task_id.clone()));
                            close_drawer = true;
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Hủy")
                                        .color(theme.text_silver)
                                        .size(10.2),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            draft.confirm_delete = false;
                        }
                    } else if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(Lucide::TRASH)
                                    .color(theme.danger)
                                    .size(12.0),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text("Xóa task này")
                        .clicked()
                    {
                        draft.confirm_delete = true;
                    }
                });
            });
        });

    if close_drawer {
        composer.selected_task_id = None;
        composer.drawer_draft = None;
    }

    action
}

#[allow(clippy::too_many_arguments)]
fn render_create_task_modal(
    ctx: &egui::Context,
    theme: &CanhTheme,
    composer: &mut KanbanComposerState,
    skills: &[DiscoveredSkill],
    custom_agents: &[DiscoveredAgent],
    branches: &[String],
    current_branch: &str,
) -> Option<KanbanAction> {
    let mut action: Option<KanbanAction> = None;
    let mut open = composer.show_create_modal;

    if composer.selected_branch.is_empty() {
        composer.selected_branch = current_branch.to_string();
    }

    egui::Window::new(format!("{} Tạo Task mới", Lucide::KANBAN))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(510.0)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .frame(
            Frame::none()
                .fill(theme.panel_bg)
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(1.0, theme.border_strong))
                .inner_margin(Margin::same(14.0)),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Tiêu đề task")
                    .color(theme.text_silver)
                    .size(10.8),
            );
            ui.add_space(3.0);
            ui.add(
                egui::TextEdit::singleline(&mut composer.new_title)
                    .desired_width(ui.available_width())
                    .hint_text("VD: Tối ưu truy vấn API giỏ hàng..."),
            );

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Mô tả / Prompt chi tiết cho Agent")
                    .color(theme.text_silver)
                    .size(10.8),
            );
            ui.add_space(3.0);
            ui.add(
                egui::TextEdit::multiline(&mut composer.new_desc)
                    .desired_rows(4)
                    .desired_width(ui.available_width())
                    .hint_text("Phạm vi file, tiêu chí hoàn thành, lệnh test cần chạy..."),
            );

            ui.add_space(8.0);
            egui::Grid::new("create_task_modal_grid")
                .num_columns(2)
                .spacing(vec2(12.0, 8.0))
                .show(ui, |ui| {
                    // Priority
                    ui.label(
                        egui::RichText::new("Ưu tiên")
                            .color(theme.text_silver)
                            .size(10.8),
                    );
                    ui.horizontal(|ui| {
                        for p in ["P0", "P1", "P2"] {
                            let is_p = composer.selected_priority == p;
                            let pcol = priority_color(p, theme);
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(p)
                                            .color(if is_p { Color32::WHITE } else { pcol })
                                            .strong()
                                            .size(10.5),
                                    )
                                    .fill(if is_p { pcol } else { theme.surface_2 })
                                    .rounding(Rounding::same(4.0)),
                                )
                                .clicked()
                            {
                                composer.selected_priority = p.to_string();
                            }
                        }
                    });
                    ui.end_row();

                    // Agent
                    ui.label(
                        egui::RichText::new("Agent phụ trách")
                            .color(theme.text_silver)
                            .size(10.8),
                    );
                    let cur_ag = composer
                        .selected_custom_agent
                        .as_deref()
                        .map(|c| format!("@{}", c))
                        .unwrap_or_else(|| {
                            format!(
                                "{} {}",
                                composer.selected_agent.icon(),
                                composer.selected_agent.name()
                            )
                        });
                    ui.menu_button(
                        format!("{} {}", cur_ag, Lucide::CHEVRON_DOWN),
                        |ui| {
                            for &ag in ALL_AGENTS {
                                let sel = composer.selected_custom_agent.is_none()
                                    && composer.selected_agent == ag;
                                if ui
                                    .selectable_label(
                                        sel,
                                        format!("{} {} — {}", ag.icon(), ag.name(), ag.title()),
                                    )
                                    .clicked()
                                {
                                    composer.selected_agent = ag;
                                    composer.selected_custom_agent = None;
                                    ui.close_menu();
                                }
                            }
                            if !custom_agents.is_empty() {
                                ui.separator();
                                for cag in custom_agents {
                                    let sel = composer.selected_custom_agent.as_deref()
                                        == Some(cag.name.as_str());
                                    if ui
                                        .selectable_label(
                                            sel,
                                            format!("@{} — {}", cag.name, cag.role),
                                        )
                                        .clicked()
                                    {
                                        composer.selected_custom_agent = Some(cag.name.clone());
                                        ui.close_menu();
                                    }
                                }
                            }
                        },
                    );
                    ui.end_row();

                    // Branch
                    ui.label(
                        egui::RichText::new("Branch")
                            .color(theme.text_silver)
                            .size(10.8),
                    );
                    ui.menu_button(
                        format!(
                            "{} {} {}",
                            Lucide::GIT_BRANCH,
                            composer.selected_branch,
                            Lucide::CHEVRON_DOWN
                        ),
                        |ui| {
                            for b in branches {
                                if ui
                                    .selectable_value(
                                        &mut composer.selected_branch,
                                        b.clone(),
                                        b.as_str(),
                                    )
                                    .changed()
                                {
                                    ui.close_menu();
                                }
                            }
                        },
                    );
                    ui.end_row();
                });

            // Attached Skills picker
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Skills đính kèm:")
                        .color(theme.text_silver)
                        .size(10.8),
                );
                let mut rem = None;
                for (i, s) in composer.selected_skills.iter().enumerate() {
                    Frame::none()
                        .fill(theme.surface_2)
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("${}", s))
                                        .color(theme.accent_pop)
                                        .size(10.2),
                                );
                                if ui.small_button(Lucide::X).clicked() {
                                    rem = Some(i);
                                }
                            });
                        });
                }
                if let Some(i) = rem {
                    composer.selected_skills.remove(i);
                }
                ui.menu_button(
                    egui::RichText::new(format!("{} Chọn skill", Lucide::PLUS))
                        .color(theme.text_silver)
                        .size(10.2),
                    |ui| {
                        for sk in skills.iter().filter(|s| s.enabled) {
                            if !composer.selected_skills.contains(&sk.name)
                                && ui.button(format!("${}", sk.name)).clicked()
                            {
                                composer.selected_skills.push(sk.name.clone());
                                ui.close_menu();
                            }
                        }
                    },
                );
            });

            ui.add_space(6.0);
            ui.checkbox(
                &mut composer.auto_schedule_idle,
                "Cho phép tự động chạy khi IDE nhàn rỗi",
            );

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            let can_submit = !composer.new_title.trim().is_empty();
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        can_submit,
                        egui::Button::new(
                            egui::RichText::new(format!("{} Lưu vào Backlog", Lucide::PLUS))
                                .color(theme.text_paper)
                                .size(11.0),
                        )
                        .fill(theme.surface_2)
                        .stroke(Stroke::new(1.0, theme.border_strong))
                        .rounding(Rounding::same(5.0)),
                    )
                    .clicked()
                {
                    action = Some(KanbanAction::CreateTask {
                        title: composer.new_title.clone(),
                        description: composer.new_desc.clone(),
                        priority: composer.selected_priority.clone(),
                        agent: composer.selected_agent,
                        custom_agent: composer.selected_custom_agent.clone(),
                        attached_skills: composer.selected_skills.clone(),
                        branch: composer.selected_branch.clone(),
                        column: KanbanColumn::Backlog,
                        auto_schedule_idle: composer.auto_schedule_idle,
                        run_now: false,
                    });
                    composer.new_title.clear();
                    composer.new_desc.clear();
                    composer.selected_skills.clear();
                    composer.show_create_modal = false;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(
                            can_submit,
                            egui::Button::new(
                                egui::RichText::new(format!("{} Giao & chạy ngay", Lucide::PLAY))
                                    .color(theme.text_on_accent())
                                    .strong()
                                    .size(11.0),
                            )
                            .fill(theme.accent_pop)
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                    {
                        action = Some(KanbanAction::CreateTask {
                            title: composer.new_title.clone(),
                            description: composer.new_desc.clone(),
                            priority: composer.selected_priority.clone(),
                            agent: composer.selected_agent,
                            custom_agent: composer.selected_custom_agent.clone(),
                            attached_skills: composer.selected_skills.clone(),
                            branch: composer.selected_branch.clone(),
                            column: KanbanColumn::InProgress,
                            auto_schedule_idle: composer.auto_schedule_idle,
                            run_now: true,
                        });
                        composer.new_title.clear();
                        composer.new_desc.clear();
                        composer.selected_skills.clear();
                        composer.show_create_modal = false;
                    }
                });
            });
        });

    if !open {
        composer.show_create_modal = false;
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kanban_board_layout_renders_full_width_cards_in_dark_and_light_themes() {
        for theme in [CanhTheme::warm_dark(), CanhTheme::warm_light()] {
            let ctx = egui::Context::default();
            let mut kanban = AgentKanbanManager::default_workspace_board();
            let mut composer = KanbanComposerState::default();
            let branches = vec!["main".to_string(), "feat/ui".to_string()];

            let mut raw_input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    vec2(1320.0, 800.0),
                )),
                ..Default::default()
            };

            // Run 2 frames so egui resolves ScrollArea and Frame layout rects
            for _ in 0..2 {
                let _ = ctx.run(raw_input.clone(), |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let _ = render_agent_kanban_view(
                            ui,
                            &theme,
                            1.0,
                            &mut kanban,
                            &mut composer,
                            &[],
                            &[],
                            &branches,
                            "main",
                        );
                    });
                });
                raw_input.events.clear();
            }

            assert_eq!(
                composer.last_column_rects.len(),
                4,
                "All 4 Kanban columns must be rendered"
            );
            for i in 0..3 {
                assert!(
                    composer.last_column_rects[i].left()
                        < composer.last_column_rects[i + 1].left(),
                    "Columns must be arranged horizontally left-to-right"
                );
            }

            assert!(
                composer.last_card_rects.len() >= 4,
                "All default task cards must be rendered"
            );
            for (id, rect) in &composer.last_card_rects {
                assert!(
                    rect.width() >= 190.0,
                    "Card {} width ({}) is too narrow (would cause vertical text wrapping!)",
                    id,
                    rect.width()
                );
                assert!(
                    rect.height() < 240.0,
                    "Card {} height ({}) is abnormally tall (indicates vertical character wrapping!)",
                    id,
                    rect.height()
                );
            }
        }
    }
}
