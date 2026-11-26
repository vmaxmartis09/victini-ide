use crate::theme::CanhTheme;
use crate::ui::layout_state::{CenterViewMode, LayoutState, ResponsiveBreakpoint};
use egui::{Align, Color32, Frame, Layout, Margin, Rounding, Stroke, Ui};

pub enum HeaderAction {
    ToggleLeftSidebar,
    ToggleRightDock,
    SetCenterMode(CenterViewMode),
    OpenProjectsModal,
    LaunchGitLabBrowser,
    RefreshWorkspace,
    CycleCanhTheme,
}

pub fn render_transparent_header(
    ui: &mut Ui,
    theme: &CanhTheme,
    project_name: &str,
    branch_name: &str,
    active_agent_title: &str,
    running_bg_count: usize,
    layout: &LayoutState,
    breakpoint: ResponsiveBreakpoint,
) -> Option<HeaderAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        // macOS Traffic Light Padding (shorter in compact window)
        let pad = match breakpoint {
            ResponsiveBreakpoint::Compact => 12.0,
            _ => 66.0,
        };
        ui.add_space(pad);

        // 1. Left Sidebar Toggle Button [◧]
        let left_active = layout.show_left_sidebar && !layout.left_mini_mode;
        let left_btn = egui::Button::new(
            egui::RichText::new("◧")
                .size(13.0)
                .color(if left_active { theme.accent_pop } else { theme.text_silver }),
        )
        .fill(if left_active {
            Color32::from_rgb(0x18, 0x1C, 0x26)
        } else {
            Color32::TRANSPARENT
        })
        .rounding(Rounding::same(5.0));

        if ui
            .add(left_btn)
            .on_hover_text("Đóng/Mở Sidebar Trái (⌘B)")
            .clicked()
        {
            action = Some(HeaderAction::ToggleLeftSidebar);
        }

        ui.add_space(4.0);

        // 2. App Brand + Active Project Selector + Branch Pill
        ui.label(
            egui::RichText::new("⚡ Victini")
                .color(theme.text_paper)
                .strong()
                .size(13.0),
        );
        ui.label(egui::RichText::new("/").color(theme.text_silver).size(12.0));

        if ui
            .button(
                egui::RichText::new(format!("📁 {}", project_name))
                    .color(theme.text_paper)
                    .strong()
                    .size(12.0),
            )
            .on_hover_text("Chuyển đổi hoặc thêm Project / Git Repository")
            .clicked()
        {
            action = Some(HeaderAction::OpenProjectsModal);
        }

        ui.label(egui::RichText::new("/").color(theme.text_silver).size(12.0));

        Frame::none()
            .fill(Color32::from_rgb(0x14, 0x18, 0x22))
            .rounding(Rounding::same(5.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x24, 0x2A, 0x38)))
            .inner_margin(Margin::symmetric(7.0, 2.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!(" {}", branch_name))
                        .color(theme.accent_pop)
                        .strong()
                        .size(11.5),
                );
            });

        // 3. Right side Responsive Controls & View Switchers
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right BeeBox Dock Toggle [◨]
            let right_btn = egui::Button::new(
                egui::RichText::new("◨")
                    .size(13.0)
                    .color(if layout.show_right_dock {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    }),
            )
            .fill(if layout.show_right_dock {
                Color32::from_rgb(0x18, 0x1C, 0x26)
            } else {
                Color32::TRANSPARENT
            })
            .rounding(Rounding::same(5.0));

            if ui
                .add(right_btn)
                .on_hover_text("Đóng/Mở BeeBox & Remote Emulator Dock (⌘E)")
                .clicked()
            {
                action = Some(HeaderAction::ToggleRightDock);
            }

            // Browser Launcher
            if ui
                .button(
                    egui::RichText::new("🌐 GitLab")
                        .size(11.0)
                        .color(theme.text_paper),
                )
                .on_hover_text("Mở trình duyệt Ego Lite / Chrome với Profile cô lập (Bảo vệ Dad profile)")
                .clicked()
            {
                action = Some(HeaderAction::LaunchGitLabBrowser);
            }

            // Sync Git
            if ui
                .button(
                    egui::RichText::new("🔄 Sync")
                        .size(11.0)
                        .color(theme.text_silver),
                )
                .on_hover_text("Làm mới trạng thái Git, Worktrees & Skills")
                .clicked()
            {
                action = Some(HeaderAction::RefreshWorkspace);
            }

            // Center Mode Switcher Pills: [💬 Chat] [📊 Kanban] [◫ Split]
            let modes = [
                (CenterViewMode::SplitChatKanban, "◫ Split"),
                (CenterViewMode::KanbanBoard, "📊 Kanban"),
                (CenterViewMode::ChatTimeline, "💬 Chat"),
            ];

            for (m, label) in modes {
                if breakpoint == ResponsiveBreakpoint::Compact && m == CenterViewMode::SplitChatKanban {
                    continue;
                }
                let is_sel = layout.center_mode == m;
                let btn = egui::Button::new(
                    egui::RichText::new(label)
                        .size(11.0)
                        .strong()
                        .color(if is_sel { Color32::BLACK } else { theme.text_paper }),
                )
                .fill(if is_sel {
                    theme.accent_pop
                } else {
                    Color32::from_rgb(0x16, 0x19, 0x22)
                })
                .rounding(Rounding::same(5.0));

                if ui.add(btn).clicked() {
                    action = Some(HeaderAction::SetCenterMode(m));
                }
            }

            // Canh Giờ Theme Pill (hidden on very small compact width)
            if breakpoint != ResponsiveBreakpoint::Compact {
                if ui
                    .button(
                        egui::RichText::new(format!("🌗 {}", theme.name))
                            .size(10.5)
                            .color(theme.text_silver),
                    )
                    .on_hover_text("Đổi bảng màu 12 Canh Giờ (Elegans Theme)")
                    .clicked()
                {
                    action = Some(HeaderAction::CycleCanhTheme);
                }

                // Active Swarm / Background count indicator
                let bg_badge = if running_bg_count > 0 {
                    format!("⚡ {} BG Running · {}", running_bg_count, active_agent_title)
                } else {
                    format!("Swarm: {}", active_agent_title)
                };
                ui.label(
                    egui::RichText::new(bg_badge)
                        .color(if running_bg_count > 0 {
                            theme.accent_pop
                        } else {
                            theme.text_silver
                        })
                        .size(11.0),
                );
            }
        });
    });
    action
}
