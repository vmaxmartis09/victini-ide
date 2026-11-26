use crate::ai::{
    AgentAvatarState, ChatSessionManager, ElegansAgent, ExecutionMode,
    MessageRole, QuotedContext,
};
use crate::config::DiscoveredSkill;
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};

pub enum ChatUiAction {
    SendPrompt,
    StopStream,
    SetExecutionMode(ExecutionMode),
    SelectActiveAgent(ElegansAgent),
    SetQuote(QuotedContext),
    ClearQuote,
    SendMessageToKanban { title: String, detail: String, agent: ElegansAgent },
}

#[allow(clippy::too_many_arguments)]
pub fn render_qoder_chat_timeline(
    ui: &mut Ui,
    theme: &CanhTheme,
    _elapsed_secs: f32,
    _active_agent: ElegansAgent,
    avatar_state: AgentAvatarState,
    _exec_mode: ExecutionMode,
    chat_mgr: &ChatSessionManager,
    _file_changes: &[GitFileChange],
    _skills: &[DiscoveredSkill],
    input_prompt: &mut String,
    show_mention_popup: &mut bool,
    _show_skill_popup: &mut bool,
) -> Option<ChatUiAction> {
    let mut action = None;
    let is_streaming = avatar_state == AgentAvatarState::Thinking
        || avatar_state == AgentAvatarState::Answering;

    ui.vertical(|ui| {
        // 1. T3 Code Top Chat Header Breadcrumb
        ui.horizontal(|ui| {
            Frame::none()
                .fill(Color32::from_rgb(0x28, 0x0E, 0x10))
                .rounding(Rounding::same(3.0))
                .inner_margin(Margin::symmetric(4.0, 1.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("PN").color(theme.tag_red).strong().size(9.5));
                });
            ui.label(egui::RichText::new("postplan").color(theme.text_silver).size(11.0));
            ui.label(egui::RichText::new("/").color(theme.text_silver).size(11.0));
            let active_title = &chat_mgr.active_thread().title;
            ui.label(egui::RichText::new(active_title).color(theme.text_paper).strong().size(12.0));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if is_streaming
                    && ui
                        .add(
                            egui::Button::new(egui::RichText::new("Stop").color(theme.text_paper).size(10.5))
                                .fill(Color32::from_rgb(0x30, 0x12, 0x14))
                                .rounding(Rounding::same(4.0)),
                        )
                        .clicked()
                {
                    action = Some(ChatUiAction::StopStream);
                }
            });
        });

        ui.add_space(10.0);

        // 2. Chat Timeline Area
        let scroll_h = (ui.available_height() - 95.0).max(100.0);
        ScrollArea::vertical()
            .id_salt("t3_center_chat_scroll")
            .auto_shrink([false, false])
            .max_height(scroll_h)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let thread = chat_mgr.active_thread();

                for msg in &thread.messages {
                    match msg.role {
                        MessageRole::User => {
                            // User Message in dark rounded capsule on the right
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                Frame::none()
                                    .fill(Color32::from_rgb(0x16, 0x16, 0x16))
                                    .rounding(Rounding::same(14.0))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x22, 0x22)))
                                    .inner_margin(Margin::symmetric(16.0, 9.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(&msg.content)
                                                .color(theme.text_paper)
                                                .size(12.2),
                                        );
                                    });
                            });
                            ui.add_space(14.0);
                        }
                        MessageRole::Assistant => {
                            // T3 Code Assistant Message Layout
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("Worked for {:.0}s >", msg.duration_secs.max(20.0)))
                                        .color(theme.text_silver)
                                        .size(10.5),
                                );

                                ui.add_space(6.0);

                                // Subagent header line
                                ui.horizontal(|ui| {
                                    // Colored dot icons for subagents
                                    let colors = [
                                        theme.accent_pop,
                                        Color32::from_rgb(0x51, 0xA2, 0xFF),
                                        Color32::from_rgb(0x4E, 0xC9, 0xB0),
                                        Color32::from_rgb(0xE5, 0xC0, 0x7B),
                                    ];
                                    for c in colors {
                                        let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                                        ui.painter().circle_filled(r.center(), 2.8, c);
                                    }

                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("4 subagents").color(theme.text_paper).strong().size(11.5));
                                    ui.label(
                                        egui::RichText::new("4 working")
                                            .color(Color32::from_rgb(0x51, 0xA2, 0xFF))
                                            .size(10.5),
                                    );

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(egui::RichText::new("25s v").color(theme.text_silver).size(10.5));
                                    });
                                });

                                ui.add_space(8.0);

                                // Main assistant text
                                ui.label(
                                    egui::RichText::new(&msg.content)
                                        .color(theme.text_paper)
                                        .size(12.5),
                                );

                                ui.add_space(10.0);

                                // Waiting / progress indicator
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(egui::RichText::new("o").color(theme.text_silver).size(10.0));
                                    ui.label(
                                        egui::RichText::new("Waiting on 4 subagents Dashboard share button, Share link API, Share link tests...")
                                            .color(theme.text_silver)
                                            .size(10.5),
                                    );
                                    if is_streaming {
                                        if ui.small_button("Stop").clicked() {
                                            action = Some(ChatUiAction::StopStream);
                                        }
                                    }
                                });

                                // Action buttons (Subtle and clean with Lucide icons)
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    if ui.add(egui::Button::new(egui::RichText::new(format!("{} Copy", Lucide::COPY)).color(theme.text_silver).size(10.0)).fill(Color32::from_rgb(0x16, 0x16, 0x16))).clicked() {
                                        ui.ctx().copy_text(msg.content.clone());
                                    }
                                    if ui.add(egui::Button::new(egui::RichText::new(format!("{} Quote", Lucide::QUOTE)).color(theme.text_silver).size(10.0)).fill(Color32::from_rgb(0x16, 0x16, 0x16))).clicked() {
                                        action = Some(ChatUiAction::SetQuote(QuotedContext {
                                            source_msg_id: msg.id.clone(),
                                            author_label: "T3 Subagents".to_string(),
                                            snippet: msg.content.clone(),
                                            is_quote_block: true,
                                        }));
                                    }
                                    if ui.add(egui::Button::new(egui::RichText::new(format!("{} +Kanban", Lucide::KANBAN)).color(theme.text_silver).size(10.0)).fill(Color32::from_rgb(0x16, 0x16, 0x16))).clicked() {
                                        action = Some(ChatUiAction::SendMessageToKanban {
                                            title: msg.content.lines().next().unwrap_or("Task").to_string(),
                                            detail: msg.content.clone(),
                                            agent: msg.agent,
                                        });
                                    }
                                });
                            });
                            ui.add_space(16.0);
                        }
                        MessageRole::System => {
                            Frame::none()
                                .fill(Color32::from_rgb(0x11, 0x11, 0x11))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(10.0, 5.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(&msg.content).color(theme.text_silver).size(10.5));
                                });
                            ui.add_space(8.0);
                        }
                    }
                }
            });

        ui.add_space(4.0);

        // 3. T3 Code Floating Bottom Prompt Bar (Dark OLED Pill #111111)
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
            .rounding(Rounding::same(14.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1C, 0x1C, 0x1C)))
            .inner_margin(Margin::symmetric(14.0, 8.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    if let Some(ref q) = chat_mgr.active_quote {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Quote ({}):", q.author_label)).color(theme.accent_pop).size(10.0));
                            let short_snip: String = q.snippet.chars().take(60).collect();
                            ui.label(egui::RichText::new(short_snip).color(theme.text_silver).size(10.0));
                            if ui.small_button("x").clicked() {
                                action = Some(ChatUiAction::ClearQuote);
                            }
                        });
                        ui.add_space(3.0);
                    }

                    // Prompt Input - Frame false to prevent white box!
                    let text_edit = egui::TextEdit::multiline(input_prompt)
                        .desired_rows(1)
                        .desired_width(ui.available_width())
                        .frame(false)
                        .text_color(theme.text_paper)
                        .hint_text("Ask anything. @ files, $ skills, / commands");
                    let resp = ui.add(text_edit);

                    let enter_submit = resp.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
                    if enter_submit {
                        while input_prompt.ends_with('\n') || input_prompt.ends_with('\r') {
                            input_prompt.pop();
                        }
                        if !input_prompt.trim().is_empty() {
                            action = Some(ChatUiAction::SendPrompt);
                        }
                    }

                    ui.add_space(4.0);

                    // Lower Action Row
                    ui.horizontal(|ui| {
                        // Paperclip button
                        if ui.add(egui::Button::new(egui::RichText::new("+").color(theme.text_silver).size(14.0)).fill(Color32::TRANSPARENT)).clicked() {
                            *show_mention_popup = !*show_mention_popup;
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // Rounded Up Arrow Send Button (Crisp White Circle with Lucide::ARROW_UP)
                            let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::click());
                            ui.painter().circle_filled(btn_rect.center(), 12.0, theme.text_paper);
                            ui.painter().text(
                                btn_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                Lucide::ARROW_UP,
                                egui::FontId::proportional(13.0),
                                Color32::BLACK,
                            );

                            if btn_resp.clicked() && !input_prompt.trim().is_empty() {
                                action = Some(ChatUiAction::SendPrompt);
                            }

                            ui.add_space(8.0);

                            // Model Badge: Opus 5.5 High
                            Frame::none()
                                .fill(Color32::from_rgb(0x19, 0x19, 0x19))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(7.0, 3.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(Lucide::SPARKLES)
                                                .color(theme.accent_pop)
                                                .size(11.0),
                                        );
                                        ui.label(
                                            egui::RichText::new("Opus 5.5 High")
                                                .color(theme.text_paper)
                                                .size(11.0),
                                        );
                                    });
                                });
                        });
                    });
                });
            });
    });

    action
}
