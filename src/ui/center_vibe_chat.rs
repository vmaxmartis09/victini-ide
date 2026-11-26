use crate::theme::Lucide;
use crate::ai::{AgentAvatarState, AvatarManager, ElegansAgent, ExecutionMode};
use crate::theme::CanhTheme;
use egui::{vec2, Color32, Frame, Margin, Rounding, ScrollArea, Stroke, Ui};

pub fn render_vibe_center(
    ui: &mut Ui,
    theme: &CanhTheme,
    active_agent: ElegansAgent,
    avatar_state: AgentAvatarState,
    exec_mode: ExecutionMode,
    chat_logs: &[String],
    input_prompt: &mut String,
    on_send_prompt: impl FnOnce(),
    mut on_toggle_mode: impl FnMut(ExecutionMode),
) {
    ui.vertical(|ui| {
        // 1. Sleek Top Bar with Agent Avatar & Mode Pill
        Frame::none()
            .fill(Color32::from_rgb(0x10, 0x12, 0x18))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1F, 0x23, 0x30)))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    AvatarManager::render_avatar(ui, avatar_state, 36.0);
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{} {}", active_agent.icon(), active_agent.title()))
                                    .color(theme.text_paper)
                                    .strong()
                                    .size(13.0),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (mode_str, mode_color) = match exec_mode {
                                    ExecutionMode::Auto => ("⚡ Auto Vibe", theme.accent_pop),
                                    ExecutionMode::Bypass => (Lucide::PLAY, Color32::from_rgb(0xFF, 0x5C, 0x5C)),
                                    ExecutionMode::Gatekeeper => (Lucide::STOP, theme.text_silver),
                                };

                                if ui.button(egui::RichText::new(mode_str).color(mode_color).strong().size(11.0)).clicked() {
                                    let next = match exec_mode {
                                        ExecutionMode::Auto => ExecutionMode::Bypass,
                                        ExecutionMode::Bypass => ExecutionMode::Gatekeeper,
                                        ExecutionMode::Gatekeeper => ExecutionMode::Auto,
                                    };
                                    on_toggle_mode(next);
                                }
                            });
                        });

                        let state_label = match avatar_state {
                            AgentAvatarState::Idle => "Ready for next instruction",
                            AgentAvatarState::Thinking => "Analyzing context & AST dependencies...",
                            AgentAvatarState::Answering => "Streaming implementation & running S3 guard...",
                            AgentAvatarState::Offline => "Sleeping to reduce RAM (< 35MB)",
                        };
                        ui.label(egui::RichText::new(state_label).color(theme.text_silver).size(11.0));
                    });
                });
            });

        ui.add_space(8.0);

        // 2. Chat Timeline (Qoder/T3 Code Clean Chat Bubbles)
        let available_height = ui.available_height() - 95.0;
        ScrollArea::vertical()
            .id_salt("victini_chat_scroll")
            .max_height(available_height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for log in chat_logs {
                    if log.starts_with("👤") {
                        // User prompt bubble
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            Frame::none()
                                .fill(Color32::from_rgb(0x18, 0x1B, 0x24))
                                .rounding(Rounding::same(10.0))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(0x28, 0x2D, 0x3D)))
                                .inner_margin(Margin::symmetric(14.0, 9.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(log).color(theme.text_paper).size(12.5));
                                });
                        });
                    } else if log.starts_with("🏷️") || log.starts_with("✓") || log.starts_with("✨") {
                        // System / Agent Action status card
                        Frame::none()
                            .fill(Color32::from_rgb(0x0E, 0x11, 0x16))
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1A, 0x20, 0x2C)))
                            .inner_margin(Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                let col = if log.starts_with("✓") || log.starts_with("✨") {
                                    Color32::from_rgb(0x4E, 0xC9, 0xB0)
                                } else {
                                    theme.accent_pop
                                };
                                ui.label(egui::RichText::new(log).color(col).strong().size(12.0));
                            });
                    } else {
                        // Regular stream content
                        Frame::none()
                            .fill(Color32::from_rgb(0x0C, 0x0E, 0x12))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(log).color(theme.text_paper).size(12.0));
                            });
                    }
                    ui.add_space(5.0);
                }
            });

        ui.add_space(8.0);

        // 3. Floating Bottom Action Box (T3 Code inspired prompt box)
        Frame::none()
            .fill(Color32::from_rgb(0x10, 0x12, 0x18))
            .rounding(Rounding::same(12.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x26, 0x34)))
            .inner_margin(Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let resp = ui.add_sized(
                        vec2(ui.available_width() - 86.0, 32.0),
                        egui::TextEdit::singleline(input_prompt)
                            .hint_text("Ask anything. @ files, $ skills, / commands..."),
                    );

                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        on_send_prompt();
                    } else if ui
                        .add_sized(
                            vec2(72.0, 30.0),
                            egui::Button::new(
                                egui::RichText::new("⚡ Vibe")
                                    .color(Color32::BLACK)
                                    .strong()
                                    .size(12.5),
                            ).fill(theme.accent_pop).rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        on_send_prompt();
                    }
                });
            });
    });
}
