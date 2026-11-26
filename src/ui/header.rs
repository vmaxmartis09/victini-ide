use crate::theme::{CanhTheme, Lucide};
use crate::ui::layout_state::{CenterViewMode, LayoutState, ResponsiveBreakpoint};
use egui::{Align, Color32, Frame, Layout, Margin, Rounding, Ui};

pub enum HeaderAction {
    ToggleLeftSidebar,
    ToggleRightDock,
    SetCenterMode(CenterViewMode),
    OpenProjectsModal,
    LaunchGitLabBrowser,
    RefreshWorkspace,
    CycleCanhTheme,
}

pub fn project_badge_code(project_name: &str) -> String {
    let clean: String = project_name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(2)
        .collect();
    if clean.is_empty() {
        "PR".to_string()
    } else {
        clean.to_uppercase()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_transparent_header(
    ui: &mut Ui,
    theme: &CanhTheme,
    project_name: &str,
    branch_name: &str,
    active_thread_title: &str,
    running_bg_count: usize,
    layout: &LayoutState,
    breakpoint: ResponsiveBreakpoint,
) -> Option<HeaderAction> {
    let mut action = None;
    let badge = project_badge_code(project_name);

    ui.horizontal(|ui| {
        let pad = match breakpoint {
            ResponsiveBreakpoint::Compact => 12.0,
            _ => 68.0,
        };
        ui.add_space(pad);

        // Sidebar Toggle Icon Button (Lucide::PANEL_LEFT)
        let left_active = layout.show_left_sidebar && !layout.left_mini_mode;
        let left_btn = egui::Button::new(
            egui::RichText::new(Lucide::PANEL_LEFT)
                .size(13.0)
                .color(if left_active {
                    theme.accent_pop
                } else {
                    theme.text_silver
                }),
        )
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(4.0));

        if ui.add(left_btn).on_hover_text("Toggle Sidebar (⌘B)").clicked() {
            action = Some(HeaderAction::ToggleLeftSidebar);
        }

        ui.add_space(6.0);

        // Dynamic Project Badge [LA] landorus / branch / Thread Title
        let badge_resp = Frame::none()
            .fill(Color32::from_rgb(0x28, 0x10, 0x12))
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
        if badge_resp.response.interact(egui::Sense::click()).clicked() {
            action = Some(HeaderAction::OpenProjectsModal);
        }

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(project_name)
                        .color(theme.text_silver)
                        .size(11.5),
                )
                .fill(Color32::TRANSPARENT),
            )
            .on_hover_text("Click để chuyển Project / Workspace")
            .clicked()
        {
            action = Some(HeaderAction::OpenProjectsModal);
        }

        ui.label(egui::RichText::new("/").color(theme.text_silver).size(11.5));
        ui.label(
            egui::RichText::new(format!("{} {}", Lucide::GIT_BRANCH, branch_name))
                .color(theme.accent_pop)
                .size(11.0),
        );
        ui.label(egui::RichText::new("/").color(theme.text_silver).size(11.5));
        ui.label(
            egui::RichText::new(active_thread_title)
                .color(theme.text_paper)
                .strong()
                .size(12.0),
        );

        // Right side: T3 Code Top Window Controls
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right Dock Toggle (Lucide::PANEL_RIGHT)
            let right_btn = egui::Button::new(
                egui::RichText::new(Lucide::PANEL_RIGHT)
                    .size(13.0)
                    .color(if layout.show_right_dock {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    }),
            )
            .fill(Color32::TRANSPARENT)
            .rounding(Rounding::same(4.0));

            if ui
                .add(right_btn)
                .on_hover_text("Toggle BeeBox Git & Emulator Dock (⌘E)")
                .clicked()
            {
                action = Some(HeaderAction::ToggleRightDock);
            }

            // Split View (Lucide::COLUMNS)
            let split_active = layout.center_mode == CenterViewMode::SplitChatKanban;
            let split_btn = egui::Button::new(
                egui::RichText::new(Lucide::COLUMNS)
                    .size(13.0)
                    .color(if split_active {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    }),
            )
            .fill(Color32::TRANSPARENT)
            .rounding(Rounding::same(4.0));

            if ui
                .add(split_btn)
                .on_hover_text("Toggle Split Kanban / Chat")
                .clicked()
            {
                let next_mode = if split_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::SplitChatKanban
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // Kanban View Toggle (Lucide::KANBAN)
            let kanban_active = layout.center_mode == CenterViewMode::KanbanBoard;
            let kanban_label = if running_bg_count > 0 {
                format!("{} {}", Lucide::KANBAN, running_bg_count)
            } else {
                Lucide::KANBAN.to_string()
            };
            let kanban_btn = egui::Button::new(
                egui::RichText::new(kanban_label)
                    .size(12.5)
                    .color(if kanban_active || running_bg_count > 0 {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    }),
            )
            .fill(Color32::TRANSPARENT)
            .rounding(Rounding::same(4.0));

            if ui
                .add(kanban_btn)
                .on_hover_text("Toggle Swarm Kanban Board (⌘K)")
                .clicked()
            {
                let next_mode = if kanban_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::KanbanBoard
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // Code & Diff Editor Toggle (Lucide::CODE)
            let diff_active = layout.center_mode == CenterViewMode::GitDiffModal;
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(Lucide::CODE)
                            .color(if diff_active {
                                theme.accent_pop
                            } else {
                                theme.text_silver
                            })
                            .size(12.5),
                    )
                    .fill(Color32::TRANSPARENT),
                )
                .on_hover_text("Open Live File & Diff Editor")
                .clicked()
            {
                let next_mode = if diff_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::GitDiffModal
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // Refresh Git Status (Lucide::REFRESH)
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(Lucide::REFRESH)
                            .color(theme.text_silver)
                            .size(12.0),
                    )
                    .fill(Color32::TRANSPARENT),
                )
                .on_hover_text("Refresh Git Status & Workspace")
                .clicked()
            {
                action = Some(HeaderAction::RefreshWorkspace);
            }

            // Browser Profile Launcher (Lucide::GLOBE)
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(Lucide::GLOBE)
                            .color(theme.text_silver)
                            .size(12.0),
                    )
                    .fill(Color32::TRANSPARENT),
                )
                .on_hover_text("Launch GitLab / Isolated Browser Profile")
                .clicked()
            {
                action = Some(HeaderAction::LaunchGitLabBrowser);
            }
        });
    });

    action
}
