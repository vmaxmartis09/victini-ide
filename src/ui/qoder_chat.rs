use crate::ai::{
    AgentAvatarState, AvatarManager, ChatMessage, ChatSessionManager, ElegansAgent, ExecutionMode,
    MessageRole, QuotedContext, ALL_AGENTS,
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
    elapsed_secs: f32,
    active_agent: ElegansAgent,
    avatar_state: AgentAvatarState,
    exec_mode: ExecutionMode,
    chat_mgr: &ChatSessionManager,
    file_changes: &[GitFileChange],
    skills: &[DiscoveredSkill],
    input_prompt: &mut String,
    show_mention_popup: &mut bool,
    show_skill_popup: &mut bool,
) -> Option<ChatUiAction> {
    let mut action = None;

    let total_added: usize = file_changes.iter().map(|f| f.added_lines).sum();
    let total_removed: usize = file_changes.iter().map(|f| f.removed_lines).sum();
    let is_streaming = avatar_state == AgentAvatarState::Thinking
        || avatar_state == AgentAvatarState::Answering;

    ui.vertical(|ui| {
        // 1. Top Agent Command Bar & Live Process Capsule (Qoder Studio 1:1)
        Frame::none()
            .fill(Color32::from_rgb(0x10, 0x12, 0x18))
            .rounding(Rounding::same(10.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x22, 0x2E)))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    AvatarManager::render_animated_avatar(
                        ui,
                        active_agent,
                        avatar_state,
                        38.0,
                        elapsed_secs,
                        theme.accent_pop,
                    );
                    ui.add_space(6.0);

                    ui.vertical(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} {}",
                                    active_agent.icon(),
                                    active_agent.title()
                                ))
                                .color(theme.text_paper)
                                .strong()
                                .size(12.5),
                            );

                            // Quick Agent Switcher Pills
                            for &ag in ALL_AGENTS {
                                let is_cur = ag == active_agent;
                                let btn = egui::Button::new(
                                    egui::RichText::new(ag.icon())
                                        .size(10.5)
                                        .color(if is_cur { Color32::BLACK } else { theme.text_silver }),
                                )
                                .fill(if is_cur {
                                    theme.accent_pop
                                } else {
                                    Color32::from_rgb(0x17, 0x1A, 0x24)
                                })
                                .rounding(Rounding::same(4.0));

                                if ui
                                    .add(btn)
                                    .on_hover_text(format!("Chuyển sang {} ({})", ag.title(), ag.name()))
                                    .clicked()
                                {
                                    action = Some(ChatUiAction::SelectActiveAgent(ag));
                                }
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if is_streaming
                                    && ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("■ Stop")
                                                    .color(Color32::WHITE)
                                                    .strong()
                                                    .size(10.5),
                                            )
                                            .fill(Color32::from_rgb(0xE0, 0x52, 0x52))
                                            .rounding(Rounding::same(5.0)),
                                        )
                                        .clicked()
                                {
                                    action = Some(ChatUiAction::StopStream);
                                }

                                let (mode_str, mode_color) = match exec_mode {
                                    ExecutionMode::Auto => ("⚡ Auto Mode", theme.accent_pop),
                                    ExecutionMode::Bypass => {
                                        ("🚀 Bypass", Color32::from_rgb(0xFF, 0x6B, 0x6B))
                                    }
                                    ExecutionMode::Gatekeeper => {
                                        ("🛡️ Gatekeeper", Color32::from_rgb(0x61, 0xAF, 0xEF))
                                    }
                                };

                                if ui
                                    .button(
                                        egui::RichText::new(mode_str)
                                            .color(mode_color)
                                            .strong()
                                            .size(11.0),
                                    )
                                    .on_hover_text("Click để chuyển đổi Auto / Bypass / Gatekeeper")
                                    .clicked()
                                {
                                    let next = match exec_mode {
                                        ExecutionMode::Auto => ExecutionMode::Bypass,
                                        ExecutionMode::Bypass => ExecutionMode::Gatekeeper,
                                        ExecutionMode::Gatekeeper => ExecutionMode::Auto,
                                    };
                                    action = Some(ChatUiAction::SetExecutionMode(next));
                                }
                            });
                        });

                        // Process Capsule: → Status · Files changed +X -Y
                        ui.horizontal_wrapped(|ui| {
                            let status_pill = if is_streaming {
                                format!("⟳ {} (AGY Live Stream...)", avatar_state.label())
                            } else {
                                format!("→ {} · Tier {}", avatar_state.label(), active_agent.ponytail_tier())
                            };
                            ui.label(
                                egui::RichText::new(status_pill)
                                    .color(if is_streaming {
                                        theme.accent_pop
                                    } else {
                                        theme.text_silver
                                    })
                                    .size(10.5),
                            );
                            ui.label(egui::RichText::new("•").color(theme.text_silver).size(10.0));
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} files changed",
                                    file_changes.len()
                                ))
                                .color(theme.text_paper)
                                .size(10.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("+{}", total_added))
                                    .color(Color32::from_rgb(0x98, 0xC3, 0x79))
                                    .strong()
                                    .size(10.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("-{}", total_removed))
                                    .color(Color32::from_rgb(0xE0, 0x6C, 0x75))
                                    .strong()
                                    .size(10.5),
                            );
                        });
                    });
                });
            });

        ui.add_space(6.0);

        // 2. Responsive Chat Timeline (calculates exact height remaining above composer)
        let composer_reserved = if chat_mgr.active_quote.is_some() {
            165.0
        } else if *show_mention_popup || *show_skill_popup {
            175.0
        } else {
            122.0
        };
        let scroll_h = (ui.available_height() - composer_reserved).max(120.0);

        ScrollArea::vertical()
            .id_salt("victini_qoder_structured_chat_scroll")
            .max_height(scroll_h)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let thread = chat_mgr.active_thread();
                for msg in &thread.messages {
                    if let Some(act) = render_single_chat_message(ui, theme, elapsed_secs, msg) {
                        action = Some(act);
                    }
                    ui.add_space(8.0);
                }
            });

        ui.add_space(6.0);

        // 3. Optional Quick Autocomplete Bar for @Files or $Skills
        if *show_mention_popup {
            Frame::none()
                .fill(Color32::from_rgb(0x16, 0x19, 0x23))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.accent_pop))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("📎 Chọn file đính kèm (@):")
                                .color(theme.accent_pop)
                                .strong()
                                .size(10.5),
                        );
                        if file_changes.is_empty() {
                            for sample in ["src/main.rs", "Cargo.toml", "src/ui/qoder_chat.rs"] {
                                if ui.small_button(format!("@{}", sample)).clicked() {
                                    input_prompt.push_str(&format!(" @{} ", sample));
                                    *show_mention_popup = false;
                                }
                            }
                        } else {
                            for fc in file_changes.iter().take(6) {
                                if ui.small_button(format!("@{}", fc.path)).clicked() {
                                    input_prompt.push_str(&format!(" @{} ", fc.path));
                                    *show_mention_popup = false;
                                }
                            }
                        }
                        if ui.small_button("✕").clicked() {
                            *show_mention_popup = false;
                        }
                    });
                });
            ui.add_space(4.0);
        }

        if *show_skill_popup {
            Frame::none()
                .fill(Color32::from_rgb(0x16, 0x19, 0x23))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.accent_pop))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("🔧 Chọn Skill kích hoạt ($):")
                                .color(theme.accent_pop)
                                .strong()
                                .size(10.5),
                        );
                        for sk in skills.iter().take(6) {
                            if ui.small_button(format!("${}", sk.name)).clicked() {
                                input_prompt.push_str(&format!(" ${} ", sk.name));
                                *show_skill_popup = false;
                            }
                        }
                        for slash in ["/review", "/commit", "/kanban", "/test"] {
                            if ui.small_button(slash).clicked() {
                                input_prompt.push_str(&format!("{} ", slash));
                                *show_skill_popup = false;
                            }
                        }
                        if ui.small_button("✕").clicked() {
                            *show_skill_popup = false;
                        }
                    });
                });
            ui.add_space(4.0);
        }

        // 4. Floating Multi-Line Prompt Input Box with Active Reply / Quote Banner
        Frame::none()
            .fill(Color32::from_rgb(0x12, 0x14, 0x1B))
            .rounding(Rounding::same(12.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x24, 0x2A, 0x3A)))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Active Quote / Reply Banner inside composer
                    if let Some(ref q) = chat_mgr.active_quote {
                        Frame::none()
                            .fill(Color32::from_rgb(0x18, 0x1C, 0x28))
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.accent_pop))
                            .inner_margin(Margin::symmetric(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let badge = if q.is_quote_block {
                                        format!("❝ Trích dẫn từ {} ({}):", q.author_label, q.source_msg_id)
                                    } else {
                                        format!("↩ Trả lời {} ({}):", q.author_label, q.source_msg_id)
                                    };
                                    ui.label(
                                        egui::RichText::new(badge)
                                            .color(theme.accent_pop)
                                            .strong()
                                            .size(10.5),
                                    );
                                    let short_snip: String = q.snippet.chars().take(65).collect();
                                    ui.label(
                                        egui::RichText::new(short_snip.replace('\n', " "))
                                            .color(theme.text_silver)
                                            .italics()
                                            .size(10.5),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui
                                            .small_button("✕")
                                            .on_hover_text("Hủy trích dẫn / trả lời")
                                            .clicked()
                                        {
                                            action = Some(ChatUiAction::ClearQuote);
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }

                    // Multi-line Auto-Wrapping Prompt Editor
                    let text_edit = egui::TextEdit::multiline(input_prompt)
                        .desired_rows(2)
                        .desired_width(ui.available_width())
                        .hint_text("Ask anything... @ files, $ skills, / commands (Enter để gửi, Shift+Enter xuống dòng)");
                    let resp = ui.add(text_edit);

                    // Enter without Shift submits prompt
                    let enter_submit = resp.has_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
                    if enter_submit {
                        // Trim trailing newline added by Enter key
                        while input_prompt.ends_with('\n') || input_prompt.ends_with('\r') {
                            input_prompt.pop();
                        }
                        if !input_prompt.trim().is_empty() {
                            action = Some(ChatUiAction::SendPrompt);
                        }
                    }

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("@ File").color(theme.text_silver).size(10.5))
                            .on_hover_text("Đính kèm file trong repo vào prompt")
                            .clicked()
                        {
                            *show_mention_popup = !*show_mention_popup;
                            *show_skill_popup = false;
                        }

                        if ui
                            .button(
                                egui::RichText::new("$ Skill / Cmd")
                                    .color(theme.text_silver)
                                    .size(10.5),
                            )
                            .on_hover_text("Chèn Skill hoặc Slash Command")
                            .clicked()
                        {
                            *show_skill_popup = !*show_skill_popup;
                            *show_mention_popup = false;
                        }

                        Frame::none()
                            .fill(Color32::from_rgb(0x1A, 0x1E, 0x28))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} {}",
                                        active_agent.icon(),
                                        active_agent.name()
                                    ))
                                    .color(theme.text_silver)
                                    .size(10.0),
                                );
                            });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add_sized(
                                    vec2(58.0, 26.0),
                                    egui::Button::new(
                                        egui::RichText::new("Gửi ⚡")
                                            .color(Color32::BLACK)
                                            .strong()
                                            .size(11.5),
                                    )
                                    .fill(theme.accent_pop)
                                    .rounding(Rounding::same(6.0)),
                                )
                                .clicked()
                                && !input_prompt.trim().is_empty()
                            {
                                action = Some(ChatUiAction::SendPrompt);
                            }
                        });
                    });
                });
            });
    });

    action
}

