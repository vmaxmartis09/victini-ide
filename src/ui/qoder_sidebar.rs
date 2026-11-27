use crate::ai::{ChatSessionManager, ElegansAgent};
use crate::config::{DiscoveredAgent, DiscoveredSkill, ProjectManager};
use crate::git::GitFileChange;
use crate::theme::{
    CanhTheme, Lucide, CTL_DEFAULT, FONT_CAPTION, FONT_FOOTNOTE, ROUND_CTL, SPACE_1, SPACE_2,
    SPACE_3,
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
    _is_streaming: bool,
    _show_agents_tab: &mut bool,
    search_query: &mut String,
    _file_changes: &[GitFileChange],
    _skills: &[DiscoveredSkill],
    _agents: &[DiscoveredAgent],
) -> Option<SidebarAction> {
    let mut action = None;
    ui.spacing_mut().item_spacing = vec2(SPACE_2, SPACE_1);

    // 1. MINI RAIL MODE (Compact 48px Rail on 4px Grid — Pure Icon Controls)
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
        });
        return action;
    }

    // 2. FULL SIDEBAR: SINGLE-PURPOSE PROJECT & CONVERSATION TREE
    //    (Zero duplicate Agent list, zero duplicate footer status, strict left/right alignment)
    ui.vertical(|ui| {
        // Top Row: Search Input (28px height, ROUND_CTL = 8px) + Primary New Thread Icon CTA (28x28)
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
                if icon_btn(
                    ui,
                    Lucide::PLUS,
                    theme.text_on_ink(),
                    theme.text_paper,
                    "Tạo hội thoại mới trong Project đang mở",
                )
                .clicked()
                {
                    action = Some(SidebarAction::NewChatThread);
                }
            });
        });

        ui.add_space(SPACE_3);

        // Main Scrollable Multi-Project -> Threads Tree (Uses full height cleanly)
        let q = search_query.trim().to_lowercase();
        ScrollArea::vertical()
            .id_salt("elegans_multi_project_sidebar_scroll")
            .auto_shrink([false, false])
            .max_height(ui.available_height().max(120.0))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(SPACE_1, SPACE_1);

                for proj in &project_mgr.projects {
                    let is_active_proj = proj.path == project_mgr.active_project;
                    let mgr = if is_active_proj {
                        Some(active_chat_mgr)
                    } else {
                        project_chats.get(&proj.path)
                    };

                    let threads_slice = mgr.map(|m| m.threads.as_slice()).unwrap_or(&[]);
                    let active_idx = mgr.map(|m| m.active_thread_idx).unwrap_or(0);

                    let proj_matches_q = q.is_empty() || proj.name.to_lowercase().contains(&q);
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

                    // Project Header Row (28px height, left-aligned with right quick-add icon)
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
                            action = Some(SidebarAction::ToggleProjectExpand(proj.path.clone()));
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
                                action =
                                    Some(SidebarAction::NewProjectChatThread(proj.path.clone()));
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
                                // 16px Tree Indent + Vertical Guide Line
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
                                            let display_title = if t.title.chars().count() > 22 {
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
                                                                    egui::RichText::new(Lucide::X)
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
                                        "{}\nNhánh: {} · Agent: {} · Cập nhật: {}",
                                        t.title,
                                        t.branch,
                                        t.agent_mode.short_tag(),
                                        t.updated_at
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
            });
    });

    action
}
