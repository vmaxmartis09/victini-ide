use crate::ai::{ChatSessionManager, ElegansAgent, ALL_AGENTS};
use crate::config::{DiscoveredAgent, DiscoveredSkill, ProjectManager};
use crate::git::GitFileChange;
use crate::theme::{
    CanhTheme, Lucide, CTL_DEFAULT, CTL_SM, FONT_BODY, FONT_CAPTION, FONT_FOOTNOTE, ROUND_CTL,
    ROUND_XS, SPACE_1, SPACE_2, SPACE_3,
};
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
    _active_mode: CenterViewMode,
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
    ui.spacing_mut().item_spacing = vec2(SPACE_2, SPACE_1);

    // 1. MINI RAIL MODE (Compact 48px Rail on 4px Grid)
    if is_mini {
        ui.vertical_centered(|ui| {
            ui.add_space(SPACE_2);
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
            ui.add_space(SPACE_2);
            // Primary CTA in Ink color per Elegans rule
            if icon_btn(
                ui,
                Lucide::PLUS,
                theme.text_on_ink(),
                theme.text_paper,
                "Tạo phiên hội thoại mới",
            )
            .clicked()
            {
                action = Some(SidebarAction::NewChatThread);
            }
            ui.add_space(SPACE_2);
            if icon_btn(
                ui,
                Lucide::FOLDER,
                if !*show_agents_tab {
                    theme.text_paper
                } else {
                    theme.text_silver
                },
                if !*show_agents_tab {
                    theme.surface_2
                } else {
                    Color32::TRANSPARENT
                },
                "Cây thư mục Project & Hội thoại",
            )
            .clicked()
            {
                *show_agents_tab = false;
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(SPACE_1);
            if icon_btn(
                ui,
                Lucide::BOT,
                if *show_agents_tab {
                    theme.text_paper
                } else {
                    theme.text_silver
                },
                if *show_agents_tab {
                    theme.surface_2
                } else {
                    Color32::TRANSPARENT
                },
                "Chọn Agent Chat 1-1 chuyên biệt",
            )
            .clicked()
            {
                *show_agents_tab = true;
                action = Some(SidebarAction::ToggleMiniMode);
            }
        });
        return action;
    }

    // 2. FULL SIDEBAR: ELEGANS FRAMED-SHEET WORKSPACE TREE (Zero nested-box clutter, strict 4px grid)
    ui.vertical(|ui| {
        // Search Input (28px height, ROUND_CTL = 8px) + Ink Primary New Chat CTA (28x28)
        ui.horizontal(|ui| {
            let search_w = (ui.available_width() - 36.0).max(80.0);
            Frame::none()
                .fill(theme.input_bg)
                .rounding(Rounding::same(ROUND_CTL))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::SEARCH)
                                .color(theme.text_silver)
                                .size(FONT_CAPTION),
                        );
                        ui.add_sized(
                            vec2((search_w - 28.0).max(40.0), 20.0),
                            egui::TextEdit::singleline(search_query)
                                .hint_text("Tìm project, hội thoại...")
                                .frame(false)
                                .text_color(theme.text_paper),
                        );
                    });
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Elegans Primary Button: Ink fill (`theme.text_paper`), contrasting text (`theme.text_on_ink()`)
                if icon_btn(
                    ui,
                    Lucide::PLUS,
                    theme.text_on_ink(),
                    theme.text_paper,
                    "Tạo hội thoại mới trong Project đang mở",
                )
                .clicked()
                {
                    *show_agents_tab = false;
                    action = Some(SidebarAction::NewChatThread);
                }
            });
        });

        ui.add_space(SPACE_2);

        // Clean 2-Segment Mode Switcher: [ Projects (N) ] | [ Agents (M) ] (Concentric 8px outer -> 4px inner)
        Frame::none()
            .fill(theme.surface_1)
            .rounding(Rounding::same(ROUND_CTL))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(SPACE_1))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(SPACE_1, 0.0);
                ui.horizontal(|ui| {
                    let half_w = (ui.available_width() - SPACE_1) * 0.5;
                    let total_projects = project_mgr.projects.len();

                    let chats_sel = !*show_agents_tab;
                    let chats_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} Projects · {}",
                            Lucide::FOLDER,
                            total_projects
                        ))
                        .color(if chats_sel {
                            theme.text_paper
                        } else {
                            theme.text_silver
                        })
                        .strong()
                        .size(FONT_CAPTION),
                    )
                    .fill(if chats_sel {
                        theme.surface_2
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(ROUND_XS));

                    if ui
                        .add_sized(vec2(half_w, CTL_SM), chats_btn)
                        .on_hover_text("Cây hội thoại phân nhóm theo từng Project")
                        .clicked()
                    {
                        *show_agents_tab = false;
                    }

                    let agents_sel = *show_agents_tab;
                    let agents_btn = egui::Button::new(
                        egui::RichText::new(format!(
                            "{} Agents · {}",
                            Lucide::BOT,
                            ALL_AGENTS.len() + agents.len()
                        ))
                        .color(if agents_sel {
                            theme.text_paper
                        } else {
                            theme.text_silver
                        })
                        .strong()
                        .size(FONT_CAPTION),
                    )
                    .fill(if agents_sel {
                        theme.surface_2
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(ROUND_XS));

                    if ui
                        .add_sized(vec2(half_w, CTL_SM), agents_btn)
                        .on_hover_text("Danh sách Agent Chat 1-1 chuyên biệt")
                        .clicked()
                    {
                        *show_agents_tab = true;
                    }
                });
            });

        ui.add_space(SPACE_3);

        // 3. MAIN SCROLLABLE LIST (Flat-at-Rest rows, strict 4px spacing, >= 11px typography)
        let q = search_query.trim().to_lowercase();
        let bottom_reserve = 48.0;
        ScrollArea::vertical()
            .id_salt("elegans_multi_project_sidebar_scroll")
            .auto_shrink([false, false])
            .max_height((ui.available_height() - bottom_reserve).max(120.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(SPACE_1, SPACE_1);

                if !*show_agents_tab {
                    // =========================================================
                    // TAB A: FLAT-AT-REST MULTI-PROJECT -> THREADS TREE
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

                        let is_expanded = proj.expanded || !q.is_empty();
                        let chev = if is_expanded {
                            Lucide::CHEVRON_DOWN
                        } else {
                            Lucide::CHEVRON_RIGHT
                        };
                        let proj_branch = if is_active_proj {
                            branch_name
                        } else {
                            proj.last_branch.as_str()
                        };

                        // Clean Flat Project Group Row (28px height, no border box clutter)
                        ui.horizontal(|ui| {
                            let label_col = if is_active_proj {
                                theme.text_paper
                            } else {
                                theme.text_silver
                            };
                            let btn = egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} {} {}",
                                    chev,
                                    Lucide::FOLDER,
                                    proj.name
                                ))
                                .color(label_col)
                                .strong()
                                .size(FONT_FOOTNOTE),
                            )
                            .fill(Color32::TRANSPARENT)
                            .rounding(Rounding::same(ROUND_CTL));

                            if ui
                                .add_sized(vec2(0.0, CTL_DEFAULT), btn)
                                .on_hover_text(format!(
                                    "{}\nNhánh: {} · {} hội thoại",
                                    proj.path.display(),
                                    proj_branch,
                                    threads_slice.len()
                                ))
                                .clicked()
                            {
                                action = Some(SidebarAction::ToggleProjectExpand(
                                    proj.path.clone(),
                                ));
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if icon_btn(
                                    ui,
                                    Lucide::PLUS,
                                    theme.text_silver,
                                    Color32::TRANSPARENT,
                                    &format!("Hội thoại mới trong '{}'", proj.name),
                                )
                                .clicked()
                                {
                                    action = Some(SidebarAction::NewProjectChatThread(
                                        proj.path.clone(),
                                    ));
                                }
                            });
                        });

                        // Nested Single-Line Threads (28px height, clean tree guide line)
                        if is_expanded {
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
                                    theme.surface_2
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let mut delete_clicked = false;
                                ui.horizontal(|ui| {
                                    // 16px Tree Indent + 1px Vertical Guide Line (< 3% accent only on selected thread)
                                    let (indent_rect, _) = ui.allocate_exact_size(
                                        vec2(16.0, CTL_DEFAULT),
                                        egui::Sense::hover(),
                                    );
                                    let line_x = indent_rect.center().x;
                                    ui.painter().line_segment(
                                        [
                                            pos2(line_x, indent_rect.top() + 2.0),
                                            pos2(line_x, indent_rect.bottom() - 2.0),
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

                                    let row_w = ui.available_width().max(80.0);
                                    let resp = Frame::none()
                                        .fill(row_bg)
                                        .rounding(Rounding::same(ROUND_CTL))
                                        .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
                                        .show(ui, |ui| {
                                            ui.set_width((row_w - 16.0).max(64.0));
                                            ui.horizontal(|ui| {
                                                let short_title: String =
                                                    t.title.chars().take(22).collect();
                                                let display_title =
                                                    if t.title.chars().count() > 22 {
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
                                                        .size(FONT_FOOTNOTE),
                                                );

                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        if can_delete
                                                            && ui
                                                                .add(
                                                                    egui::Button::new(
                                                                        egui::RichText::new(
                                                                            Lucide::X,
                                                                        )
                                                                        .color(theme.text_silver)
                                                                        .size(FONT_CAPTION),
                                                                    )
                                                                    .fill(Color32::TRANSPARENT),
                                                                )
                                                                .on_hover_text(
                                                                    "Xóa phiên hội thoại này",
                                                                )
                                                                .clicked()
                                                        {
                                                            delete_clicked = true;
                                                        } else {
                                                            ui.label(
                                                                egui::RichText::new(&t.updated_at)
                                                                    .color(theme.text_silver)
                                                                    .size(FONT_CAPTION),
                                                            );
                                                        }
                                                    },
                                                );
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
                                        .on_hover_text(format!(
                                            "{}\nNhánh: {} · Agent: {}",
                                            t.title,
                                            t.branch,
                                            t.agent_mode.short_tag()
                                        ))
                                        .clicked()
                                    {
                                        action = Some(SidebarAction::SelectProjectThread {
                                            project_path: proj.path.clone(),
                                            thread_idx: idx,
                                        });
                                    }
                                });
                            }
                        }
                        ui.add_space(SPACE_1);
                    }

                    // Elegans Smart Empty/Add State: Dashed-style subtle action row at end of list
                    ui.add_space(SPACE_2);
                    if ui
                        .add_sized(
                            vec2(ui.available_width(), CTL_DEFAULT),
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Thêm Project vào Workspace...",
                                    Lucide::FOLDER_PLUS
                                ))
                                .color(theme.text_silver)
                                .size(FONT_CAPTION),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, theme.border))
                            .rounding(Rounding::same(ROUND_CTL)),
                        )
                        .clicked()
                    {
                        action = Some(SidebarAction::SwitchCenterView(
                            CenterViewMode::ProjectsModal,
                        ));
                    }
                } else {
                    // =========================================================
                    // TAB B: CLEAN FLAT AGENT 1-ON-1 LIST (Sentence case, >= 11px)
                    // =========================================================
                    let swarm_resp = Frame::none()
                        .fill(theme.surface_1)
                        .rounding(Rounding::same(ROUND_CTL))
                        .inner_margin(Margin::symmetric(SPACE_2, SPACE_2))
                        .show(ui, |ui| {
                            ui.set_width((ui.available_width() - 16.0).max(80.0));
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(Lucide::SPARKLES)
                                        .color(theme.accent_pop)
                                        .size(FONT_BODY),
                                );
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Swarm Auto-Router")
                                            .color(theme.text_paper)
                                            .strong()
                                            .size(FONT_FOOTNOTE),
                                    );
                                    ui.label(
                                        egui::RichText::new("Tự động điều phối Agent theo câu hỏi")
                                            .color(theme.text_silver)
                                            .size(FONT_CAPTION),
                                    );
                                });
                            });
                        });
                    if swarm_resp.response.interact(egui::Sense::click()).clicked() {
                        *show_agents_tab = false;
                        action = Some(SidebarAction::SwitchToSwarmAuto);
                    }

                    ui.add_space(SPACE_2);

                    for &ag in ALL_AGENTS {
                        if !q.is_empty()
                            && !ag.name().to_lowercase().contains(&q)
                            && !ag.title().to_lowercase().contains(&q)
                        {
                            continue;
                        }
                        let card_resp = Frame::none()
                            .fill(Color32::TRANSPARENT)
                            .rounding(Rounding::same(ROUND_CTL))
                            .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
                            .show(ui, |ui| {
                                ui.set_width((ui.available_width() - 16.0).max(80.0));
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(ag.icon())
                                            .color(theme.text_silver)
                                            .size(FONT_BODY),
                                    );
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(ag.name())
                                                .color(theme.text_paper)
                                                .strong()
                                                .size(FONT_FOOTNOTE),
                                        );
                                        ui.label(
                                            egui::RichText::new(ag.title())
                                                .color(theme.text_silver)
                                                .size(FONT_CAPTION),
                                        );
                                    });
                                });
                            });
                        if card_resp.response.interact(egui::Sense::click()).clicked() {
                            *show_agents_tab = false;
                            action = Some(SidebarAction::OpenDedicatedCoreAgent(ag));
                        }
                    }

                    if !agents.is_empty() {
                        ui.add_space(SPACE_2);
                        // Sentence case section label (Zero TASTE-WAVE2-EYEBROW all-caps slop)
                        ui.label(
                            egui::RichText::new(format!("Subagents tùy chỉnh ({})", agents.len()))
                                .color(theme.text_silver)
                                .strong()
                                .size(FONT_CAPTION),
                        );

                        for (idx, ag) in agents.iter().enumerate() {
                            if !q.is_empty()
                                && !ag.name.to_lowercase().contains(&q)
                                && !ag.role.to_lowercase().contains(&q)
                            {
                                continue;
                            }
                            let card_resp = Frame::none()
                                .fill(Color32::TRANSPARENT)
                                .rounding(Rounding::same(ROUND_CTL))
                                .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
                                .show(ui, |ui| {
                                    ui.set_width((ui.available_width() - 16.0).max(80.0));
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("@{}", ag.name))
                                                .color(theme.text_paper)
                                                .strong()
                                                .size(FONT_FOOTNOTE),
                                        );
                                        ui.with_layout(
                                            Layout::right_to_left(Align::Center),
                                            |ui| {
                                                if icon_btn(
                                                    ui,
                                                    Lucide::EDIT,
                                                    theme.text_silver,
                                                    Color32::TRANSPARENT,
                                                    "Chỉnh sửa Subagent này",
                                                )
                                                .clicked()
                                                {
                                                    action = Some(SidebarAction::InspectAgent(idx));
                                                }
                                            },
                                        );
                                    });
                                    ui.label(
                                        egui::RichText::new(&ag.role)
                                            .color(theme.text_silver)
                                            .size(FONT_CAPTION),
                                    );
                                });
                            if card_resp.response.interact(egui::Sense::click()).clicked() {
                                *show_agents_tab = false;
                                action = Some(SidebarAction::OpenDedicatedCustomAgent(idx));
                            }
                        }
                    }

                    ui.add_space(SPACE_2);
                    if ui
                        .add_sized(
                            vec2(ui.available_width(), CTL_DEFAULT),
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Quản lý {} Skills & Subagents...",
                                    Lucide::WAND,
                                    skills.len()
                                ))
                                .color(theme.text_silver)
                                .size(FONT_CAPTION),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, theme.border))
                            .rounding(Rounding::same(ROUND_CTL)),
                        )
                        .clicked()
                    {
                        action = Some(SidebarAction::SwitchCenterView(
                            CenterViewMode::AgentSkillStudio,
                        ));
                    }
                }
            });

        // 4. Minimalist Single-Line Status Footer (No bulky framed card)
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(SPACE_1);
            let active_th = active_chat_mgr.active_thread();
            ui.horizontal(|ui| {
                let dot_col = if is_streaming {
                    theme.accent_pop
                } else {
                    theme.success
                };
                let (dot_rect, _) =
                    ui.allocate_exact_size(vec2(SPACE_2, SPACE_2), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 3.0, dot_col);

                ui.label(
                    egui::RichText::new(active_th.agent_mode.short_tag())
                        .color(theme.text_paper)
                        .strong()
                        .size(FONT_CAPTION),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{} thay đổi", file_changes.len()))
                            .color(theme.text_silver)
                            .size(FONT_CAPTION),
                    );
                });
            });
        });
    });

    action
}
