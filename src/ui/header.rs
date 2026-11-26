use crate::theme::CanhTheme;
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

pub fn render_transparent_header(
    ui: &mut Ui,
    theme: &CanhTheme,
    _project_name: &str,
    _branch_name: &str,
    _active_agent_title: &str,
    _running_bg_count: usize,
    layout: &LayoutState,
    breakpoint: ResponsiveBreakpoint,
) -> Option<HeaderAction> {
    let mut action = None;

    // T3 Code Top Bar:
    // Left: macOS Traffic Lights space + Breadcrumb: [PN postplan] / Add Expiring Draft Share Links
    // Right: Window controls icons [◧] [◫] [◨]
    ui.horizontal(|ui| {
        let pad = match breakpoint {
            ResponsiveBreakpoint::Compact => 12.0,
            _ => 68.0,
        };
        ui.add_space(pad);

        // Sidebar Toggle Icon Button
        let left_active = layout.show_left_sidebar && !layout.left_mini_mode;
        let left_btn = egui::Button::new(
            egui::RichText::new("◧")
                .size(13.0)
                .color(if left_active { theme.accent_pop } else { theme.text_silver }),
        )
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(4.0));

        if ui.add(left_btn).on_hover_text("Toggle Sidebar (⌘B)").clicked() {
            action = Some(HeaderAction::ToggleLeftSidebar);
        }

        ui.add_space(6.0);

        // T3 Code Breadcrumb:
        // [PN] postplan / Add Expiring Draft Share Links
        Frame::none()
            .fill(Color32::from_rgb(0x28, 0x10, 0x12))
            .rounding(Rounding::same(3.0))
            .inner_margin(Margin::symmetric(4.0, 1.0))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("PN").color(theme.tag_red).strong().size(9.5));
            });

        ui.label(egui::RichText::new("postplan").color(theme.text_silver).size(11.5));
        ui.label(egui::RichText::new("/").color(theme.text_silver).size(11.5));
        ui.label(
            egui::RichText::new("Add Expiring Draft Share Links")
                .color(theme.text_paper)
                .strong()
                .size(12.0),
        );

        // Right side: T3 Code Top Window Controls [sidebar right] [split] [window mode]
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right Dock Toggle [◨]
            let right_btn = egui::Button::new(
                egui::RichText::new("◨")
                    .size(13.0)
                    .color(if layout.show_right_dock {
                        theme.accent_pop
                    } else {
                        theme.text_silver
                    }),
            )
            .fill(Color32::TRANSPARENT)
            .rounding(Rounding::same(4.0));

            if ui.add(right_btn).on_hover_text("Toggle BeeBox Dock (⌘E)").clicked() {
                action = Some(HeaderAction::ToggleRightDock);
            }

            // Split View [◫]
            let split_active = layout.center_mode == CenterViewMode::SplitChatKanban;
            let split_btn = egui::Button::new(
                egui::RichText::new("◫")
                    .size(13.0)
                    .color(if split_active { theme.accent_pop } else { theme.text_silver }),
            )
            .fill(Color32::TRANSPARENT)
            .rounding(Rounding::same(4.0));

            if ui.add(split_btn).on_hover_text("Toggle Split Kanban/Chat").clicked() {
                let next_mode = if split_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::SplitChatKanban
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // Kanban View Toggle
            let kanban_active = layout.center_mode == CenterViewMode::KanbanBoard;
            let kanban_btn = egui::Button::new(
                egui::RichText::new("📊")
                    .size(12.0)
                    .color(if kanban_active { theme.accent_pop } else { theme.text_silver }),
            )
            .fill(Color32::TRANSPARENT)
            .rounding(Rounding::same(4.0));

            if ui.add(kanban_btn).on_hover_text("Toggle Kanban Board (⌘K)").clicked() {
                let next_mode = if kanban_active {
                    CenterViewMode::ChatTimeline
                } else {
                    CenterViewMode::KanbanBoard
                };
                action = Some(HeaderAction::SetCenterMode(next_mode));
            }

            // Browser Profile Launcher (Ego Lite / Chrome)
            if ui
                .button(egui::RichText::new("🌐").color(theme.text_silver).size(12.0))
                .on_hover_text("Launch GitLab / Isolated Browser Profile")
                .clicked()
            {
                action = Some(HeaderAction::LaunchGitLabBrowser);
            }
        });
    });

    action
}
