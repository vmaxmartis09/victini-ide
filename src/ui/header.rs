use crate::theme::{
    CanhTheme, Lucide, CTL_DEFAULT, FONT_CAPTION, FONT_FOOTNOTE, ROUND_CTL, ROUND_XS, SPACE_1,
    SPACE_2, SPACE_3,
};
use crate::ui::layout_state::{CenterViewMode, LayoutState, ResponsiveBreakpoint};
use egui::{pos2, vec2, Align, Color32, Frame, Layout, Margin, Rounding, Stroke, Ui};

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

/// Elegans h-7 (28px) default icon control with mandatory tooltip (VICIEO-UI-03) and 8px concentric radius.
pub fn icon_btn(
    ui: &mut Ui,
    icon: &str,
    color: Color32,
    bg: Color32,
    tooltip: &str,
) -> egui::Response {
    ui.add_sized(
        vec2(CTL_DEFAULT, CTL_DEFAULT),
        egui::Button::new(egui::RichText::new(icon).color(color).size(FONT_FOOTNOTE))
            .fill(bg)
            .rounding(Rounding::same(ROUND_CTL)),
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
        ui.spacing_mut().item_spacing = vec2(SPACE_2, SPACE_2);

        let left_pad = match breakpoint {
            ResponsiveBreakpoint::Compact => SPACE_3,
            _ => 72.0,
        };
        ui.add_space(left_pad);

        // 1. Left: Sidebar Toggle (28x28) + Flat Project/Branch Button
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
            "Đóng/mở thanh điều hướng bên trái (⌘B)",
        )
        .clicked()
        {
            action = Some(HeaderAction::ToggleLeftSidebar);
        }

        let proj_btn = egui::Button::new(
            egui::RichText::new(format!(
                "{} {}  ·  {} {}",
                Lucide::FOLDER,
                project_name,
                Lucide::GIT_BRANCH,
                branch_name
            ))
            .color(theme.text_paper)
            .size(FONT_FOOTNOTE),
        )
        .fill(theme.surface_1)
        .stroke(Stroke::new(1.0, theme.border))
        .rounding(Rounding::same(ROUND_CTL));

        if ui
            .add_sized(vec2(0.0, CTL_DEFAULT), proj_btn)
            .on_hover_text("Project & nhánh Git hiện tại (Click để quản lý danh sách Project)")
            .clicked()
        {
            action = Some(HeaderAction::OpenProjectsModal);
        }

        ui.add_space(SPACE_2);

        // 2. Center: Elegans Floating Pill Navigation (< 3% Brand Accent: Active tab uses surface_2 + 1 bottom accent underline)
        Frame::none()
            .fill(theme.surface_1)
            .rounding(Rounding::same(ROUND_CTL))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(SPACE_1))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(SPACE_1, 0.0);
                ui.horizontal(|ui| {
                    let kanban_lbl = if running_bg_count > 0 {
                        format!("{} Kanban · {}", Lucide::KANBAN, running_bg_count)
                    } else {
                        format!("{} Kanban", Lucide::KANBAN)
                    };

                    let tabs = [
                        (
                            CenterViewMode::ChatTimeline,
                            format!("{} Hội thoại", Lucide::MESSAGE_SQUARE),
                            "Hội thoại AI & Điều phối (Chat Timeline)",
                        ),
                        (
                            CenterViewMode::KanbanBoard,
                            kanban_lbl,
                            "Bảng điều phối tác vụ 4 cột (Swarm Kanban · ⌘K)",
                        ),
                        (
                            CenterViewMode::SplitChatKanban,
                            format!("{} Song song", Lucide::COLUMNS),
                            "Chia đôi màn hình: Hội thoại & Kanban",
                        ),
                        (
                            CenterViewMode::GitDiffModal,
                            format!("{} Code & Diff", Lucide::CODE),
                            "So sánh Git Diff & sửa file trực tiếp",
                        ),
                        (
                            CenterViewMode::AgentSkillStudio,
                            format!("{} Skills & Agents", Lucide::SPARKLES),
                            "Quản lý, chỉnh sửa & nạp Skills / Subagents",
                        ),
                    ];

                    for (mode, label, tip) in tabs {
                        let is_active = layout.center_mode == mode;
                        let btn = egui::Button::new(
                            egui::RichText::new(label)
                                .color(if is_active {
                                    theme.text_paper
                                } else {
                                    theme.text_silver
                                })
                                .strong()
                                .size(FONT_CAPTION),
                        )
                        .fill(if is_active {
                            theme.surface_2
                        } else {
                            Color32::TRANSPARENT
                        })
                        .rounding(Rounding::same(ROUND_XS));

                        let resp = ui.add_sized(vec2(0.0, 24.0), btn).on_hover_text(tip);
                        if is_active {
                            // Elegans < 3% brand accent rule: single 2px accent underline on active tab
                            let r = resp.rect;
                            ui.painter().line_segment(
                                [
                                    pos2(r.left() + SPACE_2, r.bottom() - 1.0),
                                    pos2(r.right() - SPACE_2, r.bottom() - 1.0),
                                ],
                                Stroke::new(2.0, theme.accent_pop),
                            );
                        }
                        if resp.clicked() {
                            action = Some(HeaderAction::SetCenterMode(mode));
                        }
                    }
                });
            });

        // 3. Right: Single-Source Utility Controls (No duplicate buttons anywhere else)
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing = vec2(SPACE_1, 0.0);

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
                "Đóng/mở thanh Inspector bên phải (⌘E)",
            )
            .clicked()
            {
                action = Some(HeaderAction::ToggleRightDock);
            }

            let settings_active = layout.center_mode == CenterViewMode::SettingsModal;
            if icon_btn(
                ui,
                Lucide::SETTINGS,
                if settings_active {
                    theme.text_paper
                } else {
                    theme.text_silver
                },
                if settings_active {
                    theme.surface_2
                } else {
                    Color32::TRANSPARENT
                },
                "Cài đặt hệ thống & Memory Reaper",
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

            let (theme_icon, theme_tip) = if theme.is_dark {
                (Lucide::SUN, "Chuyển sang giao diện Giấy Ấm Sáng (⌘L)")
            } else {
                (Lucide::MOON, "Chuyển sang giao diện Mực Sâu Tối (⌘L)")
            };
            if icon_btn(ui, theme_icon, theme.text_silver, Color32::TRANSPARENT, theme_tip)
                .clicked()
            {
                action = Some(HeaderAction::ToggleLightDark);
            }

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
