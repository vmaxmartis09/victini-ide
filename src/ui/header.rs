use crate::theme::{CanhTheme, Lucide};
use crate::ui::layout_state::{CenterViewMode, LayoutState, ResponsiveBreakpoint};
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, Stroke, Ui};

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
        vec2(26.0, 26.0),
        egui::Button::new(egui::RichText::new(icon).color(color).size(13.0))
            .fill(bg)
            .rounding(Rounding::same(6.0)),
    )
    .on_hover_text(tooltip)
}

#[allow(clippy::too_many_arguments)]
pub fn render_transparent_header(
    ui: &mut Ui,
    theme: &CanhTheme,
    project_name: &str,
    branch_name: &str,
    _active_thread_title: &str,
    running_bg_count: usize,
    layout: &LayoutState,
    breakpoint: ResponsiveBreakpoint,
) -> Option<HeaderAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        let pad = match breakpoint {
            ResponsiveBreakpoint::Compact => 12.0,
            _ => 72.0,
        };
        ui.add_space(pad);

        // 1. Left: Sidebar Toggle + Clean Active Project & Branch Pill
        let left_active = layout.show_left_sidebar && !layout.left_mini_mode;
        if icon_btn(
            ui,
            Lucide::PANEL_LEFT,
            if left_active {
                theme.text_paper
            } else {
                theme.text_silver
            },
            if left_active {
                theme.surface_2
            } else {
                Color32::TRANSPARENT
            },
            "Đóng/mở Sidebar trái (⌘B)",
        )
        .clicked()
        {
            action = Some(HeaderAction::ToggleLeftSidebar);
        }

        ui.add_space(6.0);

        // Active Workspace & Git Branch Pill (Click to manage projects)
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(format!(
                        "{} {}  ·  {} {}",
                        Lucide::FOLDER,
                        project_name,
                        Lucide::GIT_BRANCH,
                        branch_name
                    ))
                    .color(theme.text_paper)
                    .strong()
                    .size(11.5),
                )
                .fill(theme.surface_1)
                .stroke(Stroke::new(1.0, theme.border))
                .rounding(Rounding::same(6.0)),
            )
            .on_hover_text("Project & Nhánh Git đang mở (Click để quản lý danh sách Project)")
            .clicked()
        {
            action = Some(HeaderAction::OpenProjectsModal);
        }

        ui.add_space(12.0);

        // 2. Center: Agy Segmented View Switcher (Clear Icon + Label Tabs)
        Frame::none()
            .fill(theme.bg_ink)
            .rounding(Rounding::same(7.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::symmetric(3.0, 2.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let kanban_lbl = if running_bg_count > 0 {
                        format!("{} Kanban ({})", Lucide::KANBAN, running_bg_count)
                    } else {
                        format!("{} Kanban", Lucide::KANBAN)
                    };

                    let tabs = [
                        (
                            CenterViewMode::ChatTimeline,
                            format!("{} Chat", Lucide::MESSAGE_SQUARE),
                            "Khung hội thoại AI & Điều phối (Chat Timeline)",
                        ),
                        (
                            CenterViewMode::KanbanBoard,
                            kanban_lbl,
                            "Bảng điều phối tác vụ 4 cột (Swarm Kanban · ⌘K)",
                        ),
                        (
                            CenterViewMode::SplitChatKanban,
                            format!("{} Song song", Lucide::COLUMNS),
                            "Chế độ chia đôi màn hình: Vừa Chat vừa xem Kanban",
                        ),
                        (
                            CenterViewMode::GitDiffModal,
                            format!("{} Code & Diff", Lucide::CODE),
                            "Xem Git Diff & Sửa file trực tiếp",
                        ),
                        (
                            CenterViewMode::AgentSkillStudio,
                            format!("{} Skills & Agents", Lucide::SPARKLES),
                            "Quản lý, chỉnh sửa & nạp Skills / Subagents từ GitHub",
                        ),
                    ];

                    for (mode, label, tip) in tabs {
                        let is_active = layout.center_mode == mode;
                        let btn = egui::Button::new(
                            egui::RichText::new(label)
                                .color(if is_active {
                                    theme.on_accent
                                } else {
                                    theme.text_silver
                                })
                                .strong()
                                .size(11.2),
                        )
                        .fill(if is_active {
                            theme.accent_pop
                        } else {
                            Color32::TRANSPARENT
                        })
                        .rounding(Rounding::same(5.0));

                        if ui.add(btn).on_hover_text(tip).clicked() {
                            action = Some(HeaderAction::SetCenterMode(mode));
                        }
                    }
                });
            });

        // 3. Right side: Clean Utility Controls
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right Dock Toggle (Lucide::PANEL_RIGHT)
            if icon_btn(
                ui,
                Lucide::PANEL_RIGHT,
                if layout.show_right_dock {
                    theme.text_paper
                } else {
                    theme.text_silver
                },
                if layout.show_right_dock {
                    theme.surface_2
                } else {
                    Color32::TRANSPARENT
                },
                "Đóng/mở Thanh công cụ Git & Inspector bên phải (⌘E)",
            )
            .clicked()
            {
                action = Some(HeaderAction::ToggleRightDock);
            }

            // Settings Button
            let settings_active = layout.center_mode == CenterViewMode::SettingsModal;
            if icon_btn(
                ui,
                Lucide::SETTINGS,
                if settings_active {
                    theme.accent_pop
                } else {
                    theme.text_silver
                },
                if settings_active {
                    theme.surface_2
                } else {
                    Color32::TRANSPARENT
                },
                "Cài đặt hệ thống & Giao diện",
            )
            .clicked()
            {
                let next = if settings_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::SettingsModal
                };
                action = Some(HeaderAction::SetCenterMode(next));
            }

            // Light / Dark Mode 1-Click Toggle Pill
            let (theme_icon, theme_lbl) = if theme.is_dark {
                (Lucide::SUN, "Sáng")
            } else {
                (Lucide::MOON, "Tối")
            };
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(format!("{} {}", theme_icon, theme_lbl))
                            .color(theme.text_paper)
                            .size(11.0),
                    )
                    .fill(theme.surface_1)
                    .stroke(Stroke::new(1.0, theme.border))
                    .rounding(Rounding::same(6.0)),
                )
                .on_hover_text("Đổi chế độ Sáng Ấm / Tối Ấm (⌘L)")
                .clicked()
            {
                action = Some(HeaderAction::ToggleLightDark);
            }

            // Refresh Git Status (Lucide::REFRESH)
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
        });
    });

    action
}
