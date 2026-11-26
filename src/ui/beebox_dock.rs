use crate::ai::{AgentTaskCard, KanbanColumn};
use crate::emulator::{EmulatorState, TouchRipple, ELEGANS_VIEWPORT_PRESETS};
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use egui::{
    pos2, vec2, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rounding, ScrollArea,
    Stroke, Ui,
};

pub enum BeeBoxAction {
    OpenProjectsModal,
    SetupWorktree,
    OpenGitLabMr,
    TriggerTap(u32, u32),
    SwitchPreset(usize),
    CaptureAdbFrame,
    RunBackgroundTask(String),
}

#[allow(clippy::too_many_arguments)]
pub fn render_beebox_panel(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    _project_name: &str,
    branch_name: &str,
    _commit_oid: &str,
    _file_changes: &[GitFileChange],
    kanban_tasks: &[AgentTaskCard],
    emu_state: &mut EmulatorState,
) -> Option<BeeBoxAction> {
    let mut action = None;

    // T3 CODE EXACT BEEBOX CONTAINER (#111111)
    Frame::none()
        .fill(Color32::from_rgb(0x11, 0x11, 0x11))
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(0x19, 0x19, 0x19)))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("t3_beebox_panel_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // 1. Header: BeeBox  ... [folder icon]
                    ui.horizontal(|ui| {
                        // Monitor icon glyph drawn cleanly
                        let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
                        ui.painter().rect_stroke(rect, Rounding::same(2.0), Stroke::new(1.2, theme.text_silver));

                        ui.add_space(3.0);
                        ui.label(
                            egui::RichText::new("BeeBox")
                                .color(theme.text_paper)
                                .strong()
                                .size(12.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(egui::Button::new(egui::RichText::new(Lucide::FOLDER).color(theme.text_silver).size(12.0)).fill(Color32::TRANSPARENT))
                                .on_hover_text("Open Projects")
                                .clicked()
                            {
                                action = Some(BeeBoxAction::OpenProjectsModal);
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 2. Action Accordions
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(Lucide::CODE).color(theme.text_silver).size(11.0));
                        ui.label(egui::RichText::new("Open in editor").color(theme.text_silver).size(11.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(egui::RichText::new(Lucide::CHEVRON_RIGHT).color(theme.text_silver).size(10.0));
                        });
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(Lucide::GIT_BRANCH).color(theme.text_silver).size(11.0));
                        if ui
                            .link(egui::RichText::new("Setup Worktree").color(theme.text_silver).size(11.0))
                            .clicked()
                        {
                            action = Some(BeeBoxAction::SetupWorktree);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(egui::RichText::new(Lucide::CHEVRON_RIGHT).color(theme.text_silver).size(10.0));
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 3. Git Status section:
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(Lucide::GIT_BRANCH).color(theme.accent_pop).size(11.0));
                        ui.label(
                            egui::RichText::new(branch_name)
                                .color(theme.text_paper)
                                .strong()
                                .size(11.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(egui::RichText::new(Lucide::CHEVRON_DOWN).color(theme.text_silver).size(10.0));
                        });
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(Lucide::GIT_PR).color(theme.text_silver).size(11.0));
                        if ui
                            .link(egui::RichText::new("Commit / GitLab MR").color(theme.text_silver).size(11.0))
                            .clicked()
                        {
                            action = Some(BeeBoxAction::OpenGitLabMr);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(egui::RichText::new(Lucide::CHEVRON_RIGHT).color(theme.text_silver).size(10.0));
                        });
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Changes").color(theme.text_silver).size(11.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(egui::RichText::new("+0 -0").color(theme.text_silver).size(10.5));
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 4. Lineage - 4 running ... [...]
                    let active_tasks: Vec<&AgentTaskCard> = kanban_tasks
                        .iter()
                        .filter(|t| t.column == KanbanColumn::InProgress || t.is_running_bg)
                        .collect();

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Lineage · {} running", active_tasks.len().max(4)))
                                .color(theme.text_silver)
                                .size(10.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(egui::RichText::new("...").color(theme.text_silver).size(11.0));
                        });
                    });

                    ui.add_space(5.0);

                    // Clean subagents list with colored dots instead of broken tofu icons!
                    let lineage_items = [
                        ("Share link docs", "19s", Color32::from_rgb(0x51, 0xA2, 0xFF)),
                        ("Share link tests", "21s", Color32::from_rgb(0x4E, 0xC9, 0xB0)),
                        ("Share link API", "23s", Color32::from_rgb(0xE5, 0xC0, 0x7B)),
                        ("Dashboard share button", "25s", theme.accent_pop),
                    ];

                    for (name, time_s, dot_col) in lineage_items {
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(r.center(), 2.5, dot_col);

                            ui.label(egui::RichText::new(name).color(theme.text_paper).size(11.0));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(egui::RichText::new(time_s).color(theme.text_silver).size(10.0));
                            });
                        });
                        ui.add_space(2.0);
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 5. Live Touch Viewport Canvas
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Live Touch Viewport")
                                .color(theme.text_silver)
                                .size(10.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(egui::Button::new(egui::RichText::new("Snap").color(theme.text_paper).size(10.0)).fill(Color32::from_rgb(0x19, 0x19, 0x19)))
                                .clicked()
                            {
                                action = Some(BeeBoxAction::CaptureAdbFrame);
                            }
                        });
                    });

                    ui.add_space(4.0);

                    let current_preset = ELEGANS_VIEWPORT_PRESETS[emu_state.active_preset_idx];
                    let emu_w = (ui.available_width() - 8.0).clamp(160.0, 240.0);
                    let emu_h = emu_w * 1.5;

                    Frame::none()
                        .fill(Color32::from_rgb(0x06, 0x06, 0x06))
                        .rounding(Rounding::same(14.0))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
                        .inner_margin(Margin::same(6.0))
                        .show(ui, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(vec2(emu_w, emu_h), egui::Sense::click_and_drag());
                            if resp.clicked() {
                                if let Some(pos) = resp.interact_pointer_pos() {
                                    let rel_x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                                    let rel_y = ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                                    let tap_x = (rel_x * current_preset.width as f32) as u32;
                                    let tap_y = (rel_y * current_preset.height as f32) as u32;
                                    emu_state.last_touch = Some(TouchRipple {
                                        norm_pos: Pos2::new(rel_x, rel_y),
                                        device_x: tap_x,
                                        device_y: tap_y,
                                        created_at: elapsed_secs,
                                    });
                                    action = Some(BeeBoxAction::TriggerTap(tap_x, tap_y));
                                }
                            }

                            let painter = ui.painter_at(rect);
                            painter.text(
                                rect.center(),
                                Align2::CENTER_CENTER,
                                format!("{}\n{}x{}\nLive Tap Ready", current_preset.name.split('(').next().unwrap_or(current_preset.name), current_preset.width, current_preset.height),
                                FontId::proportional(10.5),
                                theme.text_silver,
                            );

                            if let Some(ref touch) = emu_state.last_touch {
                                let tap_pt = pos2(
                                    rect.min.x + touch.norm_pos.x * rect.width(),
                                    rect.min.y + touch.norm_pos.y * rect.height(),
                                );
                                painter.circle_filled(tap_pt, 4.0, theme.accent_pop);
                            }
                        });
                });
        });

    action
}
