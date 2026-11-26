use crate::theme::Lucide;
use crate::ai::{
    AgentAvatarState, AgentKanbanManager, AgentTaskCard, AvatarManager, ElegansAgent, KanbanColumn,
     QuotedContext, ALL_AGENTS,
};
use crate::theme::CanhTheme;
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};

pub enum KanbanAction {
    CreateTask {
        title: String,
        description: String,
        priority: String,
        agent: ElegansAgent,
        run_now: bool,
    },
    ReassignAgent {
        task_id: String,
        next_agent: ElegansAgent,
    },
    MoveLeft(String),
    AdvanceTask(String),
    RunInBackground(String),
    QuoteTaskToChat(QuotedContext),
    DeleteTask(String),
}

pub struct KanbanComposerState {
    pub new_title: String,
    pub new_desc: String,
    pub selected_priority: String,
    pub selected_agent: ElegansAgent,
    pub expanded_task_id: Option<String>,
}

impl Default for KanbanComposerState {
    fn default() -> Self {
        Self {
            new_title: String::new(),
            new_desc: String::new(),
            selected_priority: "P1".to_string(),
            selected_agent: ElegansAgent::SeniorDev,
            expanded_task_id: None,
        }
    }
}

pub fn render_agent_kanban_view(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    kanban: &mut AgentKanbanManager,
    composer: &mut KanbanComposerState,
) -> Option<KanbanAction> {
    let mut action = None;

    ui.vertical(|ui| {
        // 1. Kanban Top Orchestrator Bar + Auto-Schedule Idle Toggle
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(format!("{} AI SWARM KANBAN & BACKGROUND ORCHESTRATOR", Lucide::KANBAN))
                    .color(theme.accent_pop)
                    .strong()
                    .size(13.0),
            );
            ui.checkbox(
                &mut kanban.auto_run_backlog_on_idle,
                egui::RichText::new("⏱ Auto-Run Backlog khi máy nhàn rỗi (>30s)")
                    .color(theme.text_silver)
                    .size(11.0),
            );
        });

        ui.add_space(6.0);

        // 2. Quick Task Creation & Agent Assignment Bar
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x14, 0x1C))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x20, 0x26, 0x36)))
            .inner_margin(Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("＋ Giao Task mới:")
                            .color(theme.text_paper)
                            .strong()
                            .size(11.0),
                    );
                    ui.add_sized(
                        vec2(180.0, 22.0),
                        egui::TextEdit::singleline(&mut composer.new_title)
                            .hint_text("Tên task (VD: Tối ưu API...)"),
                    );
                    ui.add_sized(
                        vec2(210.0, 22.0),
                        egui::TextEdit::singleline(&mut composer.new_desc)
                            .hint_text("Mô tả / Prompt chi tiết cho Agent..."),
                    );

                    // Priority selector pills (P0, P1, P2)
                    for prio in ["P0", "P1", "P2"] {
                        let is_p = composer.selected_priority == prio;
                        if ui
                            .selectable_label(
                                is_p,
                                egui::RichText::new(prio).strong().size(10.5),
                            )
                            .clicked()
                        {
                            composer.selected_priority = prio.to_string();
                        }
                    }

                    // Agent selector pills
                    for &ag in ALL_AGENTS {
                        let is_ag = composer.selected_agent == ag;
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} {}", ag.icon(), ag.name()))
                                .size(10.0)
                                .color(if is_ag { Color32::BLACK } else { theme.text_silver }),
                        )
                        .fill(if is_ag {
                            theme.accent_pop
                        } else {
                            Color32::from_rgb(0x18, 0x1C, 0x28)
                        })
                        .rounding(Rounding::same(4.0));

                        if ui.add(btn).clicked() {
                            composer.selected_agent = ag;
                        }
                    }

                    if ui
                        .button(
                            egui::RichText::new("＋ Backlog")
                                .color(theme.text_paper)
                                .size(10.5),
                        )
                        .clicked()
                        && !composer.new_title.trim().is_empty()
                    {
                        action = Some(KanbanAction::CreateTask {
                            title: composer.new_title.clone(),
                            description: composer.new_desc.clone(),
                            priority: composer.selected_priority.clone(),
                            agent: composer.selected_agent,
                            run_now: false,
                        });
                        composer.new_title.clear();
                        composer.new_desc.clear();
                    }

                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("▶ Giao & Chạy Ngầm Ngay")
                                    .color(Color32::BLACK)
                                    .strong()
                                    .size(10.5),
                            )
                            .fill(theme.accent_pop)
                            .rounding(Rounding::same(5.0)),
                        )
                        .clicked()
                        && !composer.new_title.trim().is_empty()
                    {
                        action = Some(KanbanAction::CreateTask {
                            title: composer.new_title.clone(),
                            description: composer.new_desc.clone(),
                            priority: composer.selected_priority.clone(),
                            agent: composer.selected_agent,
                            run_now: true,
                        });
                        composer.new_title.clear();
                        composer.new_desc.clear();
                    }
                });
            });

        ui.add_space(8.0);

        // 3. Responsive 4-Column Kanban Board (Horizontal Scroll + Min Column Width 235px so it never squishes!)
        let avail_w = ui.available_width();
        let col_width = ((avail_w - 36.0) / 4.0).max(235.0);
        let col_height = (ui.available_height() - 12.0).max(240.0);

        let columns = [
            (
                KanbanColumn::Backlog,
                Color32::from_rgb(0x8D, 0x94, 0x9E),
            ),
            (KanbanColumn::InProgress, theme.accent_pop),
            (
                KanbanColumn::Verifying,
                Color32::from_rgb(0x4E, 0xC9, 0xB0),
            ),
            (KanbanColumn::Done, Color32::from_rgb(0x98, 0xC3, 0x79)),
        ];

        ScrollArea::horizontal()
            .id_salt("kanban_responsive_horizontal_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (col_type, header_col) in columns {
                        let count = kanban
                            .tasks
                            .iter()
                            .filter(|t| t.column == col_type)
                            .count();

                        Frame::none()
                            .fill(Color32::from_rgb(0x0E, 0x10, 0x15))
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1B, 0x20, 0x2C)))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.set_width(col_width);
                                ui.set_min_height(col_height);

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(col_type.title())
                                            .color(header_col)
                                            .strong()
                                            .size(11.0),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{}", count))
                                                .color(theme.text_silver)
                                                .strong()
                                                .size(10.5),
                                        );
                                    });
                                });
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(6.0);

                                ScrollArea::vertical()
                                    .id_salt(format!("kanban_col_v2_{:?}", col_type))
                                    .max_height(col_height - 34.0)
                                    .show(ui, |ui| {
                                        for task in
                                            kanban.tasks.iter().filter(|t| t.column == col_type)
                                        {
                                            if let Some(act) = render_task_card(
                                                ui,
                                                theme,
                                                elapsed_secs,
                                                task,
                                                &mut composer.expanded_task_id,
                                            ) {
                                                action = Some(act);
                                            }
                                            ui.add_space(6.0);
                                        }
                                    });
                            });
                        ui.add_space(6.0);
                    }
                });
            });
    });

    action
}

