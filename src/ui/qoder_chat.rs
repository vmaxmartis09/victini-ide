use crate::ai::{
    AgentAvatarState, ChatSessionManager, ElegansAgent, ExecutionMode,
    MessageRole, QuotedContext,
};
use crate::config::DiscoveredSkill;
use crate::git::GitFileChange;
use crate::theme::CanhTheme;
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
        // T3 Code Top Chat Header Breadcrumb:
        // [PN] postplan / Add Expiring Draft Share Links ... right icons [sidebar/dock toggle]
        ui.horizontal(|ui| {
            // PN tag
            Frame::none()
                .fill(Color32::from_rgb(0x28, 0x10, 0x12))
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

        // Chat Timeline Area
        let scroll_h = (ui.available_height() - 110.0).max(100.0);
        ScrollArea::vertical()
            .id_salt("t3_center_chat_scroll")
            .auto_shrink([false, false])
            .max_height(scroll_h)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let thread = chat_mgr.active_thread();

                // T3 Code Hero User Request Capsule at top right
                for msg in &thread.messages {
                    match msg.role {
                        MessageRole::User => {
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                Frame::none()
                                    .fill(Color32::from_rgb(0x16, 0x16, 0x16)) // T3 dark capsule
                                    .rounding(Rounding::same(14.0))
                                    .inner_margin(Margin::symmetric(16.0, 10.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(&msg.content)
                                                .color(theme.text_paper)
                                                .size(12.5),
                                        );
                                    });
                            });
                            ui.add_space(16.0);
                        }
                        MessageRole::Assistant => {
                            // T3 Code Assistant Layout:
                            // "Worked for 20s >"
                            // Avatar cluster [icon icon icon +1]  4 subagents (4 working)  25s v
                            // Message text: "Okay, opening Claude, Codex, Muse and Gemini."
                            // Progress: "◌ Waiting on 4 subagents  Dashboard share button, Share link API... Stop"
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("Worked for {:.0}s ›", msg.duration_secs.max(20.0)))
                                        .color(theme.text_silver)
                                        .size(11.0),
                                );

                                ui.add_space(8.0);

                                // Subagent header line
                                ui.horizontal(|ui| {
                                    // Cluster badges
                                    ui.label(egui::RichText::new("✴ ⚙ ◬ +1").color(theme.accent_pop).size(12.0));
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("4 subagents").color(theme.text_paper).strong().size(11.5));
                                    ui.label(
                                        egui::RichText::new("4 working")
                                            .color(Color32::from_rgb(0x51, 0xA2, 0xFF))
                                            .size(10.5),
                                    );

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(egui::RichText::new("25s ⌵").color(theme.text_silver).size(10.5));
                                    });
                                });

                                ui.add_space(8.0);

                                // Main assistant text
                                ui.label(
                                    egui::RichText::new(&msg.content)
                                        .color(theme.text_paper)
                                        .size(12.5),
                                );

                                ui.add_space(12.0);

                                // Waiting / progress indicator
                                ui.horizontal_wrapped(|ui| {
                                    let spinner_icon = if is_streaming { "◌" } else { "✓" };
                                    ui.label(egui::RichText::new(spinner_icon).color(theme.text_silver).size(12.0));
                                    ui.label(
                                        egui::RichText::new("Waiting on subagents")
                                            .color(theme.text_silver)
                                            .size(11.0),
                                    );
                                    ui.label(
                                        egui::RichText::new("Dashboard share button, Share link API, Share link tests...")
                                            .color(theme.text_silver)
                                            .size(11.0),
                                    );
                                    if is_streaming {
                                        if ui.small_button("Stop").clicked() {
                                            action = Some(ChatUiAction::StopStream);
                                        }
                                    }
                                });

                                // Action bar
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    if ui.small_button("📋 Copy").clicked() {
                                        ui.ctx().copy_text(msg.content.clone());
                                    }
                                    if ui.small_button("❝ Quote").clicked() {
                                        action = Some(ChatUiAction::SetQuote(QuotedContext {
                                            source_msg_id: msg.id.clone(),
                                            author_label: "T3 Subagents".to_string(),
                                            snippet: msg.content.clone(),
                                            is_quote_block: true,
                                        }));
                                    }
                                    if ui.small_button("📊 +Kanban").clicked() {
                                        action = Some(ChatUiAction::SendMessageToKanban {
                                            title: msg.content.lines().next().unwrap_or("Task").to_string(),
                                            detail: msg.content.clone(),
                                            agent: msg.agent,
                                        });
                                    }
                                });
                            });
                            ui.add_space(18.0);
                        }
                        MessageRole::System => {
                            Frame::none()
                                .fill(Color32::from_rgb(0x11, 0x11, 0x11))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(10.0, 6.0))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(&msg.content).color(theme.text_silver).size(11.0));
                                });
                            ui.add_space(8.0);
                        }
                    }
                }
            });

        ui.add_space(6.0);

        // T3 Code Floating Bottom Prompt Bar (1:1 design):
        // Dark pill container (#121212) with subtle hairline border (#191919)
        // Upper line: "Ask anything. @ files, $ skills, / commands"
        // Lower icons: [paperclip] [empty space] [sparkle Opus 5.5 High] [arrow up inside rounded circle]
        Frame::none()
            .fill(Color32::from_rgb(0x12, 0x12, 0x12))
            .rounding(Rounding::same(16.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
            .inner_margin(Margin::symmetric(16.0, 10.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Active Quote preview if any
                    if let Some(ref q) = chat_mgr.active_quote {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("❝ {}:", q.author_label)).color(theme.accent_pop).size(10.5));
                            let short_snip: String = q.snippet.chars().take(60).collect();
                            ui.label(egui::RichText::new(short_snip).color(theme.text_silver).size(10.5));
                            if ui.small_button("✕").clicked() {
                                action = Some(ChatUiAction::ClearQuote);
                            }
                        });
                        ui.add_space(4.0);
                    }

                    // Text Editor
                    let text_edit = egui::TextEdit::multiline(input_prompt)
                        .desired_rows(1)
                        .desired_width(ui.available_width())
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

                    ui.add_space(6.0);

                    // Lower Action Bar
                    ui.horizontal(|ui| {
                        // Left Paperclip icon
                        if ui.button(egui::RichText::new("📎").color(theme.text_silver).size(13.0)).clicked() {
                            *show_mention_popup = !*show_mention_popup;
                        }

                        // Right: Model Selector Pill [✴ Opus 5.5 High] + Arrow Up Send Button [↑]
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // Rounded Up Arrow Send Button
                            let send_btn = egui::Button::new(
                                egui::RichText::new("↑").color(Color32::BLACK).strong().size(14.0),
                            )
                            .fill(theme.text_paper)
                            .rounding(Rounding::same(12.0));

                            if ui.add_sized(vec2(26.0, 26.0), send_btn).clicked()
                                && !input_prompt.trim().is_empty()
                            {
                                action = Some(ChatUiAction::SendPrompt);
                            }

                            ui.add_space(6.0);

                            // Model Badge: ✴ Opus 5.5 High
                            Frame::none()
                                .fill(Color32::from_rgb(0x19, 0x19, 0x19))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(8.0, 3.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("✴").color(theme.accent_pop).size(11.0));
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