fn render_single_chat_message(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    msg: &ChatMessage,
) -> Option<ChatUiAction> {
    let mut act = None;

    match msg.role {
        MessageRole::User => {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() * 0.12).max(16.0));
                Frame::none()
                    .fill(Color32::from_rgb(0x18, 0x1C, 0x26))
                    .rounding(Rounding::same(10.0))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(0x28, 0x30, 0x42)))
                    .inner_margin(Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("👤 Ngài Vmax")
                                    .color(theme.accent_pop)
                                    .strong()
                                    .size(11.0),
                            );
                            ui.label(
                                egui::RichText::new(&msg.timestamp)
                                    .color(theme.text_silver)
                                    .size(9.5),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .small_button("❝ Quote")
                                    .on_hover_text("Trích dẫn lại yêu cầu này")
                                    .clicked()
                                {
                                    act = Some(ChatUiAction::SetQuote(QuotedContext {
                                        source_msg_id: msg.id.clone(),
                                        author_label: "Ngài Vmax".to_string(),
                                        snippet: msg.content.clone(),
                                        is_quote_block: true,
                                    }));
                                }
                            });
                        });

                        if let Some(ref q) = msg.quoted {
                            ui.add_space(3.0);
                            render_embedded_quote_box(ui, theme, q);
                        }

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&msg.content)
                                .color(theme.text_paper)
                                .size(12.5),
                        );
                    });
            });
        }
        MessageRole::System => {
            Frame::none()
                .fill(Color32::from_rgb(0x11, 0x15, 0x1E))
                .rounding(Rounding::same(6.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x2A, 0x38)))
                .inner_margin(Margin::symmetric(10.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new("🔔 System")
                                .color(theme.accent_pop)
                                .strong()
                                .size(10.5),
                        );
                        ui.label(
                            egui::RichText::new(&msg.content)
                                .color(theme.text_paper)
                                .size(11.5),
                        );
                    });
                });
        }
        MessageRole::Assistant => {
            Frame::none()
                .fill(Color32::from_rgb(0x0E, 0x10, 0x16))
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(
                    1.0,
                    if msg.is_streaming {
                        theme.accent_pop
                    } else {
                        Color32::from_rgb(0x1C, 0x20, 0x2C)
                    },
                ))
                .inner_margin(Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Header row: Animated Avatar + Agent Title + Processed Duration + Reply/Quote/Copy/Kanban buttons
                    ui.horizontal_wrapped(|ui| {
                        let av_state = if msg.is_streaming {
                            AgentAvatarState::Answering
                        } else {
                            AgentAvatarState::Idle
                        };
                        AvatarManager::render_animated_avatar(
                            ui,
                            msg.agent,
                            av_state,
                            26.0,
                            elapsed_secs,
                            theme.accent_pop,
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{} {}",
                                msg.agent.icon(),
                                msg.agent.title()
                            ))
                            .color(theme.text_paper)
                            .strong()
                            .size(11.5),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "→ Processed {:.1}s · {}",
                                msg.duration_secs, msg.timestamp
                            ))
                            .color(theme.text_silver)
                            .size(10.0),
                        );

                        // Interactive Message Action Bar: Reply, Quote, Copy, Send to Kanban
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .small_button("📊 +Kanban")
                                .on_hover_text("Đẩy nội dung này thành Task trên bảng Swarm Kanban")
                                .clicked()
                            {
                                let first_line = msg
                                    .content
                                    .lines()
                                    .next()
                                    .unwrap_or("AI Follow-up Task")
                                    .chars()
                                    .take(48)
                                    .collect::<String>();
                                act = Some(ChatUiAction::SendMessageToKanban {
                                    title: first_line,
                                    detail: msg.content.clone(),
                                    agent: msg.agent,
                                });
                            }

                            if ui
                                .small_button("📋 Copy")
                                .on_hover_text("Sao chép nội dung tin nhắn")
                                .clicked()
                            {
                                ui.ctx().copy_text(msg.content.clone());
                            }

                            if ui
                                .small_button("❝ Quote")
                                .on_hover_text("Trích dẫn đoạn nội dung/code này để yêu cầu AI chỉnh sửa")
                                .clicked()
                            {
                                act = Some(ChatUiAction::SetQuote(QuotedContext {
                                    source_msg_id: msg.id.clone(),
                                    author_label: msg.agent.title().to_string(),
                                    snippet: msg.content.clone(),
                                    is_quote_block: true,
                                }));
                            }

                            if ui
                                .small_button("↩ Reply")
                                .on_hover_text("Trả lời trực tiếp tin nhắn này")
                                .clicked()
                            {
                                act = Some(ChatUiAction::SetQuote(QuotedContext {
                                    source_msg_id: msg.id.clone(),
                                    author_label: msg.agent.title().to_string(),
                                    snippet: msg
                                        .content
                                        .lines()
                                        .next()
                                        .unwrap_or("")
                                        .to_string(),
                                    is_quote_block: false,
                                }));
                            }
                        });
                    });

                    // Quoted context if this response was replying to a specific quote
                    if let Some(ref q) = msg.quoted {
                        ui.add_space(4.0);
                        render_embedded_quote_box(ui, theme, q);
                    }

                    // Nested Execution Steps (Qoder Tree View)
                    if !msg.steps.is_empty() {
                        ui.add_space(5.0);
                        Frame::none()
                            .fill(Color32::from_rgb(0x0A, 0x0C, 0x10))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(Margin::symmetric(10.0, 6.0))
                            .show(ui, |ui| {
                                for step in &msg.steps {
                                    ui.horizontal_wrapped(|ui| {
                                        let col = if step.is_done {
                                            Color32::from_rgb(0x4E, 0xC9, 0xB0)
                                        } else {
                                            theme.accent_pop
                                        };
                                        ui.label(
                                            egui::RichText::new(&step.icon)
                                                .color(col)
                                                .strong()
                                                .size(10.5),
                                        );
                                        ui.label(
                                            egui::RichText::new(&step.label)
                                                .color(theme.text_paper)
                                                .strong()
                                                .size(10.5),
                                        );
                                        ui.label(
                                            egui::RichText::new(&step.detail)
                                                .color(theme.text_silver)
                                                .size(10.5),
                                        );
                                    });
                                }
                            });
                    }

                    // Main Content Body
                    ui.add_space(6.0);
                    if msg.content.is_empty() && msg.is_streaming {
                        ui.label(
                            egui::RichText::new("⚡ Đang nhận luồng suy luận từ Antigravity CLI...")
                                .color(theme.accent_pop)
                                .italics()
                                .size(11.5),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(&msg.content)
                                .color(theme.text_paper)
                                .size(12.2),
                        );
                    }
                });
        }
    }

    act
}

fn render_embedded_quote_box(ui: &mut Ui, theme: &CanhTheme, q: &QuotedContext) {
    Frame::none()
        .fill(Color32::from_rgb(0x13, 0x16, 0x20))
        .rounding(Rounding::same(5.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(0x2A, 0x32, 0x45)))
        .inner_margin(Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            let prefix = if q.is_quote_block { "❝" } else { "↩" };
            ui.label(
                egui::RichText::new(format!(
                    "{} {} ({}):",
                    prefix, q.author_label, q.source_msg_id
                ))
                .color(theme.accent_pop)
                .strong()
                .size(10.0),
            );
            let snippet_preview: String = q.snippet.chars().take(180).collect();
            ui.label(
                egui::RichText::new(snippet_preview)
                    .color(theme.text_silver)
                    .italics()
                    .size(10.5),
            );
        });
}