fn render_task_card(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    task: &AgentTaskCard,
    expanded_task_id: &mut Option<String>,
) -> Option<KanbanAction> {
    let mut card_action = None;
    let border_col = if task.is_running_bg {
        theme.accent_pop
    } else {
        Color32::from_rgb(0x22, 0x28, 0x38)
    };

    Frame::none()
        .fill(Color32::from_rgb(0x14, 0x17, 0x20))
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(if task.is_running_bg { 1.5 } else { 1.0 }, border_col))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            // Top Row: Task ID + Priority Badge + Animated Avatar + Click-to-Reassign Agent
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&task.id)
                        .color(theme.text_silver)
                        .strong()
                        .size(10.0),
                );

                let prio_col = match task.priority.as_str() {
                    "P0" => Color32::from_rgb(0xE0, 0x6C, 0x75),
                    "P1" => theme.accent_pop,
                    _ => Color32::from_rgb(0x61, 0xAF, 0xEF),
                };
                ui.label(
                    egui::RichText::new(&task.priority)
                        .color(prio_col)
                        .strong()
                        .size(10.0),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let av_state = if task.is_running_bg {
                        AgentAvatarState::Thinking
                    } else if task.column == KanbanColumn::Done {
                        AgentAvatarState::Idle
                    } else {
                        AgentAvatarState::Idle
                    };

                    AvatarManager::render_animated_avatar(
                        ui,
                        task.assigned_agent,
                        av_state,
                        22.0,
                        elapsed_secs,
                        theme.accent_pop,
                    );

                    // Click agent pill to cycle/reassign agent directly on card!
                    if ui
                        .button(
                            egui::RichText::new(format!(
                                "{} {}",
                                task.assigned_agent.icon(),
                                task.assigned_agent.name()
                            ))
                            .color(theme.accent_pop)
                            .size(10.0)
                            .strong(),
                        )
                        .on_hover_text("Click để đổi Agent phụ trách Task này")
                        .clicked()
                    {
                        let idx = ALL_AGENTS
                            .iter()
                            .position(|&a| a == task.assigned_agent)
                            .unwrap_or(0);
                        let next_agent = ALL_AGENTS[(idx + 1) % ALL_AGENTS.len()];
                        card_action = Some(KanbanAction::ReassignAgent {
                            task_id: task.id.clone(),
                            next_agent,
                        });
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(&task.title)
                    .color(theme.text_paper)
                    .strong()
                    .size(11.8),
            );

            if !task.description.is_empty() {
                ui.label(
                    egui::RichText::new(&task.description)
                        .color(theme.text_silver)
                        .size(10.5),
                );
            }

            ui.add_space(5.0);
            // Animated Progress Bar
            let bar = egui::ProgressBar::new(task.progress_pct)
                .desired_height(5.0)
                .fill(if task.is_running_bg {
                    theme.accent_pop
                } else {
                    Color32::from_rgb(0x4E, 0xC9, 0xB0)
                });
            ui.add(bar);

            ui.add_space(5.0);
            // Live Agent Output Snippet
            Frame::none()
                .fill(Color32::from_rgb(0x0A, 0x0B, 0x0F))
                .rounding(Rounding::same(5.0))
                .inner_margin(Margin::symmetric(6.0, 4.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&task.live_output)
                            .color(if task.is_running_bg {
                                theme.accent_pop
                            } else {
                                theme.text_silver
                            })
                            .italics()
                            .size(10.2),
                    );
                });

            // Expandable Full Background Logs
            let is_expanded = expanded_task_id.as_deref() == Some(task.id.as_str());
            if is_expanded && !task.full_logs.is_empty() {
                ui.add_space(4.0);
                Frame::none()
                    .fill(Color32::from_rgb(0x07, 0x08, 0x0C))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::same(6.0))
                    .show(ui, |ui| {
                        for line in task.full_logs.iter().rev().take(6).rev() {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(theme.text_paper)
                                    .size(9.8),
                            );
                        }
                    });
            }

            ui.add_space(6.0);
            // Bottom Action Row: ←, →, ▶ Run BG, 💬 Quote to Chat, Logs, 🗑
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(format!("⏱ {}s", task.time_spent_secs))
                        .color(theme.text_silver)
                        .size(10.0),
                );

                if !task.full_logs.is_empty()
                    && ui
                        .small_button(if is_expanded { "▲ Logs" } else { "▼ Logs" })
                        .clicked()
                {
                    if is_expanded {
                        *expanded_task_id = None;
                    } else {
                        *expanded_task_id = Some(task.id.clone());
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .small_button(Lucide::TRASH)
                        .on_hover_text("Xóa task")
                        .clicked()
                    {
                        card_action = Some(KanbanAction::DeleteTask(task.id.clone()));
                    }

                    if ui
                        .small_button("→")
                        .on_hover_text("Chuyển sang cột tiếp theo")
                        .clicked()
                    {
                        card_action = Some(KanbanAction::AdvanceTask(task.id.clone()));
                    }

                    if ui
                        .small_button("←")
                        .on_hover_text("Lùi về cột trước")
                        .clicked()
                    {
                        card_action = Some(KanbanAction::MoveLeft(task.id.clone()));
                    }

                    if ui
                        .small_button(format!("{} Chat", Lucide::MESSAGE_SQUARE))
                        .on_hover_text("Trích dẫn Task & kết quả của Agent này sang khung Chat chính")
                        .clicked()
                    {
                        card_action = Some(KanbanAction::QuoteTaskToChat(QuotedContext {
                            source_msg_id: task.id.clone(),
                            author_label: format!("Kanban {}", task.assigned_agent.name()),
                            snippet: format!("{}\nKết quả: {}", task.title, task.live_output),
                            is_quote_block: true,
                        }));
                    }

                    if !task.is_running_bg
                        && ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("▶ Run BG")
                                        .color(Color32::BLACK)
                                        .strong()
                                        .size(9.8),
                                )
                                .fill(theme.accent_pop)
                                .rounding(Rounding::same(4.0)),
                            )
                            .on_hover_text("Giao Agent chạy ngầm task này bằng AGY CLI thật")
                            .clicked()
                    {
                        card_action = Some(KanbanAction::RunInBackground(task.id.clone()));
                    }
                });
            });
        });

    card_action
}
