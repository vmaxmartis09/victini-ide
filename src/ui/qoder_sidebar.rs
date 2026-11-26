use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::{GitFileChange, WorktreeInfo};
use crate::theme::{CanhTheme, Lucide};
use crate::ui::layout_state::CenterViewMode;
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};

pub enum SidebarAction {
    ToggleMiniMode,
    NewChatThread,
    SelectChatThread(usize),
    SwitchCenterView(CenterViewMode),
    SelectWorktree(String),
    CreateNewBranch(String),
    StageFile(String),
    UnstageFile(String),
    DiscardFile(String),
    StageAll,
    ViewFileDiff(String),
    CommitWithMessage(String),
    AiGenerateCommitMsg,
    GitPush,
    GitPull,
    CreateGitLabMr,
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
    search_query: &mut String,
    _worktrees: &[WorktreeInfo],
    _branches: &[String],
    _new_branch_input: &mut String,
    _file_changes: &[GitFileChange],
    _commit_message: &mut String,
    _skills: &[DiscoveredSkill],
    _agents: &[DiscoveredAgent],
    thread_titles: &[(usize, String, String)],
    active_thread_idx: usize,
) -> Option<SidebarAction> {
    let mut action = None;

    // MINI RAIL MODE
    if is_mini {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            if ui
                .button(egui::RichText::new("T3").color(theme.text_paper).strong().size(13.0))
                .on_hover_text("Mở rộng Sidebar")
                .clicked()
            {
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(8.0);
            if ui
                .button(egui::RichText::new("+").color(theme.accent_pop).strong().size(16.0))
                .on_hover_text("New Task")
                .clicked()
            {
                action = Some(SidebarAction::NewChatThread);
            }
            ui.add_space(6.0);
            if ui
                .button(egui::RichText::new("Kanban").size(11.0))
                .on_hover_text("Kanban Board")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::KanbanBoard));
            }
        });
        return action;
    }

    // T3 CODE EXACT 1:1 SIDEBAR (#000000 Pure Black)
    ui.vertical(|ui| {
        // 1. T3 Code Brand Row
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            // Distinctive T3 Code Logo Mark (rounded square with dot)
            let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
            ui.painter().rect_stroke(rect, Rounding::same(3.0), Stroke::new(1.2, Color32::from_rgb(0x80, 0x80, 0x80)));
            ui.painter().circle_filled(rect.center(), 2.5, theme.accent_pop);

            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("T3 Code")
                    .color(theme.text_paper)
                    .strong()
                    .size(13.0),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(egui::Button::new(egui::RichText::new("«").color(theme.text_silver).size(12.0)).fill(Color32::TRANSPARENT))
                    .on_hover_text("Thu gọn Sidebar")
                    .clicked()
                {
                    action = Some(SidebarAction::ToggleMiniMode);
                }
            });
        });

        ui.add_space(10.0);

        // 2. Search Box with Clean Action Buttons
        ui.horizontal(|ui| {
            let search_w = (ui.available_width() - 52.0).max(60.0);
            Frame::none()
                .fill(Color32::from_rgb(0x0C, 0x0C, 0x0C))
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::symmetric(6.0, 3.0))
                .show(ui, |ui| {
                    ui.add_sized(
                        vec2(search_w, 20.0),
                        egui::TextEdit::singleline(search_query)
                            .hint_text("Search")
                            .frame(false)
                            .text_color(theme.text_paper),
                    );
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // New Task Button (Lucide::PLUS)
                if ui
                    .add(egui::Button::new(egui::RichText::new(Lucide::PLUS).color(theme.text_paper).strong().size(13.0)).fill(Color32::TRANSPARENT))
                    .on_hover_text("New Task")
                    .clicked()
                {
                    action = Some(SidebarAction::NewChatThread);
                }
                // Projects / Workspaces (Lucide::FOLDER)
                if ui
                    .add(egui::Button::new(egui::RichText::new(Lucide::FOLDER).color(theme.text_silver).size(12.0)).fill(Color32::TRANSPARENT))
                    .on_hover_text("Projects & Workspaces")
                    .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(CenterViewMode::ProjectsModal));
                }
            });
        });

        ui.add_space(10.0);

        // 3. Thread List (T3 Code Exact Layout)
        ScrollArea::vertical()
            .id_salt("t3_sidebar_threads_scroll")
            .auto_shrink([false, false])
            .max_height(ui.available_height() - 110.0)
            .show(ui, |ui| {
                for (idx, title, time_ago) in thread_titles {
                    let is_active = *idx == active_thread_idx;
                    let bg = if is_active {
                        Color32::from_rgb(0x11, 0x11, 0x11)
                    } else {
                        Color32::TRANSPARENT
                    };

                    let resp = Frame::none()
                        .fill(bg)
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                // Line 1: Red Tag [PN] postplan ... Time
                                ui.horizontal(|ui| {
                                    Frame::none()
                                        .fill(Color32::from_rgb(0x28, 0x0E, 0x10))
                                        .rounding(Rounding::same(3.0))
                                        .inner_margin(Margin::symmetric(4.0, 1.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new("PN")
                                                    .color(theme.tag_red)
                                                    .strong()
                                                    .size(9.5),
                                            );
                                        });
                                    ui.label(
                                        egui::RichText::new("postplan")
                                            .color(theme.text_silver)
                                            .size(10.5),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            egui::RichText::new(time_ago)
                                                .color(theme.text_silver)
                                                .size(10.0),
                                        );
                                    });
                                });

                                ui.add_space(3.0);

                                // Line 2: Title
                                ui.label(
                                    egui::RichText::new(title)
                                        .color(theme.text_paper)
                                        .strong()
                                        .size(11.8),
                                );

                                ui.add_space(2.0);

                                // Line 3: Branch / Subpath + Sparkle Dot
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("t3/site-favorites-cli-download")
                                            .color(theme.text_silver)
                                            .size(10.5),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        // Orange sparkle dot
                                        let (dot_rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                                        ui.painter().circle_filled(dot_rect.center(), 2.5, theme.accent_pop);
                                    });
                                });
                            });
                        });

                    if resp.response.interact(egui::Sense::click()).clicked() {
                        action = Some(SidebarAction::SelectChatThread(*idx));
                    }
                    ui.add_space(4.0);
                }

                // Settled item in T3 Code
                Frame::none()
                    .fill(Color32::TRANSPARENT)
                    .inner_margin(Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("No project").color(theme.text_silver).size(10.5));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new("Done")
                                        .color(Color32::from_rgb(0x4E, 0xC9, 0xB0))
                                        .strong()
                                        .size(10.5),
                                );
                            });
                        });
                        ui.label(
                            egui::RichText::new("Disk and RAM check")
                                .color(theme.text_paper)
                                .size(11.5),
                        );
                    });
            });

        // 4. Bottom Accordion: Working 1 > / Settled 43 >
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.add_space(6.0);

            // Bottom utility icons: [Settings] [Kanban]
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(egui::RichText::new(Lucide::SETTINGS).color(theme.text_silver).size(13.0)).fill(Color32::TRANSPARENT))
                    .on_hover_text("Settings & Config")
                    .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(CenterViewMode::SettingsModal));
                }
                if ui
                    .add(egui::Button::new(egui::RichText::new(Lucide::MIC).color(theme.text_silver).size(13.0)).fill(Color32::TRANSPARENT))
                    .on_hover_text("Voice Transcriber")
                    .clicked()
                {
                    // Voice prompt shortcut
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new(egui::RichText::new(Lucide::KANBAN).color(theme.text_silver).size(13.0)).fill(Color32::TRANSPARENT))
                        .on_hover_text("Swarm Kanban Board")
                        .clicked()
                    {
                        action = Some(SidebarAction::SwitchCenterView(CenterViewMode::KanbanBoard));
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Settled row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Settled").color(theme.text_silver).size(11.0));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(egui::RichText::new("43 >").color(theme.text_silver).size(11.0));
                });
            });

            ui.add_space(4.0);

            // Working Row (Active session capsule)
            Frame::none()
                .fill(Color32::from_rgb(0x11, 0x11, 0x11))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(8.0, 6.0))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Working")
                                    .color(theme.text_paper)
                                    .strong()
                                    .size(11.0),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new("Waiting 18s")
                                        .color(theme.text_silver)
                                        .size(10.0),
                                );
                            });
                        });
                        ui.label(
                            egui::RichText::new("Add Expiring Draft Share Links")
                                .color(theme.text_paper)
                                .strong()
                                .size(11.5),
                        );
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("main").color(theme.text_silver).size(10.5));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let (dot_rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                                ui.painter().circle_filled(dot_rect.center(), 2.5, theme.accent_pop);
                            });
                        });
                    });
                });
        });
    });

    action
}
