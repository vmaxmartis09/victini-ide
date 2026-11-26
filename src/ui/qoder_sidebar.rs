use crate::ai::{ChatThread, ElegansAgent, ThreadAgentMode, ALL_AGENTS};
use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use crate::ui::header::project_badge_code;
use crate::ui::layout_state::CenterViewMode;
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};

pub enum SidebarAction {
    ToggleMiniMode,
    NewChatThread,
    SelectChatThread(usize),
    DeleteChatThread(usize),
    OpenDedicatedCoreAgent(ElegansAgent),
    OpenDedicatedCustomAgent(usize),
    SwitchToSwarmAuto,
    SwitchCenterView(CenterViewMode),
    OpenSkillStudioNewSkill,
    OpenSkillStudioNewAgent,
    InspectSkill(usize),
    InspectAgent(usize),
}

#[allow(clippy::too_many_arguments)]
pub fn render_qoder_sidebar(
    ui: &mut Ui,
    theme: &CanhTheme,
    is_mini: bool,
    active_mode: CenterViewMode,
    project_name: &str,
    branch_name: &str,
    is_streaming: bool,
    show_agents_tab: &mut bool,
    search_query: &mut String,
    file_changes: &[GitFileChange],
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
    threads: &[ChatThread],
    active_thread_idx: usize,
) -> Option<SidebarAction> {
    let mut action = None;
    let badge = project_badge_code(project_name);

    // MINI RAIL MODE
    if is_mini {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            if ui
                .button(
                    egui::RichText::new("T3")
                        .color(theme.text_paper)
                        .strong()
                        .size(13.0),
                )
                .on_hover_text("Mở rộng Sidebar (⌘B)")
                .clicked()
            {
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(8.0);
            if ui
                .button(
                    egui::RichText::new(Lucide::PLUS)
                        .color(theme.accent_pop)
                        .strong()
                        .size(14.0),
                )
                .on_hover_text("New Swarm Task")
                .clicked()
            {
                action = Some(SidebarAction::NewChatThread);
            }
            ui.add_space(6.0);
            if ui
                .button(
                    egui::RichText::new(Lucide::BOT)
                        .color(theme.text_paper)
                        .size(13.0),
                )
                .on_hover_text("Chat 1-1 với Agent riêng")
                .clicked()
            {
                *show_agents_tab = true;
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(6.0);
            if ui
                .button(
                    egui::RichText::new(Lucide::CODE)
                        .color(theme.text_silver)
                        .size(13.0),
                )
                .on_hover_text("Git Diff & Live Code Editor")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(
                    CenterViewMode::GitDiffModal,
                ));
            }
            ui.add_space(6.0);
            if ui
                .button(
                    egui::RichText::new(Lucide::KANBAN)
                        .color(theme.text_silver)
                        .size(13.0),
                )
                .on_hover_text("Swarm Kanban Board")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::KanbanBoard));
            }
        });
        return action;
    }

    // T3 CODE EXACT 1:1 SIDEBAR (#000000 Pure Black)
    ui.vertical(|ui| {
        // 1. T3 Code Brand Row
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_stroke(
                rect,
                Rounding::same(3.0),
                Stroke::new(1.2, Color32::from_rgb(0x80, 0x80, 0x80)),
            );
            ui.painter()
                .circle_filled(rect.center(), 2.5, theme.accent_pop);

            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("T3 Code")
                    .color(theme.text_paper)
                    .strong()
                    .size(13.0),
            );
            ui.label(
                egui::RichText::new("· Victini")
                    .color(theme.text_silver)
                    .size(11.0),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("«")
                                .color(theme.text_silver)
                                .size(12.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Thu gọn Sidebar (⌘B)")
                    .clicked()
                {
                    action = Some(SidebarAction::ToggleMiniMode);
                }
            });
        });

        ui.add_space(8.0);

        // 2. Search Box with Action Buttons
        ui.horizontal(|ui| {
            let search_w = (ui.available_width() - 52.0).max(60.0);
            Frame::none()
                .fill(Color32::from_rgb(0x0C, 0x0C, 0x0C))
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::symmetric(6.0, 3.0))
                .show(ui, |ui| {
                    ui.add_sized(
                        vec2(search_w, 20.0),
                        egui::TextEdit::singleline(search_query)
                            .hint_text("Search threads or agents...")
                            .frame(false)
                            .text_color(theme.text_paper),
                    );
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // New Task Button (Lucide::PLUS)
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::PLUS)
                                .color(theme.text_paper)
                                .strong()
                                .size(13.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Phiên Chat mới")
                    .clicked()
                {
                    *show_agents_tab = false;
                    action = Some(SidebarAction::NewChatThread);
                }
                // Projects / Workspaces (Lucide::FOLDER)
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::FOLDER)
                                .color(theme.text_silver)
                                .size(12.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Projects & Workspaces")
                    .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::ProjectsModal,
                    ));
                }
            });
        });

        ui.add_space(6.0);

        // 3. Mode Switcher Tabs: [Threads (N)] | [Agents 1-1 (6+M)]
        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 6.0) * 0.5;
            let threads_btn = egui::Button::new(
                egui::RichText::new(format!(
                    "{} Threads ({})",
                    Lucide::MESSAGE_SQUARE,
                    threads.len()
                ))
                .color(if !*show_agents_tab {
                    Color32::BLACK
                } else {
                    theme.text_silver
                })
                .strong()
                .size(10.5),
            )
            .fill(if !*show_agents_tab {
                theme.accent_pop
            } else {
                Color32::from_rgb(0x12, 0x12, 0x12)
            })
            .rounding(Rounding::same(5.0));

            if ui.add_sized(vec2(half_w, 23.0), threads_btn).clicked() {
                *show_agents_tab = false;
            }

            let agents_btn = egui::Button::new(
                egui::RichText::new(format!(
                    "{} Agents 1-1 ({})",
                    Lucide::BOT,
                    ALL_AGENTS.len() + agents.len()
                ))
                .color(if *show_agents_tab {
                    Color32::BLACK
                } else {
                    theme.text_silver
                })
                .strong()
                .size(10.5),
            )
            .fill(if *show_agents_tab {
                theme.accent_pop
            } else {
                Color32::from_rgb(0x12, 0x12, 0x12)
            })
            .rounding(Rounding::same(5.0));

            if ui.add_sized(vec2(half_w, 23.0), agents_btn).clicked() {
                *show_agents_tab = true;
            }
        });

        ui.add_space(8.0);

        // 4. Main Scrollable List (Either Threads OR Dedicated Agents 1-on-1)
        let q = search_query.trim().to_lowercase();
        ScrollArea::vertical()
            .id_salt("t3_sidebar_main_scroll")
            .auto_shrink([false, false])
            .max_height((ui.available_height() - 115.0).max(120.0))
            .show(ui, |ui| {
                if !*show_agents_tab {
                    // TAB A: THREAD LIST
                    let can_delete = threads.len() > 1;
                    for (idx, t) in threads.iter().enumerate() {
                        if !q.is_empty()
                            && !t.title.to_lowercase().contains(&q)
                            && !t.branch.to_lowercase().contains(&q)
                            && !t.agent_mode.short_tag().to_lowercase().contains(&q)
                        {
                            continue;
                        }

                        let is_active = idx == active_thread_idx;
                        let bg = if is_active {
                            Color32::from_rgb(0x14, 0x14, 0x14)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let border_col = if is_active {
                            Color32::from_rgb(0x24, 0x24, 0x24)
                        } else {
                            Color32::TRANSPARENT
                        };

                        let mut delete_clicked = false;
                        let resp = Frame::none()
                            .fill(bg)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, border_col))
                            .inner_margin(Margin::symmetric(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    // Line 1: Dynamic Project Badge [LA] landorus ... Time & Delete
                                    ui.horizontal(|ui| {
                                        Frame::none()
                                            .fill(Color32::from_rgb(0x28, 0x0E, 0x10))
                                            .rounding(Rounding::same(3.0))
                                            .inner_margin(Margin::symmetric(4.0, 1.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(&badge)
                                                        .color(theme.tag_red)
                                                        .strong()
                                                        .size(9.5),
                                                );
                                            });
                                        ui.label(
                                            egui::RichText::new(project_name)
                                                .color(theme.text_silver)
                                                .size(10.5),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if can_delete
                                                    && ui
                                                        .add(
                                                            egui::Button::new(
                                                                egui::RichText::new(Lucide::X)
                                                                    .color(theme.text_silver)
                                                                    .size(9.5),
                                                            )
                                                            .fill(Color32::TRANSPARENT),
                                                        )
                                                        .on_hover_text("Xóa phiên chat này")
                                                        .clicked()
                                                {
                                                    delete_clicked = true;
                                                }
                                                ui.label(
                                                    egui::RichText::new(&t.updated_at)
                                                        .color(theme.text_silver)
                                                        .size(10.0),
                                                );
                                            },
                                        );
                                    });

                                    ui.add_space(3.0);

                                    // Line 2: Thread Title
                                    ui.label(
                                        egui::RichText::new(&t.title)
                                            .color(theme.text_paper)
                                            .strong()
                                            .size(11.8),
                                    );

                                    ui.add_space(2.0);

                                    // Line 3: Branch + Dedicated Agent Tag + Status Dot
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} {}",
                                                Lucide::GIT_BRANCH,
                                                t.branch
                                            ))
                                            .color(theme.text_silver)
                                            .size(10.2),
                                        );

                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                let dot_col = match t.agent_mode {
                                                    ThreadAgentMode::SwarmAuto => theme.accent_pop,
                                                    ThreadAgentMode::DedicatedCore(_) => {
                                                        Color32::from_rgb(0x4E, 0xC9, 0xB0)
                                                    }
                                                    ThreadAgentMode::DedicatedCustom { .. } => {
                                                        Color32::from_rgb(0x51, 0xA2, 0xFF)
                                                    }
                                                };
                                                let (dot_rect, _) = ui.allocate_exact_size(
                                                    vec2(8.0, 8.0),
                                                    egui::Sense::hover(),
                                                );
                                                ui.painter().circle_filled(
                                                    dot_rect.center(),
                                                    2.5,
                                                    dot_col,
                                                );

                                                ui.label(
                                                    egui::RichText::new(t.agent_mode.short_tag())
                                                        .color(dot_col)
                                                        .size(9.8),
                                                );
                                            },
                                        );
                                    });
                                });
                            });

                        if delete_clicked {
                            action = Some(SidebarAction::DeleteChatThread(idx));
                        } else if resp.response.interact(egui::Sense::click()).clicked() {
                            action = Some(SidebarAction::SelectChatThread(idx));
                        }
                        ui.add_space(4.0);
                    }
                } else {
                    // TAB B: DEDICATED 1-ON-1 AGENT LAUNCHER & SKILLS
                    ui.label(
                        egui::RichText::new("CHỌN AGENT ĐỂ CHAT RIÊNG (1-ON-1)")
                            .color(theme.accent_pop)
                            .strong()
                            .size(10.0),
                    );
                    ui.add_space(4.0);

                    // 0. Swarm Auto Router card
                    Frame::none()
                        .fill(Color32::from_rgb(0x13, 0x13, 0x13))
                        .rounding(Rounding::same(6.0))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x22, 0x22)))
                        .inner_margin(Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(Lucide::SPARKLES)
                                        .color(theme.accent_pop)
                                        .size(12.0),
                                );
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Swarm Auto-Router")
                                            .color(theme.text_paper)
                                            .strong()
                                            .size(11.2),
                                    );
                                    ui.label(
                                        egui::RichText::new("Tự động chọn Agent theo câu hỏi")
                                            .color(theme.text_silver)
                                            .size(9.8),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Auto")
                                                    .color(Color32::BLACK)
                                                    .strong()
                                                    .size(9.5),
                                            )
                                            .fill(theme.accent_pop)
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .clicked()
                                    {
                                        *show_agents_tab = false;
                                        action = Some(SidebarAction::SwitchToSwarmAuto);
                                    }
                                });
                            });
                        });

                    ui.add_space(6.0);

                    // 1. 6 Core Elegans Agents
                    for &ag in ALL_AGENTS {
                        if !q.is_empty()
                            && !ag.name().to_lowercase().contains(&q)
                            && !ag.title().to_lowercase().contains(&q)
                        {
                            continue;
                        }
                        Frame::none()
                            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1D, 0x1D, 0x1D)))
                            .inner_margin(Margin::symmetric(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(ag.icon())
                                            .color(theme.accent_pop)
                                            .size(12.0),
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
                                                .size(9.8),
                                        );
                                    });
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("Chat 1-1")
                                                        .color(Color32::BLACK)
                                                        .strong()
                                                        .size(9.5),
                                                )
                                                .fill(Color32::from_rgb(0x4E, 0xC9, 0xB0))
                                                .rounding(Rounding::same(4.0)),
                                            )
                                            .on_hover_text(format!(
                                                "Mở phòng chat 1-1 riêng với {}",
                                                ag.name()
                                            ))
                                            .clicked()
                                        {
                                            *show_agents_tab = false;
                                            action =
                                                Some(SidebarAction::OpenDedicatedCoreAgent(ag));
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("CUSTOM SUBAGENTS ({})", agents.len()))
                                .color(Color32::from_rgb(0x51, 0xA2, 0xFF))
                                .strong()
                                .size(10.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("+ Tạo Agent")
                                            .color(theme.accent_pop)
                                            .size(9.5),
                                    )
                                    .fill(Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                action = Some(SidebarAction::OpenSkillStudioNewAgent);
                            }
                        });
                    });
                    ui.add_space(3.0);

                    for (idx, ag) in agents.iter().enumerate() {
                        if !q.is_empty()
                            && !ag.name.to_lowercase().contains(&q)
                            && !ag.role.to_lowercase().contains(&q)
                        {
                            continue;
                        }
                        Frame::none()
                            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1D, 0x1D, 0x1D)))
                            .inner_margin(Margin::symmetric(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("@{}", ag.name))
                                            .color(theme.text_paper)
                                            .strong()
                                            .size(10.8),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("Chat 1-1")
                                                        .color(Color32::BLACK)
                                                        .strong()
                                                        .size(9.5),
                                                )
                                                .fill(Color32::from_rgb(0x51, 0xA2, 0xFF))
                                                .rounding(Rounding::same(4.0)),
                                            )
                                            .clicked()
                                        {
                                            *show_agents_tab = false;
                                            action =
                                                Some(SidebarAction::OpenDedicatedCustomAgent(idx));
                                        }
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("Sửa")
                                                        .color(theme.text_silver)
                                                        .size(9.5),
                                                )
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .clicked()
                                        {
                                            action = Some(SidebarAction::InspectAgent(idx));
                                        }
                                    });
                                });
                                ui.label(
                                    egui::RichText::new(&ag.role)
                                        .color(theme.text_silver)
                                        .size(9.5),
                                );
                            });
                        ui.add_space(3.0);
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("SKILLS ({})", skills.len()))
                                .color(theme.text_silver)
                                .strong()
                                .size(10.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("+ Tạo Skill")
                                            .color(theme.accent_pop)
                                            .size(9.5),
                                    )
                                    .fill(Color32::TRANSPARENT),
                                )
                                .clicked()
                            {
                                action = Some(SidebarAction::OpenSkillStudioNewSkill);
                            }
                        });
                    });
                    for (idx, sk) in skills.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} ${}",
                                            Lucide::CODE,
                                            sk.name
                                        ))
                                        .color(theme.text_paper)
                                        .size(10.5),
                                    )
                                    .fill(Color32::TRANSPARENT),
                                )
                                .on_hover_text(&sk.description)
                                .clicked()
                            {
                                action = Some(SidebarAction::InspectSkill(idx));
                            }
                        });
                    }
                }
            });

        // 5. Bottom Accordion: Real Working / Settled Status & Utility Icons
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(4.0);

            // Bottom utility icons: [Settings] [Agent Studio] [Diff/Editor] [Kanban]
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::SETTINGS)
                                .color(if active_mode == CenterViewMode::SettingsModal {
                                    theme.accent_pop
                                } else {
                                    theme.text_silver
                                })
                                .size(13.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Settings & Memory Reaper")
                    .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::SettingsModal,
                    ));
                }
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::BOT)
                                .color(if active_mode == CenterViewMode::AgentSkillStudio {
                                    theme.accent_pop
                                } else {
                                    theme.text_silver
                                })
                                .size(13.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Agent & Skill Studio")
                    .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::AgentSkillStudio,
                    ));
                }
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::CODE)
                                .color(if active_mode == CenterViewMode::GitDiffModal {
                                    theme.accent_pop
                                } else {
                                    theme.text_silver
                                })
                                .size(13.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Git Diff & Live Code Editor")
                    .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::GitDiffModal,
                    ));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(Lucide::KANBAN)
                                    .color(if active_mode == CenterViewMode::KanbanBoard {
                                        theme.accent_pop
                                    } else {
                                        theme.text_silver
                                    })
                                    .size(13.0),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text("Swarm Kanban Board")
                        .clicked()
                    {
                        action = Some(SidebarAction::SwitchCenterView(
                            CenterViewMode::KanbanBoard,
                        ));
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Settled / Changes summary row
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Threads · {}", threads.len()))
                        .color(theme.text_silver)
                        .size(10.5),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} changes >",
                                    file_changes.len()
                                ))
                                .color(if file_changes.is_empty() {
                                    theme.text_silver
                                } else {
                                    theme.accent_pop
                                })
                                .size(10.5),
                            )
                            .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text("Xem & sửa các file thay đổi")
                        .clicked()
                    {
                        action = Some(SidebarAction::SwitchCenterView(
                            CenterViewMode::GitDiffModal,
                        ));
                    }
                });
            });

            ui.add_space(4.0);

            // Working Row (Real Active Session Capsule)
            let active_th = threads.get(active_thread_idx);
            let active_title = active_th
                .map(|t| t.title.as_str())
                .unwrap_or("Swarm Session");
            let active_tag = active_th
                .map(|t| t.agent_mode.short_tag())
                .unwrap_or_else(|| "Swarm Auto".to_string());

            Frame::none()
                .fill(Color32::from_rgb(0x11, 0x11, 0x11))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(if is_streaming {
                                    "Working · Streaming"
                                } else {
                                    "Active Session"
                                })
                                .color(if is_streaming {
                                    theme.accent_pop
                                } else {
                                    theme.text_paper
                                })
                                .strong()
                                .size(10.5),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(&active_tag)
                                        .color(theme.text_silver)
                                        .size(9.8),
                                );
                            });
                        });
                        ui.label(
                            egui::RichText::new(active_title)
                                .color(theme.text_paper)
                                .strong()
                                .size(11.2),
                        );
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} {}",
                                    Lucide::GIT_BRANCH,
                                    branch_name
                                ))
                                .color(theme.text_silver)
                                .size(10.2),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let (dot_rect, _) =
                                    ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                                ui.painter().circle_filled(
                                    dot_rect.center(),
                                    2.5,
                                    if is_streaming {
                                        theme.accent_pop
                                    } else {
                                        Color32::from_rgb(0x4E, 0xC9, 0xB0)
                                    },
                                );
                            });
                        });
                    });
                });
        });
    });

    action
}
