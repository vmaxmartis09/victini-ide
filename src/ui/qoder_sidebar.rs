use crate::ai::{ChatThread, ElegansAgent, ThreadAgentMode, ALL_AGENTS};
use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use crate::ui::header::{icon_btn, project_badge_code};
use crate::ui::layout_state::CenterViewMode;
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};

pub enum SidebarAction {
    ToggleMiniMode,
    ToggleLightDark,
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

    // MINI RAIL MODE (100% Icon-First)
    if is_mini {
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::PANEL_LEFT,
                theme.text_paper,
                theme.chip_bg,
                "Mở rộng Sidebar (⌘B)",
            )
            .clicked()
            {
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(8.0);
            if icon_btn(
                ui,
                Lucide::PLUS,
                theme.on_accent,
                theme.accent_pop,
                "Phiên Chat mới",
            )
            .clicked()
            {
                action = Some(SidebarAction::NewChatThread);
            }
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::BOT,
                theme.text_paper,
                Color32::TRANSPARENT,
                "Chat 1-1 với Agent riêng",
            )
            .clicked()
            {
                *show_agents_tab = true;
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::CODE,
                theme.text_silver,
                Color32::TRANSPARENT,
                "Git Diff & Live Code Editor",
            )
            .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(
                    CenterViewMode::GitDiffModal,
                ));
            }
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::KANBAN,
                theme.text_silver,
                Color32::TRANSPARENT,
                "Swarm Kanban Board",
            )
            .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::KanbanBoard));
            }
            ui.add_space(6.0);
            let theme_ic = if theme.is_dark {
                Lucide::SUN
            } else {
                Lucide::MOON
            };
            if icon_btn(
                ui,
                theme_ic,
                theme.accent_pop,
                Color32::TRANSPARENT,
                "Đổi Light / Dark Mode (⌘L)",
            )
            .clicked()
            {
                action = Some(SidebarAction::ToggleLightDark);
            }
        });
        return action;
    }

    // FULL SIDEBAR (Adapts dynamically to Dark OLED & Paper Light Mode)
    ui.vertical(|ui| {
        // 1. Brand Row with Icon Controls
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_stroke(
                rect,
                Rounding::same(3.5),
                Stroke::new(1.3, theme.text_silver),
            );
            ui.painter()
                .circle_filled(rect.center(), 2.8, theme.accent_pop);

            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("T3 Code")
                    .color(theme.text_paper)
                    .strong()
                    .size(13.0),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_btn(
                    ui,
                    Lucide::CHEVRON_LEFT,
                    theme.text_silver,
                    Color32::TRANSPARENT,
                    "Thu gọn Sidebar (⌘B)",
                )
                .clicked()
                {
                    action = Some(SidebarAction::ToggleMiniMode);
                }
                let theme_ic = if theme.is_dark {
                    Lucide::SUN
                } else {
                    Lucide::MOON
                };
                if icon_btn(
                    ui,
                    theme_ic,
                    theme.accent_pop,
                    Color32::TRANSPARENT,
                    "Đổi Light / Dark Mode (⌘L)",
                )
                .clicked()
                {
                    action = Some(SidebarAction::ToggleLightDark);
                }
            });
        });

        ui.add_space(8.0);

        // 2. Search Box + Icon Action Buttons ([+] [Folder])
        ui.horizontal(|ui| {
            let search_w = (ui.available_width() - 62.0).max(55.0);
            Frame::none()
                .fill(theme.input_bg)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::symmetric(6.0, 3.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::SEARCH)
                                .color(theme.text_silver)
                                .size(11.0),
                        );
                        ui.add_sized(
                            vec2((search_w - 18.0).max(36.0), 18.0),
                            egui::TextEdit::singleline(search_query)
                                .hint_text("Search...")
                                .frame(false)
                                .text_color(theme.text_paper),
                        );
                    });
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_btn(
                    ui,
                    Lucide::PLUS,
                    theme.on_accent,
                    theme.accent_pop,
                    "Tạo phiên Chat mới",
                )
                .clicked()
                {
                    *show_agents_tab = false;
                    action = Some(SidebarAction::NewChatThread);
                }
                if icon_btn(
                    ui,
                    Lucide::FOLDER,
                    theme.text_silver,
                    theme.chip_bg,
                    "Quản lý Projects & Workspaces",
                )
                .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::ProjectsModal,
                    ));
                }
            });
        });

        ui.add_space(6.0);

        // 3. Icon-First Mode Switcher Tabs: [MessageSquare N] | [Bot 1-1 M]
        ui.horizontal(|ui| {
            let half_w = (ui.available_width() - 6.0) * 0.5;
            let threads_btn = egui::Button::new(
                egui::RichText::new(format!(
                    "{} {}",
                    Lucide::MESSAGE_SQUARE,
                    threads.len()
                ))
                .color(if !*show_agents_tab {
                    theme.on_accent
                } else {
                    theme.text_silver
                })
                .strong()
                .size(11.0),
            )
            .fill(if !*show_agents_tab {
                theme.accent_pop
            } else {
                theme.chip_bg
            })
            .rounding(Rounding::same(5.0));

            if ui
                .add_sized(vec2(half_w, 24.0), threads_btn)
                .on_hover_text("Danh sách các phiên Chat (Threads)")
                .clicked()
            {
                *show_agents_tab = false;
            }

            let agents_btn = egui::Button::new(
                egui::RichText::new(format!(
                    "{} 1-1 ({})",
                    Lucide::BOT,
                    ALL_AGENTS.len() + agents.len()
                ))
                .color(if *show_agents_tab {
                    theme.on_accent
                } else {
                    theme.text_silver
                })
                .strong()
                .size(11.0),
            )
            .fill(if *show_agents_tab {
                theme.accent_pop
            } else {
                theme.chip_bg
            })
            .rounding(Rounding::same(5.0));

            if ui
                .add_sized(vec2(half_w, 24.0), agents_btn)
                .on_hover_text("Chat 1-1 với từng Agent riêng & Quản lý Skills")
                .clicked()
            {
                *show_agents_tab = true;
            }
        });

        ui.add_space(8.0);

        // 4. Main Scrollable List
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
                            theme.panel_bg
                        } else {
                            Color32::TRANSPARENT
                        };
                        let border_col = if is_active {
                            theme.border_strong
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
                                    // Line 1: Dynamic Project Badge [LA] landorus ... Time & Delete Icon
                                    ui.horizontal(|ui| {
                                        Frame::none()
                                            .fill(theme.tag_red_bg)
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
                                                                egui::RichText::new(Lucide::TRASH)
                                                                    .color(theme.text_silver)
                                                                    .size(10.0),
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
                                                        theme.success
                                                    }
                                                    ThreadAgentMode::DedicatedCustom { .. } => {
                                                        theme.info
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
                    // TAB B: DEDICATED 1-ON-1 AGENT LAUNCHER & SKILLS (Icon-First)
                    // 0. Swarm Auto Router card
                    Frame::none()
                        .fill(theme.panel_bg)
                        .rounding(Rounding::same(6.0))
                        .stroke(Stroke::new(1.0, theme.border))
                        .inner_margin(Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(Lucide::SPARKLES)
                                        .color(theme.accent_pop)
                                        .size(13.0),
                                );
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Swarm Auto-Router")
                                            .color(theme.text_paper)
                                            .strong()
                                            .size(11.2),
                                    );
                                    ui.label(
                                        egui::RichText::new("Tự động chọn Agent")
                                            .color(theme.text_silver)
                                            .size(9.8),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if icon_btn(
                                        ui,
                                        Lucide::MESSAGE_SQUARE,
                                        theme.on_accent,
                                        theme.accent_pop,
                                        "Chuyển sang chế độ Swarm Auto-Router",
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
                        let card_resp = Frame::none()
                            .fill(theme.panel_bg)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::symmetric(8.0, 6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(ag.icon())
                                            .color(theme.accent_pop)
                                            .size(13.0),
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
                                                .size(9.5),
                                        );
                                    });
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if icon_btn(
                                            ui,
                                            Lucide::MESSAGE_SQUARE,
                                            theme.text_silver,
                                            theme.chip_bg,
                                            &format!("Mở Chat 1-1 riêng với {}", ag.name()),
                                        )
                                        .clicked()
                                        {
                                            *show_agents_tab = false;
                                            action =
                                                Some(SidebarAction::OpenDedicatedCoreAgent(ag));
                                        }
                                    });
                                });
                            });
                        if card_resp.response.interact(egui::Sense::click()).clicked() {
                            *show_agents_tab = false;
                            action = Some(SidebarAction::OpenDedicatedCoreAgent(ag));
                        }
                        ui.add_space(3.0);
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("SUBAGENTS ({})", agents.len()))
                                .color(theme.info)
                                .strong()
                                .size(10.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::PLUS,
                                theme.accent_pop,
                                theme.chip_bg,
                                "Tạo Subagent mới (.claude/agents/*.md)",
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
                        let card_resp = Frame::none()
                            .fill(theme.panel_bg)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
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
                                        if icon_btn(
                                            ui,
                                            Lucide::MESSAGE_SQUARE,
                                            theme.text_silver,
                                            theme.chip_bg,
                                            &format!("Chat 1-1 với @{}", ag.name),
                                        )
                                        .clicked()
                                        {
                                            *show_agents_tab = false;
                                            action =
                                                Some(SidebarAction::OpenDedicatedCustomAgent(idx));
                                        }
                                        if icon_btn(
                                            ui,
                                            Lucide::EDIT,
                                            theme.text_silver,
                                            Color32::TRANSPARENT,
                                            "Sửa Subagent này",
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
                        if card_resp.response.interact(egui::Sense::click()).clicked() {
                            *show_agents_tab = false;
                            action = Some(SidebarAction::OpenDedicatedCustomAgent(idx));
                        }
                        ui.add_space(3.0);
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("SKILLS ({})", skills.len()))
                                .color(theme.text_silver)
                                .strong()
                                .size(10.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::CLOUD_DOWNLOAD,
                                theme.accent_pop,
                                theme.chip_bg,
                                "Import Skills từ GitHub",
                            )
                            .clicked()
                            {
                                action = Some(SidebarAction::OpenSkillStudioNewSkill);
                            }
                            if icon_btn(
                                ui,
                                Lucide::PLUS,
                                theme.text_paper,
                                theme.chip_bg,
                                "Tạo Skill mới (SKILL.md)",
                            )
                            .clicked()
                            {
                                action = Some(SidebarAction::OpenSkillStudioNewSkill);
                            }
                        });
                    });
                    ui.add_space(3.0);

                    if q.is_empty() {
                        // Clean, compact card summarizing all skills without spamming 35 rows
                        Frame::none()
                            .fill(theme.surface_1)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::symmetric(9.0, 7.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(Lucide::WAND)
                                            .color(theme.accent_pop)
                                            .size(13.0),
                                    );
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "Skill Studio ({} skills)",
                                                skills.len()
                                            ))
                                            .color(theme.text_paper)
                                            .strong()
                                            .size(10.8),
                                        );
                                        ui.label(
                                            egui::RichText::new("Xem, soạn thảo, test & nạp GitHub")
                                                .color(theme.text_silver)
                                                .size(9.2),
                                        );
                                    });
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if icon_btn(
                                            ui,
                                            Lucide::CHEVRON_RIGHT,
                                            theme.text_silver,
                                            Color32::TRANSPARENT,
                                            "Mở Skill Studio đầy đủ",
                                        )
                                        .clicked()
                                        {
                                            action = Some(SidebarAction::SwitchCenterView(
                                                CenterViewMode::AgentSkillStudio,
                                            ));
                                        }
                                    });
                                });
                            });
                    } else {
                        // When searching, show matching skills compactly (max 5)
                        let mut count = 0;
                        for (idx, sk) in skills.iter().enumerate() {
                            if !sk.name.to_lowercase().contains(&q)
                                && !sk.description.to_lowercase().contains(&q)
                            {
                                continue;
                            }
                            if count >= 5 {
                                ui.label(
                                    egui::RichText::new("... mở Skill Studio để xem thêm kết quả")
                                        .color(theme.text_silver)
                                        .size(9.5),
                                );
                                break;
                            }
                            count += 1;
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(format!(
                                                "{} ${}",
                                                Lucide::WAND,
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
                }
            });

        // 5. Bottom Accordion: Real Working / Settled Status & Icon-First Utility Bar
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(4.0);

            // Bottom utility icons: [Settings] [Studio] [GitHub Import] [Code Editor] [Kanban]
            ui.horizontal(|ui| {
                if icon_btn(
                    ui,
                    Lucide::SETTINGS,
                    if active_mode == CenterViewMode::SettingsModal {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    },
                    Color32::TRANSPARENT,
                    "Cài đặt, Giao diện Light/Dark & Memory Reaper",
                )
                .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::SettingsModal,
                    ));
                }
                if icon_btn(
                    ui,
                    Lucide::BOT,
                    if active_mode == CenterViewMode::AgentSkillStudio {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    },
                    Color32::TRANSPARENT,
                    "Agent & Skill Studio + GitHub Importer",
                )
                .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::AgentSkillStudio,
                    ));
                }
                if icon_btn(
                    ui,
                    Lucide::CODE,
                    if active_mode == CenterViewMode::GitDiffModal {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    },
                    Color32::TRANSPARENT,
                    "Git Diff & Live Code Editor",
                )
                .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::GitDiffModal,
                    ));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_btn(
                        ui,
                        Lucide::KANBAN,
                        if active_mode == CenterViewMode::KanbanBoard {
                            theme.accent_pop
                        } else {
                            theme.text_silver
                        },
                        Color32::TRANSPARENT,
                        "Swarm Kanban Board (⌘K)",
                    )
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
                    egui::RichText::new(format!(
                        "{} {}",
                        Lucide::MESSAGE_SQUARE,
                        threads.len()
                    ))
                    .color(theme.text_silver)
                    .size(10.5),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} {} {}",
                                    Lucide::GIT_COMMIT,
                                    file_changes.len(),
                                    Lucide::CHEVRON_RIGHT
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
                .fill(theme.panel_bg)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(if is_streaming {
                                    "Working"
                                } else {
                                    "Active"
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
                                        theme.success
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
