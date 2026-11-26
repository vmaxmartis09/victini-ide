use crate::ai::{ChatSessionManager, ElegansAgent, ALL_AGENTS};
use crate::config::{DiscoveredAgent, DiscoveredSkill, ProjectManager};
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use crate::ui::header::icon_btn;
use crate::ui::layout_state::CenterViewMode;
use egui::{pos2, vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};
use std::collections::HashMap;
use std::path::PathBuf;

pub enum SidebarAction {
    ToggleMiniMode,
    ToggleLightDark,
    NewChatThread,
    SelectChatThread(usize),
    DeleteChatThread(usize),
    ToggleProjectExpand(PathBuf),
    NewProjectChatThread(PathBuf),
    SelectProjectThread {
        project_path: PathBuf,
        thread_idx: usize,
    },
    DeleteProjectThread {
        project_path: PathBuf,
        thread_idx: usize,
    },
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
    project_mgr: &ProjectManager,
    project_chats: &HashMap<PathBuf, ChatSessionManager>,
    active_chat_mgr: &ChatSessionManager,
    branch_name: &str,
    is_streaming: bool,
    show_agents_tab: &mut bool,
    search_query: &mut String,
    file_changes: &[GitFileChange],
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
) -> Option<SidebarAction> {
    let mut action = None;

    // 1. MINI RAIL MODE (Compact 54px Icon Rail)
    if is_mini {
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::PANEL_LEFT,
                theme.text_paper,
                theme.surface_2,
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
                "Tạo phiên Chat mới",
            )
            .clicked()
            {
                action = Some(SidebarAction::NewChatThread);
            }
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::MESSAGE_SQUARE,
                if !*show_agents_tab && active_mode == CenterViewMode::ChatTimeline {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                Color32::TRANSPARENT,
                "Cây thư mục Project & Phiên Chat",
            )
            .clicked()
            {
                *show_agents_tab = false;
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::BOT,
                if *show_agents_tab {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                Color32::TRANSPARENT,
                "Danh sách Agent 1-1 & Skills",
            )
            .clicked()
            {
                *show_agents_tab = true;
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(6.0);
            if icon_btn(
                ui,
                Lucide::KANBAN,
                if active_mode == CenterViewMode::KanbanBoard {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                Color32::TRANSPARENT,
                "Bảng Swarm Kanban (⌘K)",
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
                theme.text_silver,
                Color32::TRANSPARENT,
                "Đổi Sáng / Tối (⌘L)",
            )
            .clicked()
            {
                action = Some(SidebarAction::ToggleLightDark);
            }
        });
        return action;
    }

    // 2. FULL SIDEBAR: ANTIGRAVITY WORKSPACE TREE & AGENT LAUNCHER
    ui.vertical(|ui| {
        // Top Brand & Quick Collapse Row
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_filled(
                rect,
                Rounding::same(4.0),
                theme.accent_surface,
            );
            ui.painter()
                .circle_filled(rect.center(), 3.2, theme.accent_pop);

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("VICTINI")
                    .color(theme.text_paper)
                    .strong()
                    .size(12.5),
            );
            ui.label(
                egui::RichText::new("Studio")
                    .color(theme.text_silver)
                    .size(11.5),
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
                if icon_btn(
                    ui,
                    Lucide::FOLDER_PLUS,
                    theme.text_silver,
                    Color32::TRANSPARENT,
                    "Thêm hoặc quản lý Projects / Workspaces",
                )
                .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(
                        CenterViewMode::ProjectsModal,
                    ));
                }
            });
        });

        ui.add_space(8.0);

        // Search Box + New Chat CTA
        ui.horizontal(|ui| {
            let search_w = (ui.available_width() - 34.0).max(80.0);
            Frame::none()
                .fill(theme.input_bg)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::SEARCH)
                                .color(theme.text_silver)
                                .size(11.0),
                        );
                        ui.add_sized(
                            vec2((search_w - 22.0).max(40.0), 18.0),
                            egui::TextEdit::singleline(search_query)
                                .hint_text("Tìm project, chat, agent...")
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
                    "Tạo phiên Chat mới trong Project đang mở",
                )
                .clicked()
                {
                    *show_agents_tab = false;
                    action = Some(SidebarAction::NewChatThread);
                }
            });
        });

        ui.add_space(8.0);

        // Clean 2-Segment Mode Switcher: [ Projects & Chats ] | [ Agents 1-1 ]
        Frame::none()
            .fill(theme.input_bg)
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(2.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let half_w = (ui.available_width() - 4.0) * 0.5;
                    let total_projects = project_mgr.projects.len();

                    let chats_sel = !*show_agents_tab;
                    let chats_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} Projects ({})",
                            Lucide::FOLDER,
                            total_projects
                        ))
                        .color(if chats_sel {
                            theme.text_paper
                        } else {
                            theme.text_silver
                        })
                        .strong()
                        .size(11.0),
                    )
                    .fill(if chats_sel {
                        theme.surface_2
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(4.5));

                    if ui
                        .add_sized(vec2(half_w, 24.0), chats_btn)
                        .on_hover_text("Danh sách các phiên Chat phân nhóm theo từng Project")
                        .clicked()
                    {
                        *show_agents_tab = false;
                    }

                    let agents_sel = *show_agents_tab;
                    let agents_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} Agents ({})",
                            Lucide::BOT,
                            ALL_AGENTS.len() + agents.len()
                        ))
                        .color(if agents_sel {
                            theme.text_paper
                        } else {
                            theme.text_silver
                        })
                        .strong()
                        .size(11.0),
                    )
                    .fill(if agents_sel {
                        theme.surface_2
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(4.5));

                    if ui
                        .add_sized(vec2(half_w, 24.0), agents_btn)
                        .on_hover_text("Chọn Agent Chat 1-1 chuyên biệt hoặc mở Skill Studio")
                        .clicked()
                    {
                        *show_agents_tab = true;
                    }
                });
            });

        ui.add_space(8.0);

        // 3. MAIN SCROLLABLE TREE: MULTI-PROJECT WORKSPACE CHATS OR AGENTS
        let q = search_query.trim().to_lowercase();
        let bottom_reserve = 78.0;
        ScrollArea::vertical()
            .id_salt("victini_multi_project_sidebar_scroll")
            .auto_shrink([false, false])
            .max_height((ui.available_height() - bottom_reserve).max(120.0))
            .show(ui, |ui| {
                if !*show_agents_tab {
                    // =========================================================
                    // TAB A: MULTI-PROJECT -> CHAT THREADS TREE (AGY STYLE)
                    // =========================================================
                    for proj in &project_mgr.projects {
                        let is_active_proj = proj.path == project_mgr.active_project;
                        let mgr = if is_active_proj {
                            Some(active_chat_mgr)
                        } else {
                            project_chats.get(&proj.path)
                        };

                        let threads_slice = mgr.map(|m| m.threads.as_slice()).unwrap_or(&[]);
                        let active_idx = mgr.map(|m| m.active_thread_idx).unwrap_or(0);

                        // Filter check when searching
                        let proj_matches_q =
                            q.is_empty() || proj.name.to_lowercase().contains(&q);
                        let any_thread_matches_q = threads_slice.iter().any(|t| {
                            t.title.to_lowercase().contains(&q)
                                || t.branch.to_lowercase().contains(&q)
                                || t.agent_mode.short_tag().to_lowercase().contains(&q)
                        });

                        if !q.is_empty() && !proj_matches_q && !any_thread_matches_q {
                            continue;
                        }

                        // Project Group Header Row
                        let is_expanded = proj.expanded || !q.is_empty();
                        let chev = if is_expanded {
                            Lucide::CHEVRON_DOWN
                        } else {
                            Lucide::CHEVRON_RIGHT
                        };
                        let hdr_bg = if is_active_proj {
                            theme.surface_1
                        } else {
                            Color32::TRANSPARENT
                        };
                        let hdr_border = if is_active_proj {
                            theme.border
                        } else {
                            Color32::TRANSPARENT
                        };

                        Frame::none()
                            .fill(hdr_bg)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, hdr_border))
                            .inner_margin(Margin::symmetric(6.0, 4.5))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Expand/Collapse Project Toggle Button
                                    let folder_col = if is_active_proj {
                                        theme.accent_pop
                                    } else {
                                        theme.text_silver
                                    };
                                    let proj_branch = if is_active_proj {
                                        branch_name
                                    } else {
                                        proj.last_branch.as_str()
                                    };

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} {} {}",
                                                    chev,
                                                    Lucide::FOLDER,
                                                    proj.name
                                                ))
                                                .color(if is_active_proj {
                                                    theme.text_paper
                                                } else {
                                                    theme.text_silver
                                                })
                                                .strong()
                                                .size(11.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .on_hover_text(format!(
                                            "Project: {}\nNhánh: {}\nClick để đóng/mở nhóm hội thoại",
                                            proj.path.display(),
                                            proj_branch
                                        ))
                                        .clicked()
                                    {
                                        action = Some(SidebarAction::ToggleProjectExpand(
                                            proj.path.clone(),
                                        ));
                                    }

                                    ui.label(
                                        egui::RichText::new(format!("({})", threads_slice.len()))
                                            .color(folder_col)
                                            .size(10.0),
                                    );

                                    // Right side of Project header: [+ New Chat in this project]
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if icon_btn(
                                            ui,
                                            Lucide::PLUS,
                                            if is_active_proj {
                                                theme.accent_pop
                                            } else {
                                                theme.text_silver
                                            },
                                            Color32::TRANSPARENT,
                                            &format!(
                                                "Tạo hội thoại mới trong project '{}'",
                                                proj.name
                                            ),
                                        )
                                        .clicked()
                                        {
                                            action = Some(SidebarAction::NewProjectChatThread(
                                                proj.path.clone(),
                                            ));
                                        }
                                    });
                                });
                            });

                        // Nested Threads inside this Project
                        if is_expanded {
                            ui.add_space(2.0);
                            let can_delete = threads_slice.len() > 1;

                            for (idx, t) in threads_slice.iter().enumerate() {
                                if !q.is_empty()
                                    && !proj_matches_q
                                    && !t.title.to_lowercase().contains(&q)
                                    && !t.branch.to_lowercase().contains(&q)
                                    && !t.agent_mode.short_tag().to_lowercase().contains(&q)
                                {
                                    continue;
                                }

                                let is_selected = is_active_proj && idx == active_idx;
                                let row_bg = if is_selected {
                                    theme.panel_bg
                                } else {
                                    Color32::TRANSPARENT
                                };
                                let row_border = if is_selected {
                                    theme.border_strong
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let mut delete_clicked = false;
                                ui.horizontal(|ui| {
                                    // Tree indent + vertical guide line
                                    let (indent_rect, _) = ui.allocate_exact_size(
                                        vec2(12.0, 36.0),
                                        egui::Sense::hover(),
                                    );
                                    let line_x = indent_rect.center().x;
                                    ui.painter().line_segment(
                                        [
                                            pos2(line_x, indent_rect.top()),
                                            pos2(line_x, indent_rect.bottom()),
                                        ],
                                        Stroke::new(
                                            if is_selected { 2.0 } else { 1.0 },
                                            if is_selected {
                                                theme.accent_pop
                                            } else {
                                                theme.border
                                            },
                                        ),
                                    );

                                    let card_w = (ui.available_width() - 2.0).max(100.0);
                                    let resp = Frame::none()
                                        .fill(row_bg)
                                        .rounding(Rounding::same(6.0))
                                        .stroke(Stroke::new(1.0, row_border))
                                        .inner_margin(Margin::symmetric(8.0, 5.0))
                                        .show(ui, |ui| {
                                            ui.set_width((card_w - 18.0).max(80.0));
                                            ui.vertical(|ui| {
                                                // Line 1: Thread Title + Delete button
                                                ui.horizontal(|ui| {
                                                    let short_title: String =
                                                        t.title.chars().take(24).collect();
                                                    let display_title =
                                                        if t.title.chars().count() > 24 {
                                                            format!("{}…", short_title)
                                                        } else {
                                                            short_title
                                                        };
                                                    ui.label(
                                                        egui::RichText::new(display_title)
                                                            .color(if is_selected {
                                                                theme.text_paper
                                                            } else {
                                                                theme.text_silver
                                                            })
                                                            .strong()
                                                            .size(11.3),
                                                    );

                                                    ui.with_layout(
                                                        Layout::right_to_left(Align::Center),
                                                        |ui| {
                                                            if can_delete
                                                                && ui
                                                                    .add(
                                                                        egui::Button::new(
                                                                            egui::RichText::new(
                                                                                Lucide::TRASH,
                                                                            )
                                                                            .color(
                                                                                theme.text_silver,
                                                                            )
                                                                            .size(9.5),
                                                                        )
                                                                        .fill(Color32::TRANSPARENT),
                                                                    )
                                                                    .on_hover_text(
                                                                        "Xóa phiên hội thoại này",
                                                                    )
                                                                    .clicked()
                                                            {
                                                                delete_clicked = true;
                                                            }
                                                            ui.label(
                                                                egui::RichText::new(&t.updated_at)
                                                                    .color(theme.text_silver)
                                                                    .size(9.5),
                                                            );
                                                        },
                                                    );
                                                });

                                                // Line 2: Branch + Agent Mode
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "{} {}",
                                                            Lucide::GIT_BRANCH,
                                                            t.branch
                                                        ))
                                                        .color(theme.text_silver)
                                                        .size(9.5),
                                                    );
                                                    ui.with_layout(
                                                        Layout::right_to_left(Align::Center),
                                                        |ui| {
                                                            let tag_col = if is_selected {
                                                                theme.accent_pop
                                                            } else {
                                                                theme.text_silver
                                                            };
                                                            ui.label(
                                                                egui::RichText::new(
                                                                    t.agent_mode.short_tag(),
                                                                )
                                                                .color(tag_col)
                                                                .size(9.5),
                                                            );
                                                        },
                                                    );
                                                });
                                            });
                                        });

                                    if delete_clicked {
                                        action = Some(SidebarAction::DeleteProjectThread {
                                            project_path: proj.path.clone(),
                                            thread_idx: idx,
                                        });
                                    } else if resp
                                        .response
                                        .interact(egui::Sense::click())
                                        .clicked()
                                    {
                                        action = Some(SidebarAction::SelectProjectThread {
                                            project_path: proj.path.clone(),
                                            thread_idx: idx,
                                        });
                                    }
                                });
                                ui.add_space(2.0);
                            }
                            ui.add_space(4.0);
                        }
                        ui.add_space(3.0);
                    }

                    ui.add_space(4.0);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Thêm / Quản lý Project...",
                                    Lucide::FOLDER_PLUS
                                ))
                                .color(theme.text_silver)
                                .size(10.8),
                            )
                            .fill(theme.surface_1)
                            .stroke(Stroke::new(1.0, theme.border))
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        action = Some(SidebarAction::SwitchCenterView(
                            CenterViewMode::ProjectsModal,
                        ));
                    }
                } else {
                    // =========================================================
                    // TAB B: DEDICATED 1-ON-1 AGENT LAUNCHER & SKILL STUDIO CARD
                    // =========================================================
                    let swarm_resp = Frame::none()
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
                                        egui::RichText::new("Tự động chọn Agent theo câu hỏi")
                                            .color(theme.text_silver)
                                            .size(9.5),
                                    );
                                });
                            });
                        });
                    if swarm_resp.response.interact(egui::Sense::click()).clicked() {
                        *show_agents_tab = false;
                        action = Some(SidebarAction::SwitchToSwarmAuto);
                    }

                    ui.add_space(6.0);

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
                                        ui.label(
                                            egui::RichText::new(Lucide::CHEVRON_RIGHT)
                                                .color(theme.text_silver)
                                                .size(11.0),
                                        );
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
                                .color(theme.text_silver)
                                .strong()
                                .size(10.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::PLUS,
                                theme.text_paper,
                                theme.chip_bg,
                                "Tạo Subagent mới",
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
                    let studio_resp = Frame::none()
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
                                        egui::RichText::new("Quản lý, chỉnh sửa & nạp GitHub")
                                            .color(theme.text_silver)
                                            .size(9.2),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new(Lucide::CHEVRON_RIGHT)
                                            .color(theme.text_silver)
                                            .size(11.0),
                                    );
                                });
                            });
                        });
                    if studio_resp
                        .response
                        .interact(egui::Sense::click())
                        .clicked()
                    {
                        action = Some(SidebarAction::SwitchCenterView(
                            CenterViewMode::AgentSkillStudio,
                        ));
                    }
                }
            });

        // 4. Bottom Status Capsule (Compact Active Session Footer)
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(2.0);
            let active_th = active_chat_mgr.active_thread();

            Frame::none()
                .fill(theme.surface_1)
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::symmetric(8.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let dot_col = if is_streaming {
                            theme.accent_pop
                        } else {
                            theme.success
                        };
                        let (dot_rect, _) =
                            ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                        ui.painter().circle_filled(dot_rect.center(), 2.8, dot_col);

                        ui.vertical(|ui| {
                            let short_th: String = active_th.title.chars().take(22).collect();
                            ui.label(
                                egui::RichText::new(short_th)
                                    .color(theme.text_paper)
                                    .strong()
                                    .size(10.8),
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} · {} changes",
                                    active_th.agent_mode.short_tag(),
                                    file_changes.len()
                                ))
                                .color(theme.text_silver)
                                .size(9.5),
                            );
                        });
                    });
                });
        });
    });

    action
}
