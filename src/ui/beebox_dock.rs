use crate::ai::{AgentTaskCard, KanbanColumn};
use crate::emulator::{EmulatorState, TouchRipple, ELEGANS_VIEWPORT_PRESETS};
use crate::git::GitFileChange;
use crate::theme::CanhTheme;
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
    project_name: &str,
    branch_name: &str,
    commit_oid: &str,
    file_changes: &[GitFileChange],
    kanban_tasks: &[AgentTaskCard],
    emu_state: &mut EmulatorState,
) -> Option<BeeBoxAction> {
    let mut action = None;

    let total_added: usize = file_changes.iter().map(|f| f.added_lines).sum();
    let total_removed: usize = file_changes.iter().map(|f| f.removed_lines).sum();

    ScrollArea::vertical()
        .id_salt("victini_beebox_right_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. BeeBox Header (T3 Code 1:1 style)
            Frame::none()
                .fill(Color32::from_rgb(0x10, 0x13, 0x1A))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x24, 0x32)))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("🖥 BeeBox · {}", project_name))
                                .color(theme.text_paper)
                                .strong()
                                .size(12.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button("📁 Switch")
                                .on_hover_text("Đổi Project Workspace")
                                .clicked()
                            {
                                action = Some(BeeBoxAction::OpenProjectsModal);
                            }
                        });
                    });

                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("🔧 +Worktree")
                                    .size(10.5)
                                    .color(theme.text_paper),
                            )
                            .on_hover_text("Tạo Git Worktree mới độc lập cho nhánh feature")
                            .clicked()
                        {
                            action = Some(BeeBoxAction::SetupWorktree);
                        }
                        if ui
                            .button(
                                egui::RichText::new("🦊 GitLab MR")
                                    .size(10.5)
                                    .color(theme.accent_pop),
                            )
                            .on_hover_text("Push & mở trang Merge Request trên trình duyệt")
                            .clicked()
                        {
                            action = Some(BeeBoxAction::OpenGitLabMr);
                        }
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(" {} ({})", branch_name, commit_oid))
                                .color(theme.accent_pop)
                                .size(11.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("-{}", total_removed))
                                    .color(Color32::from_rgb(0xE0, 0x6C, 0x75))
                                    .strong()
                                    .size(10.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("+{}", total_added))
                                    .color(Color32::from_rgb(0x98, 0xC3, 0x79))
                                    .strong()
                                    .size(10.5),
                            );
                        });
                    });
                });

            ui.add_space(10.0);

            // 2. Remote Emulator Dock (Live Screencast & Touch Ripple)
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("📱 REMOTE EMULATOR")
                        .color(theme.text_silver)
                        .strong()
                        .size(10.5),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .small_button("📸 Capture")
                        .on_hover_text("Chụp frame trực tiếp từ ADB / Simulator")
                        .clicked()
                    {
                        action = Some(BeeBoxAction::CaptureAdbFrame);
                    }
                    ui.label(
                        egui::RichText::new(if emu_state.adb_connected {
                            "● ADB Live"
                        } else {
                            "● Sim Touch"
                        })
                        .color(theme.accent_pop)
                        .size(10.0),
                    );
                });
            });

            ui.add_space(4.0);
            // 5 Viewport Preset Chips
            ui.horizontal_wrapped(|ui| {
                for (idx, preset) in ELEGANS_VIEWPORT_PRESETS.iter().enumerate() {
                    let is_sel = emu_state.active_preset_idx == idx;
                    let short_label = preset.name.split('(').next().unwrap_or(preset.name).trim();
                    let btn = egui::Button::new(
                        egui::RichText::new(short_label)
                            .size(9.5)
                            .color(if is_sel { Color32::BLACK } else { theme.text_paper }),
                    )
                    .fill(if is_sel {
                        theme.accent_pop
                    } else {
                        Color32::from_rgb(0x16, 0x18, 0x22)
                    })
                    .rounding(Rounding::same(4.0));

                    if ui.add(btn).clicked() {
                        emu_state.active_preset_idx = idx;
                        action = Some(BeeBoxAction::SwitchPreset(idx));
                    }
                }
            });

            ui.add_space(6.0);

            // Interactive Smartphone / Viewport Screen Frame
            let current_preset = ELEGANS_VIEWPORT_PRESETS[emu_state.active_preset_idx];
            let aspect = (current_preset.width as f32 / current_preset.height as f32).clamp(0.42, 1.6);
            let emu_w = (ui.available_width() - 18.0).clamp(160.0, 265.0);
            let emu_h = (emu_w / aspect).clamp(170.0, 340.0);

            Frame::none()
                .fill(Color32::from_rgb(0x06, 0x07, 0x0B))
                .rounding(Rounding::same(22.0))
                .stroke(Stroke::new(1.8, Color32::from_rgb(0x28, 0x30, 0x42)))
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(emu_w, emu_h), egui::Sense::click_and_drag());

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
                            emu_state.status_text =
                                format!("Tap forwarded → ({}, {})", tap_x, tap_y);
                            action = Some(BeeBoxAction::TriggerTap(tap_x, tap_y));
                        }
                    }

                    let painter = ui.painter_at(rect);

                    // Top speaker notch inside phone
                    let notch_rect = egui::Rect::from_center_size(
                        pos2(rect.center().x, rect.min.y + 8.0),
                        vec2(54.0, 6.0),
                    );
                    painter.rect_filled(
                        notch_rect,
                        Rounding::same(3.0),
                        Color32::from_rgb(0x16, 0x1A, 0x24),
                    );

                    // Simulated App Preview Header & Interactive Cards inside Emulator
                    let header_card = egui::Rect::from_min_max(
                        pos2(rect.min.x + 8.0, rect.min.y + 22.0),
                        pos2(rect.max.x - 8.0, rect.min.y + 58.0),
                    );
                    painter.rect_filled(
                        header_card,
                        Rounding::same(8.0),
                        Color32::from_rgb(0x11, 0x15, 0x20),
                    );
                    painter.text(
                        pos2(header_card.min.x + 8.0, header_card.center().y),
                        Align2::LEFT_CENTER,
                        format!("⚡ {} · {}×{}", short_preset_title(current_preset.name), current_preset.width, current_preset.height),
                        FontId::proportional(10.5),
                        theme.accent_pop,
                    );

                    // Simulated Content Preview Area
                    let body_card = egui::Rect::from_min_max(
                        pos2(rect.min.x + 8.0, rect.min.y + 66.0),
                        pos2(rect.max.x - 8.0, rect.max.y - 26.0),
                    );
                    painter.rect_filled(
                        body_card,
                        Rounding::same(10.0),
                        Color32::from_rgb(0x0D, 0x10, 0x17),
                    );
                    painter.text(
                        body_card.center(),
                        Align2::CENTER_CENTER,
                        format!(
                            "Victini Live Viewport\nBranch: {}\n{}\n\n(Click để gửi Touch ADB/CDP)",
                            branch_name, emu_state.status_text
                        ),
                        FontId::proportional(10.5),
                        theme.text_silver,
                    );

                    // Animated Neon Touch Ripple when user clicks on Emulator canvas
                    if let Some(ref touch) = emu_state.last_touch {
                        let age = (elapsed_secs - touch.created_at).max(0.0);
                        let tap_pt = pos2(
                            rect.min.x + touch.norm_pos.x * rect.width(),
                            rect.min.y + touch.norm_pos.y * rect.height(),
                        );
                        let radius = 6.0 + (age * 24.0).min(22.0);
                        let alpha = (220.0 - (age * 160.0)).clamp(50.0, 220.0) as u8;
                        let ripple_col = Color32::from_rgba_unmultiplied(
                            theme.accent_pop.r(),
                            theme.accent_pop.g(),
                            theme.accent_pop.b(),
                            alpha,
                        );
                        painter.circle_stroke(tap_pt, radius, Stroke::new(1.8, ripple_col));
                        painter.circle_filled(tap_pt, 3.5, theme.accent_pop);
                        if age < 1.2 {
                            ui.ctx().request_repaint();
                        }
                    }
                });

            ui.add_space(10.0);

            // 3. Subagent Lineage Status (T3 Code 1:1 style: Connected to real Kanban Tasks!)
            let active_tasks: Vec<&AgentTaskCard> = kanban_tasks
                .iter()
                .filter(|t| t.column == KanbanColumn::InProgress || t.is_running_bg)
                .collect();

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Lineage · {} active", active_tasks.len()))
                        .color(theme.text_silver)
                        .strong()
                        .size(10.5),
                );
            });
            ui.add_space(4.0);

            for task in kanban_tasks.iter().take(5) {
                let is_run = task.is_running_bg || task.column == KanbanColumn::InProgress;
                Frame::none()
                    .fill(if is_run {
                        Color32::from_rgb(0x13, 0x17, 0x22)
                    } else {
                        Color32::TRANSPARENT
                    })
                    .rounding(Rounding::same(5.0))
                    .inner_margin(Margin::symmetric(6.0, 3.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(task.assigned_agent.icon())
                                    .color(if is_run {
                                        theme.accent_pop
                                    } else {
                                        theme.text_silver
                                    })
                                    .size(11.0),
                            );
                            let short_t: String = task.title.chars().take(20).collect();
                            ui.label(
                                egui::RichText::new(short_t)
                                    .color(theme.text_paper)
                                    .size(10.5),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if !task.is_running_bg
                                    && ui
                                        .small_button("▶")
                                        .on_hover_text("Chạy ngầm task này")
                                        .clicked()
                                {
                                    action =
                                        Some(BeeBoxAction::RunBackgroundTask(task.id.clone()));
                                }
                                ui.label(
                                    egui::RichText::new(format!("{}s", task.time_spent_secs))
                                        .color(theme.text_silver)
                                        .size(9.5),
                                );
                            });
                        });
                    });
            }
        });

    action
}

fn short_preset_title(full: &str) -> &str {
    full.split('(').next().unwrap_or(full).trim()
}
