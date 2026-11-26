use crate::theme::{CanhTheme, Lucide};
use crate::ui::layout_state::{CenterViewMode, LayoutState, ResponsiveBreakpoint};
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, Ui};

pub enum HeaderAction {
    ToggleLeftSidebar,
    ToggleRightDock,
    SetCenterMode(CenterViewMode),
    OpenProjectsModal,
    LaunchGitLabBrowser,
    RefreshWorkspace,
    ToggleLightDark,
    CycleCanhTheme,
    RunSecurityDiagnostics,
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

pub fn icon_btn(
    ui: &mut Ui,
    icon: &str,
    color: Color32,
    bg: Color32,
    tooltip: &str,
) -> egui::Response {
    ui.add_sized(
        vec2(24.0, 24.0),
        egui::Button::new(egui::RichText::new(icon).color(color).size(13.0))
            .fill(bg)
            .rounding(Rounding::same(5.0)),
    )
    .on_hover_text(tooltip)
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
        if icon_btn(
            ui,
            Lucide::PANEL_LEFT,
            if left_active {
                theme.accent_pop
            } else {
                theme.text_silver
            },
            Color32::TRANSPARENT,
            "Đóng/mở Sidebar trái (⌘B)",
        )
        .clicked()
        {
            action = Some(HeaderAction::ToggleLeftSidebar);
        }

        ui.add_space(4.0);

        // Dynamic Project Badge [LA] landorus / branch / Thread Title
        let badge_resp = Frame::none()
            .fill(theme.tag_red_bg)
            .rounding(Rounding::same(4.0))
            .inner_margin(Margin::symmetric(5.0, 1.5))
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

        // Right side: 100% Icon-First Top Window Controls
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // 1. Right Dock Toggle (Lucide::PANEL_RIGHT)
            if icon_btn(
                ui,
                Lucide::PANEL_RIGHT,
                if layout.show_right_dock {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                Color32::TRANSPARENT,
                "Đóng/mở BeeBox Git & Emulator Dock (⌘E)",
            )
            .clicked()
            {
                action = Some(HeaderAction::ToggleRightDock);
            }

            // 2. Split View (Lucide::COLUMNS)
            let split_active = layout.center_mode == CenterViewMode::SplitChatKanban;
            if icon_btn(
                ui,
                Lucide::COLUMNS,
                if split_active {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                Color32::TRANSPARENT,
                "Chia đôi màn hình Chat & Swarm Kanban",
            )
            .clicked()
            {
                let next_mode = if split_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::SplitChatKanban
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // 3. Kanban View Toggle (Lucide::KANBAN)
            let kanban_active = layout.center_mode == CenterViewMode::KanbanBoard;
            let kanban_label = if running_bg_count > 0 {
                format!("{}{}", Lucide::KANBAN, running_bg_count)
            } else {
                Lucide::KANBAN.to_string()
            };
            if icon_btn(
                ui,
                &kanban_label,
                if kanban_active || running_bg_count > 0 {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                Color32::TRANSPARENT,
                "Bảng điều phối Swarm Kanban Board (⌘K)",
            )
            .clicked()
            {
                let next_mode = if kanban_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::KanbanBoard
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // 4. Code & Diff Editor Toggle (Lucide::CODE)
            let diff_active = layout.center_mode == CenterViewMode::GitDiffModal;
            if icon_btn(
                ui,
                Lucide::CODE,
                if diff_active {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                Color32::TRANSPARENT,
                "Trình sửa code trực tiếp & xem Git Diff",
            )
            .clicked()
            {
                let next_mode = if diff_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::GitDiffModal
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // 5. Light / Dark Mode 1-Click Toggle (Lucide::SUN / Lucide::MOON)
            let theme_icon = if theme.is_dark {
                Lucide::SUN
            } else {
                Lucide::MOON
            };
            let theme_tip = if theme.is_dark {
                "Chuyển sang giao diện Sáng (Light Mode · ⌘L)"
            } else {
                "Chuyển sang giao diện Tối (OLED Dark Mode · ⌘L)"
            };
            if icon_btn(
                ui,
                theme_icon,
                theme.accent_pop,
                theme.chip_bg,
                theme_tip,
            )
            .clicked()
            {
                action = Some(HeaderAction::ToggleLightDark);
            }

            // 6. Elegans Security & AST Blast Audit (Lucide::SHIELD)
            if icon_btn(
                ui,
                Lucide::SHIELD,
                theme.text_silver,
                Color32::TRANSPARENT,
                "Quét bảo mật Elegans (Env Secret Guard & AST Blast Radius)",
            )
            .clicked()
            {
                action = Some(HeaderAction::RunSecurityDiagnostics);
            }

            // 7. Refresh Git Status (Lucide::REFRESH)
            if icon_btn(
                ui,
                Lucide::REFRESH,
                theme.text_silver,
                Color32::TRANSPARENT,
                "Làm mới trạng thái Git & Workspace",
            )
            .clicked()
            {
                action = Some(HeaderAction::RefreshWorkspace);
            }

            // 8. Browser Profile Launcher (Lucide::GLOBE)
            if icon_btn(
                ui,
                Lucide::GLOBE,
                theme.text_silver,
                Color32::TRANSPARENT,
                "Mở GitLab MR / Trình duyệt cách ly (Bảo vệ Dad Profile)",
            )
            .clicked()
            {
                action = Some(HeaderAction::LaunchGitLabBrowser);
            }
        });
    });

    action
}
