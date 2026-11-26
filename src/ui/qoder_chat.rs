use crate::ai::{
    AgentAvatarState, ChatSessionManager, ElegansAgent, ExecutionMode, MessageRole, QuotedContext,
    ThreadAgentMode, ALL_AGENTS,
};
use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use crate::ui::header::icon_btn;
use egui::text::LayoutJob;
use egui::{
    vec2, Align, Color32, FontFamily, FontId, Frame, Layout, Margin, Rounding, ScrollArea, Stroke,
    TextFormat, Ui,
};

pub enum ChatUiAction {
    SendPrompt,
    StopStream,
    SetExecutionMode(ExecutionMode),
    SetThreadAgentMode(ThreadAgentMode),
    OpenDedicatedCoreAgent(ElegansAgent),
    OpenDedicatedCustomAgent(usize),
    ToggleAttachFile(String),
    ToggleAttachSkill(String),
    OpenFileInEditor {
        path: String,
        edit_mode: bool,
    },
    ApplyCodeBlockToEditor(String),
    RunQuickSlashCommand(String),
    RunSecurityDiagnostics,
    SetQuote(QuotedContext),
    ClearQuote,
    SendMessageToKanban {
        title: String,
        detail: String,
        agent: ElegansAgent,
    },
}

#[derive(Default)]
pub struct ChatComposerUiState {
    pub show_context_popup: bool, // @files, $skills, /commands
    pub context_tab: usize,       // 0 = @Files, 1 = $Skills, 2 = /Commands
    pub context_filter: String,
    pub show_agent_popup: bool, // 1-on-1 Agent & Swarm selector popover
    pub expanded_step_msgs: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn render_qoder_chat_timeline(
    ui: &mut Ui,
    theme: &CanhTheme,
    _elapsed_secs: f32,
    project_name: &str,
    branch_name: &str,
    active_agent: ElegansAgent,
    avatar_state: AgentAvatarState,
    exec_mode: ExecutionMode,
    chat_mgr: &ChatSessionManager,
    file_changes: &[GitFileChange],
    workspace_files: &[String],
    skills: &[DiscoveredSkill],
    custom_agents: &[DiscoveredAgent],
    input_prompt: &mut String,
    composer_state: &mut ChatComposerUiState,
) -> Option<ChatUiAction> {
    let mut action = None;
    let is_streaming = avatar_state == AgentAvatarState::Thinking
        || avatar_state == AgentAvatarState::Answering;
    let thread = chat_mgr.active_thread();

    // Auto-trigger popovers when typing @, $, or / at the end of prompt
    if input_prompt.ends_with('@') {
        composer_state.show_context_popup = true;
        composer_state.context_tab = 0;
    } else if input_prompt.ends_with('$') {
        composer_state.show_context_popup = true;
        composer_state.context_tab = 1;
    } else if input_prompt == "/" || input_prompt.ends_with(" /") {
        composer_state.show_context_popup = true;
        composer_state.context_tab = 2;
    }

    let total_w = ui.available_width();
    let reading_w = total_w.min(820.0);
    let side_pad = ((total_w - reading_w) * 0.5).max(0.0);

    ui.vertical(|ui| {
        // 1. Top Session Bar (Centered within reading column)
        ui.horizontal(|ui| {
            ui.add_space(side_pad);
            ui.allocate_ui_with_layout(
                vec2(reading_w, 28.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.label(
                        egui::RichText::new(Lucide::MESSAGE_SQUARE)
                            .color(theme.accent_pop)
                            .size(13.0),
                    );
                    ui.label(
                        egui::RichText::new(&thread.title)
                            .color(theme.text_paper)
                            .strong()
                            .size(13.0),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "· {} / {}",
                            project_name, branch_name
                        ))
                        .color(theme.text_silver)
                        .size(11.0),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if is_streaming
                            && icon_btn(
                                ui,
                                Lucide::STOP,
                                Color32::WHITE,
                                theme.danger,
                                "Dừng tiến trình AI Stream ngay lập tức",
                            )
                            .clicked()
                        {
                            action = Some(ChatUiAction::StopStream);
                        }

                        // Active Agent Mode Pill
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{} {} {}",
                                        Lucide::BOT,
                                        thread.agent_mode.badge_label(active_agent),
                                        Lucide::CHEVRON_DOWN
                                    ))
                                    .color(theme.text_paper)
                                    .size(11.0),
                                )
                                .fill(theme.surface_1)
                                .stroke(Stroke::new(1.0, theme.border))
                                .rounding(Rounding::same(6.0)),
                            )
                            .on_hover_text(format!(
                                "{}\nClick để đổi Agent hoặc mở Chat 1-1",
                                thread.agent_mode.role_description()
                            ))
                            .clicked()
                        {
                            composer_state.show_agent_popup = !composer_state.show_agent_popup;
                        }
                    });
                },
            );
        });

        ui.add_space(6.0);

        // 2. Main Chat Timeline Scroll Area
        let reserved_bottom =
            if composer_state.show_context_popup || composer_state.show_agent_popup {
                275.0
            } else {
                126.0
            };
        let scroll_h = (ui.available_height() - reserved_bottom).max(100.0);

        ScrollArea::vertical()
            .id_salt("agy_center_chat_scroll")
            .auto_shrink([false, false])
            .max_height(scroll_h)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(side_pad);
                    ui.allocate_ui_with_layout(
                        vec2(reading_w, ui.available_height()),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_width(reading_w);

                            for msg in &thread.messages {
                                match msg.role {
                                    MessageRole::User => {
                                        // User Prompt Bubble (Right-aligned warm surface card)
                                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                            let max_bubble_w = (reading_w * 0.78).max(260.0);
                                            Frame::none()
                                                .fill(theme.surface_2)
                                                .rounding(Rounding::same(12.0))
                                                .stroke(Stroke::new(1.0, theme.border_strong))
                                                .inner_margin(Margin::symmetric(14.0, 10.0))
                                                .show(ui, |ui| {
                                                    ui.set_max_width(max_bubble_w);
                                                    ui.vertical(|ui| {
                                                        if let Some(ref q) = msg.quoted {
                                                            Frame::none()
                                                                .fill(theme.surface_1)
                                                                .rounding(Rounding::same(6.0))
                                                                .inner_margin(Margin::symmetric(
                                                                    8.0, 4.0,
                                                                ))
                                                                .show(ui, |ui| {
                                                                    ui.label(
                                                                        egui::RichText::new(
                                                                            format!(
                                                                                "{} {}",
                                                                                Lucide::QUOTE,
                                                                                q.author_label
                                                                            ),
                                                                        )
                                                                        .color(theme.accent_pop)
                                                                        .strong()
                                                                        .size(10.5),
                                                                    );
                                                                    let short_q: String = q
                                                                        .snippet
                                                                        .chars()
                                                                        .take(120)
                                                                        .collect();
                                                                    ui.label(
                                                                        egui::RichText::new(
                                                                            short_q,
                                                                        )
                                                                        .color(theme.text_silver)
                                                                        .monospace()
                                                                        .size(10.5),
                                                                    );
                                                                });
                                                            ui.add_space(4.0);
                                                        }
                                                        render_inline_markdown_line(
                                                            ui,
                                                            theme,
                                                            &msg.content,
                                                            13.0,
                                                            theme.text_paper,
                                                        );
                                                    });
                                                });
                                        });
                                        ui.add_space(16.0);
                                    }
                                    MessageRole::Assistant => {
                                        // By default, steps are collapsed unless streaming or explicitly expanded
                                        let is_expanded = msg.is_streaming
                                            || composer_state
                                                .expanded_step_msgs
                                                .contains(&msg.id);

                                        Frame::none()
                                            .fill(theme.panel_bg)
                                            .rounding(Rounding::same(10.0))
                                            .stroke(Stroke::new(1.0, theme.border))
                                            .inner_margin(Margin::symmetric(16.0, 12.0))
                                            .show(ui, |ui| {
                                                ui.set_width((reading_w - 34.0).max(200.0));
                                                ui.vertical(|ui| {
                                                    // Single Clean Agy Header Line: [Agent Icon & Name] + [Collapsible Thought/Step Pill] + [Time]
                                                    ui.horizontal(|ui| {
                                                        let agent_header_name =
                                                            match &thread.agent_mode {
                                                                ThreadAgentMode::DedicatedCustom {
                                                                    name,
                                                                    ..
                                                                } => format!("@{}", name),
                                                                _ => msg.agent.name().to_string(),
                                                            };
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "{} {}",
                                                                msg.agent.icon(),
                                                                agent_header_name
                                                            ))
                                                            .color(theme.text_paper)
                                                            .strong()
                                                            .size(12.2),
                                                        );
                                                        ui.label(
                                                            egui::RichText::new(msg.agent.title())
                                                                .color(theme.text_silver)
                                                                .size(11.0),
                                                        );

                                                        if !msg.steps.is_empty() {
                                                            ui.add_space(4.0);
                                                            let chev = if is_expanded {
                                                                Lucide::CHEVRON_DOWN
                                                            } else {
                                                                Lucide::CHEVRON_RIGHT
                                                            };
                                                            let step_pill_txt = if msg.is_streaming
                                                            {
                                                                format!(
                                                                    "{} Đang xử lý ({:.1}s · {} bước)",
                                                                    chev,
                                                                    msg.duration_secs,
                                                                    msg.steps.len()
                                                                )
                                                            } else {
                                                                format!(
                                                                    "{} Đã xử lý {:.1}s · {} bước",
                                                                    chev,
                                                                    msg.duration_secs.max(0.2),
                                                                    msg.steps.len()
                                                                )
                                                            };

                                                            if ui
                                                                .add(
                                                                    egui::Button::new(
                                                                        egui::RichText::new(
                                                                            step_pill_txt,
                                                                        )
                                                                        .color(
                                                                            if msg.is_streaming {
                                                                                theme.accent_pop
                                                                            } else {
                                                                                theme.text_silver
                                                                            },
                                                                        )
                                                                        .size(10.5),
                                                                    )
                                                                    .fill(theme.surface_1)
                                                                    .rounding(Rounding::same(5.0)),
                                                                )
                                                                .on_hover_text(
                                                                    "Click để xem chi tiết các bước thực thi",
                                                                )
                                                                .clicked()
                                                            {
                                                                if let Some(pos) = composer_state
                                                                    .expanded_step_msgs
                                                                    .iter()
                                                                    .position(|id| id == &msg.id)
                                                                {
                                                                    composer_state
                                                                        .expanded_step_msgs
                                                                        .remove(pos);
                                                                } else {
                                                                    composer_state
                                                                        .expanded_step_msgs
                                                                        .push(msg.id.clone());
                                                                }
                                                            }
                                                        }

                                                        ui.with_layout(
                                                            Layout::right_to_left(Align::Center),
                                                            |ui| {
                                                                ui.label(
                                                                    egui::RichText::new(
                                                                        &msg.timestamp,
                                                                    )
                                                                    .color(theme.text_silver)
                                                                    .size(10.5),
                                                                );
                                                            },
                                                        );
                                                    });

                                                    // Expandable Agy Execution Steps Box
                                                    if is_expanded && !msg.steps.is_empty() {
                                                        ui.add_space(6.0);
                                                        Frame::none()
                                                            .fill(theme.surface_1)
                                                            .rounding(Rounding::same(6.0))
                                                            .stroke(Stroke::new(
                                                                1.0,
                                                                theme.border,
                                                            ))
                                                            .inner_margin(Margin::symmetric(
                                                                10.0, 6.0,
                                                            ))
                                                            .show(ui, |ui| {
                                                                for st in &msg.steps {
                                                                    ui.horizontal_wrapped(|ui| {
                                                                        ui.label(
                                                                            egui::RichText::new(
                                                                                Lucide::CHECK,
                                                                            )
                                                                            .color(theme.success)
                                                                            .size(10.5),
                                                                        );
                                                                        ui.label(
                                                                            egui::RichText::new(
                                                                                &st.label,
                                                                            )
                                                                            .color(
                                                                                theme.text_paper,
                                                                            )
                                                                            .strong()
                                                                            .size(10.8),
                                                                        );
                                                                        ui.label(
                                                                            egui::RichText::new(
                                                                                &st.detail,
                                                                            )
                                                                            .color(
                                                                                theme.text_silver,
                                                                            )
                                                                            .size(10.5),
                                                                        );
                                                                    });
                                                                }
                                                            });
                                                    }

                                                    ui.add_space(8.0);

                                                    // Rich Markdown Message Content (Parses **bold**, `code`, lists, headers, code blocks)
                                                    render_formatted_message_content(
                                                        ui,
                                                        theme,
                                                        &msg.content,
                                                        &mut action,
                                                    );

                                                    if msg.is_streaming {
                                                        ui.add_space(6.0);
                                                        ui.horizontal(|ui| {
                                                            let (r, _) = ui.allocate_exact_size(
                                                                vec2(8.0, 8.0),
                                                                egui::Sense::hover(),
                                                            );
                                                            ui.painter().circle_filled(
                                                                r.center(),
                                                                3.0,
                                                                theme.accent_pop,
                                                            );
                                                            ui.label(
                                                                egui::RichText::new(
                                                                    "Đang trả lời...",
                                                                )
                                                                .color(theme.accent_pop)
                                                                .size(11.0),
                                                            );
                                                        });
                                                    }

                                                    // Subtle Footer Action Bar
                                                    ui.add_space(8.0);
                                                    ui.horizontal(|ui| {
                                                        if icon_btn(
                                                            ui,
                                                            Lucide::COPY,
                                                            theme.text_silver,
                                                            Color32::TRANSPARENT,
                                                            "Sao chép nội dung tin nhắn",
                                                        )
                                                        .clicked()
                                                        {
                                                            ui.ctx().copy_text(msg.content.clone());
                                                        }
                                                        if icon_btn(
                                                            ui,
                                                            Lucide::QUOTE,
                                                            theme.text_silver,
                                                            Color32::TRANSPARENT,
                                                            "Trích dẫn (Quote) vào câu hỏi tiếp theo",
                                                        )
                                                        .clicked()
                                                        {
                                                            action = Some(ChatUiAction::SetQuote(
                                                                QuotedContext {
                                                                    source_msg_id: msg.id.clone(),
                                                                    author_label: msg
                                                                        .agent
                                                                        .name()
                                                                        .to_string(),
                                                                    snippet: msg.content.clone(),
                                                                    is_quote_block: true,
                                                                },
                                                            ));
                                                        }
                                                        if icon_btn(
                                                            ui,
                                                            Lucide::REPLY,
                                                            theme.text_silver,
                                                            Color32::TRANSPARENT,
                                                            "Trả lời (Reply) tin nhắn này",
                                                        )
                                                        .clicked()
                                                        {
                                                            action = Some(ChatUiAction::SetQuote(
                                                                QuotedContext {
                                                                    source_msg_id: msg.id.clone(),
                                                                    author_label: msg
                                                                        .agent
                                                                        .name()
                                                                        .to_string(),
                                                                    snippet: msg.content.clone(),
                                                                    is_quote_block: false,
                                                                },
                                                            ));
                                                        }
                                                        if icon_btn(
                                                            ui,
                                                            Lucide::KANBAN,
                                                            theme.text_silver,
                                                            Color32::TRANSPARENT,
                                                            "Chuyển nội dung này thành thẻ tác vụ trên Kanban",
                                                        )
                                                        .clicked()
                                                        {
                                                            action = Some(
                                                                ChatUiAction::SendMessageToKanban {
                                                                    title: msg
                                                                        .content
                                                                        .lines()
                                                                        .next()
                                                                        .unwrap_or("Task")
                                                                        .replace("**", "")
                                                                        .chars()
                                                                        .take(48)
                                                                        .collect(),
                                                                    detail: msg.content.clone(),
                                                                    agent: msg.agent,
                                                                },
                                                            );
                                                        }
                                                    });
                                                });
                                            });
                                        ui.add_space(14.0);
                                    }
                                    MessageRole::System => {
                                        Frame::none()
                                            .fill(theme.surface_1)
                                            .rounding(Rounding::same(8.0))
                                            .stroke(Stroke::new(1.0, theme.border))
                                            .inner_margin(Margin::symmetric(12.0, 8.0))
                                            .show(ui, |ui| {
                                                render_formatted_message_content(
                                                    ui,
                                                    theme,
                                                    &msg.content,
                                                    &mut action,
                                                );
                                            });
                                        ui.add_space(10.0);
                                    }
                                }
                            }

                            // 2b. Agy Quick-Start Action Cards when session only has the initial welcome message
                            if thread.messages.len() <= 1 {
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new("GỢI Ý BẮT ĐẦU NHANH (QUICK ACTIONS)")
                                        .color(theme.text_silver)
                                        .strong()
                                        .size(10.5),
                                );
                                ui.add_space(6.0);

                                let card_w = ((reading_w - 12.0) * 0.5).max(180.0);
                                let quick_actions = [
                                    (
                                        Lucide::GIT_COMMIT,
                                        "Review Git Diff hiện tại",
                                        "Đính kèm các thay đổi chưa commit và kiểm tra lỗi",
                                        "/diff",
                                    ),
                                    (
                                        Lucide::KANBAN,
                                        "Mở bảng điều phối Swarm Kanban",
                                        "Quản lý tiến độ tác vụ 4 cột & chạy Agent dưới nền",
                                        "/kanban",
                                    ),
                                    (
                                        Lucide::SHIELD,
                                        "Quét bảo mật & AST Blast Radius",
                                        "Kiểm tra lộ API key/Secret và thống kê cấu trúc code",
                                        "/env-guard",
                                    ),
                                    (
                                        Lucide::CODE,
                                        "Mở Live Code & Diff Editor",
                                        "Xem so sánh Diff trực quan và chỉnh sửa file trực tiếp",
                                        "/editor",
                                    ),
                                ];

                                for pair in quick_actions.chunks(2) {
                                    ui.horizontal(|ui| {
                                        for (ic, title, sub, cmd) in pair {
                                            let q_resp = Frame::none()
                                                .fill(theme.surface_1)
                                                .rounding(Rounding::same(8.0))
                                                .stroke(Stroke::new(1.0, theme.border))
                                                .inner_margin(Margin::symmetric(12.0, 9.0))
                                                .show(ui, |ui| {
                                                    ui.set_width((card_w - 26.0).max(140.0));
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            egui::RichText::new(*ic)
                                                                .color(theme.accent_pop)
                                                                .size(14.0),
                                                        );
                                                        ui.vertical(|ui| {
                                                            ui.label(
                                                                egui::RichText::new(*title)
                                                                    .color(theme.text_paper)
                                                                    .strong()
                                                                    .size(11.8),
                                                            );
                                                            ui.label(
                                                                egui::RichText::new(*sub)
                                                                    .color(theme.text_silver)
                                                                    .size(10.2),
                                                            );
                                                        });
                                                    });
                                                });
                                            if q_resp
                                                .response
                                                .interact(egui::Sense::click())
                                                .clicked()
                                            {
                                                action = Some(ChatUiAction::RunQuickSlashCommand(
                                                    (*cmd).to_string(),
                                                ));
                                            }
                                            ui.add_space(6.0);
                                        }
                                    });
                                    ui.add_space(6.0);
                                }
                            }
                        },
                    );
                });
            });

        ui.add_space(6.0);

        // 3. Centered Bottom Composer & Popovers
        ui.horizontal(|ui| {
            ui.add_space(side_pad);
            ui.allocate_ui_with_layout(
                vec2(reading_w, ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(reading_w);

                    // 3a. Interactive Floating Popover: @Files / $Skills / /Commands
                    if composer_state.show_context_popup {
                        Frame::none()
                            .fill(theme.panel_bg)
                            .rounding(Rounding::same(10.0))
                            .stroke(Stroke::new(1.0, theme.border_strong))
                            .inner_margin(Margin::same(10.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    for (t_idx, t_icon, t_tip) in [
                                        (
                                            0,
                                            format!(
                                                "{} Files ({})",
                                                Lucide::AT_SIGN,
                                                workspace_files.len()
                                            ),
                                            "Đính kèm File vào ngữ cảnh (@Files)",
                                        ),
                                        (
                                            1,
                                            format!(
                                                "{} Skills ({})",
                                                Lucide::WAND,
                                                skills.len()
                                            ),
                                            "Kích hoạt Skill ($Skills)",
                                        ),
                                        (
                                            2,
                                            format!("{} Lệnh nhanh", Lucide::TERMINAL),
                                            "Chạy lệnh nhanh (/Commands)",
                                        ),
                                    ] {
                                        let is_tab = composer_state.context_tab == t_idx;
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(t_icon)
                                                        .color(if is_tab {
                                                            theme.on_accent
                                                        } else {
                                                            theme.text_paper
                                                        })
                                                        .strong()
                                                        .size(11.0),
                                                )
                                                .fill(if is_tab {
                                                    theme.accent_pop
                                                } else {
                                                    theme.chip_bg
                                                })
                                                .rounding(Rounding::same(5.0)),
                                            )
                                            .on_hover_text(t_tip)
                                            .clicked()
                                        {
                                            composer_state.context_tab = t_idx;
                                        }
                                    }

                                    ui.add_space(6.0);
                                    Frame::none()
                                        .fill(theme.input_bg)
                                        .rounding(Rounding::same(5.0))
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .inner_margin(Margin::symmetric(8.0, 3.0))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                vec2(180.0, 18.0),
                                                egui::TextEdit::singleline(
                                                    &mut composer_state.context_filter,
                                                )
                                                .hint_text("Lọc nhanh...")
                                                .frame(false)
                                                .text_color(theme.text_paper),
                                            );
                                        });

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if icon_btn(
                                            ui,
                                            Lucide::X,
                                            theme.text_silver,
                                            Color32::TRANSPARENT,
                                            "Đóng bảng chọn",
                                        )
                                        .clicked()
                                        {
                                            composer_state.show_context_popup = false;
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                                let q = composer_state.context_filter.trim().to_lowercase();

                                ScrollArea::vertical()
                                    .id_salt("chat_context_popover_scroll")
                                    .max_height(115.0)
                                    .show(ui, |ui| match composer_state.context_tab {
                                        0 => {
                                            if !file_changes.is_empty() {
                                                ui.label(
                                                    egui::RichText::new("FILE ĐANG THAY ĐỔI (GIT):")
                                                        .color(theme.accent_pop)
                                                        .strong()
                                                        .size(10.0),
                                                );
                                                for fc in file_changes {
                                                    if !q.is_empty()
                                                        && !fc.path.to_lowercase().contains(&q)
                                                    {
                                                        continue;
                                                    }
                                                    let attached =
                                                        thread.attached_files.contains(&fc.path);
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                egui::RichText::new(format!(
                                                                    "{} [{}] @{} (+{} -{})",
                                                                    if attached {
                                                                        Lucide::CHECK
                                                                    } else {
                                                                        Lucide::PLUS
                                                                    },
                                                                    fc.status,
                                                                    fc.path,
                                                                    fc.added_lines,
                                                                    fc.removed_lines
                                                                ))
                                                                .color(if attached {
                                                                    theme.success
                                                                } else {
                                                                    theme.text_paper
                                                                })
                                                                .size(11.0),
                                                            )
                                                            .fill(Color32::TRANSPARENT),
                                                        )
                                                        .clicked()
                                                    {
                                                        if input_prompt.ends_with('@') {
                                                            input_prompt.pop();
                                                        }
                                                        action = Some(
                                                            ChatUiAction::ToggleAttachFile(
                                                                fc.path.clone(),
                                                            ),
                                                        );
                                                    }
                                                }
                                                ui.add_space(4.0);
                                            }

                                            ui.label(
                                                egui::RichText::new("TẤT CẢ FILE TRONG PROJECT:")
                                                    .color(theme.text_silver)
                                                    .strong()
                                                    .size(10.0),
                                            );
                                            for wf in workspace_files.iter().take(80) {
                                                if !q.is_empty() && !wf.to_lowercase().contains(&q)
                                                {
                                                    continue;
                                                }
                                                let attached = thread.attached_files.contains(wf);
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new(format!(
                                                                "{} @{}",
                                                                if attached {
                                                                    Lucide::CHECK
                                                                } else {
                                                                    Lucide::FILE_TEXT
                                                                },
                                                                wf
                                                            ))
                                                            .color(if attached {
                                                                theme.success
                                                            } else {
                                                                theme.text_paper
                                                            })
                                                            .size(11.0),
                                                        )
                                                        .fill(Color32::TRANSPARENT),
                                                    )
                                                    .clicked()
                                                {
                                                    if input_prompt.ends_with('@') {
                                                        input_prompt.pop();
                                                    }
                                                    action = Some(ChatUiAction::ToggleAttachFile(
                                                        wf.clone(),
                                                    ));
                                                }
                                            }
                                        }
                                        1 => {
                                            for sk in skills {
                                                if !q.is_empty()
                                                    && !sk.name.to_lowercase().contains(&q)
                                                    && !sk.description.to_lowercase().contains(&q)
                                                {
                                                    continue;
                                                }
                                                let attached =
                                                    thread.attached_skills.contains(&sk.name);
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new(format!(
                                                                "{} ${} — {}",
                                                                if attached {
                                                                    Lucide::CHECK
                                                                } else {
                                                                    Lucide::WAND
                                                                },
                                                                sk.name,
                                                                sk.description
                                                            ))
                                                            .color(if attached {
                                                                theme.accent_pop
                                                            } else {
                                                                theme.text_paper
                                                            })
                                                            .size(11.0),
                                                        )
                                                        .fill(Color32::TRANSPARENT),
                                                    )
                                                    .clicked()
                                                {
                                                    if input_prompt.ends_with('$') {
                                                        input_prompt.pop();
                                                    }
                                                    action = Some(
                                                        ChatUiAction::ToggleAttachSkill(
                                                            sk.name.clone(),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                        _ => {
                                            let cmds = [
                                                (
                                                    "/diff",
                                                    "Đính kèm toàn bộ Git Diff hiện tại và yêu cầu AI review",
                                                ),
                                                (
                                                    "/commit",
                                                    "Stage tất cả file & tạo Git Commit tự động",
                                                ),
                                                (
                                                    "/review",
                                                    "Chuyển sang 1-on-1 với fe_code_auditor để kiểm tra lỗi",
                                                ),
                                                (
                                                    "/env-guard",
                                                    "Quét bảo mật Elegans: Tìm khóa API / Secret bị lộ",
                                                ),
                                                (
                                                    "/ast-blast",
                                                    "Phân tích đồ thị phụ thuộc AST Blast Radius",
                                                ),
                                                (
                                                    "/editor",
                                                    "Mở trình sửa code trực tiếp & xem Diff",
                                                ),
                                                ("/kanban", "Mở bảng điều phối Swarm Kanban Board"),
                                                ("/clear", "Làm mới phiên chat hiện tại"),
                                            ];
                                            for (cmd, desc) in cmds {
                                                if !q.is_empty()
                                                    && !cmd.contains(&q)
                                                    && !desc.to_lowercase().contains(&q)
                                                {
                                                    continue;
                                                }
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new(format!(
                                                                "{} — {}",
                                                                cmd, desc
                                                            ))
                                                            .color(theme.text_paper)
                                                            .size(11.0),
                                                        )
                                                        .fill(Color32::TRANSPARENT),
                                                    )
                                                    .clicked()
                                                {
                                                    composer_state.show_context_popup = false;
                                                    action = Some(
                                                        ChatUiAction::RunQuickSlashCommand(
                                                            cmd.to_string(),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    });
                            });
                        ui.add_space(4.0);
                    }

                    // 3b. Interactive Floating Popover: Dedicated Agent 1-on-1 & Swarm Selector
                    if composer_state.show_agent_popup {
                        Frame::none()
                            .fill(theme.panel_bg)
                            .rounding(Rounding::same(10.0))
                            .stroke(Stroke::new(1.0, theme.border_strong))
                            .inner_margin(Margin::same(10.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} CHỌN AGENT ĐIỀU PHỐI HOẶC CHAT 1-1",
                                            Lucide::BOT
                                        ))
                                        .color(theme.accent_pop)
                                        .strong()
                                        .size(11.0),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if icon_btn(
                                            ui,
                                            Lucide::X,
                                            theme.text_silver,
                                            Color32::TRANSPARENT,
                                            "Đóng",
                                        )
                                        .clicked()
                                        {
                                            composer_state.show_agent_popup = false;
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                                ScrollArea::vertical()
                                    .id_salt("agent_selector_popover_scroll")
                                    .max_height(120.0)
                                    .show(ui, |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(format!(
                                                        "{} Swarm Auto-Router (Tự động chọn Agent theo câu hỏi)",
                                                        Lucide::SPARKLES
                                                    ))
                                                    .color(
                                                        if matches!(
                                                            thread.agent_mode,
                                                            ThreadAgentMode::SwarmAuto
                                                        ) {
                                                            theme.accent_pop
                                                        } else {
                                                            theme.text_paper
                                                        },
                                                    )
                                                    .strong()
                                                    .size(11.2),
                                                )
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .clicked()
                                        {
                                            composer_state.show_agent_popup = false;
                                            action = Some(ChatUiAction::SetThreadAgentMode(
                                                ThreadAgentMode::SwarmAuto,
                                            ));
                                        }

                                        ui.add_space(4.0);
                                        for &ag in ALL_AGENTS {
                                            let is_cur = thread.agent_mode
                                                == ThreadAgentMode::DedicatedCore(ag);
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new(format!(
                                                            "{} {} — {}",
                                                            ag.icon(),
                                                            ag.name(),
                                                            ag.title()
                                                        ))
                                                        .color(if is_cur {
                                                            theme.accent_pop
                                                        } else {
                                                            theme.text_paper
                                                        })
                                                        .size(11.0),
                                                    )
                                                    .fill(Color32::TRANSPARENT),
                                                )
                                                .clicked()
                                            {
                                                composer_state.show_agent_popup = false;
                                                action =
                                                    Some(ChatUiAction::OpenDedicatedCoreAgent(ag));
                                            }
                                        }

                                        if !custom_agents.is_empty() {
                                            ui.add_space(4.0);
                                            for (idx, ca) in custom_agents.iter().enumerate() {
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new(format!(
                                                                "{} @{} — {}",
                                                                Lucide::BOT,
                                                                ca.name,
                                                                ca.role
                                                            ))
                                                            .color(theme.text_paper)
                                                            .size(11.0),
                                                        )
                                                        .fill(Color32::TRANSPARENT),
                                                    )
                                                    .clicked()
                                                {
                                                    composer_state.show_agent_popup = false;
                                                    action = Some(
                                                        ChatUiAction::OpenDedicatedCustomAgent(idx),
                                                    );
                                                }
                                            }
                                        }
                                    });
                            });
                        ui.add_space(4.0);
                    }

                    // 3c. Agy Elevated Prompt Composer Card
                    let border_col = if !input_prompt.trim().is_empty() {
                        theme.accent_pop
                    } else {
                        theme.border_strong
                    };
                    Frame::none()
                        .fill(theme.surface_1)
                        .rounding(Rounding::same(12.0))
                        .stroke(Stroke::new(1.0, border_col))
                        .inner_margin(Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                // Active Quote / Reply Banner
                                if let Some(ref q) = chat_mgr.active_quote {
                                    ui.horizontal(|ui| {
                                        let mode_ic = if q.is_quote_block {
                                            Lucide::QUOTE
                                        } else {
                                            Lucide::REPLY
                                        };
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} {}:",
                                                mode_ic, q.author_label
                                            ))
                                            .color(theme.accent_pop)
                                            .strong()
                                            .size(10.5),
                                        );
                                        let short_snip: String = q
                                            .snippet
                                            .lines()
                                            .next()
                                            .unwrap_or("")
                                            .chars()
                                            .take(70)
                                            .collect();
                                        ui.label(
                                            egui::RichText::new(short_snip)
                                                .color(theme.text_silver)
                                                .size(10.5),
                                        );
                                        if icon_btn(
                                            ui,
                                            Lucide::X,
                                            theme.text_silver,
                                            Color32::TRANSPARENT,
                                            "Xóa trích dẫn",
                                        )
                                        .clicked()
                                        {
                                            action = Some(ChatUiAction::ClearQuote);
                                        }
                                    });
                                    ui.add_space(4.0);
                                }

                                // Attached @Files & $Skills Chips Row
                                if !thread.attached_files.is_empty()
                                    || !thread.attached_skills.is_empty()
                                {
                                    ui.horizontal_wrapped(|ui| {
                                        for f_path in &thread.attached_files {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new(format!(
                                                            "{} @{} {}",
                                                            Lucide::FILE_TEXT,
                                                            f_path,
                                                            Lucide::X
                                                        ))
                                                        .color(theme.text_paper)
                                                        .size(10.2),
                                                    )
                                                    .fill(theme.chip_bg)
                                                    .rounding(Rounding::same(5.0)),
                                                )
                                                .on_hover_text("Click để gỡ file này")
                                                .clicked()
                                            {
                                                action = Some(ChatUiAction::ToggleAttachFile(
                                                    f_path.clone(),
                                                ));
                                            }
                                        }
                                        for sk_name in &thread.attached_skills {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new(format!(
                                                            "{} ${} {}",
                                                            Lucide::WAND,
                                                            sk_name,
                                                            Lucide::X
                                                        ))
                                                        .color(theme.accent_pop)
                                                        .size(10.2),
                                                    )
                                                    .fill(theme.chip_bg)
                                                    .rounding(Rounding::same(5.0)),
                                                )
                                                .on_hover_text("Click để gỡ Skill này")
                                                .clicked()
                                            {
                                                action = Some(ChatUiAction::ToggleAttachSkill(
                                                    sk_name.clone(),
                                                ));
                                            }
                                        }
                                    });
                                    ui.add_space(4.0);
                                }

                                // Multiline Prompt Input
                                let hint = match &thread.agent_mode {
                                    ThreadAgentMode::SwarmAuto => {
                                        format!(
                                            "Nhập yêu cầu cho '{}' (nhánh '{}')... Gõ @ đính kèm file, $ gọi skill, / chạy lệnh",
                                            project_name, branch_name
                                        )
                                    }
                                    ThreadAgentMode::DedicatedCore(ag) => {
                                        format!(
                                            "Chat 1-1 với {} ({})... Gõ @ file, $ skill...",
                                            ag.name(),
                                            ag.title()
                                        )
                                    }
                                    ThreadAgentMode::DedicatedCustom { name, role, .. } => {
                                        format!(
                                            "Chat 1-1 với @{} ({})... Gõ @ file, $ skill...",
                                            name, role
                                        )
                                    }
                                };

                                let text_edit = egui::TextEdit::multiline(input_prompt)
                                    .desired_rows(2)
                                    .desired_width(ui.available_width())
                                    .frame(false)
                                    .text_color(theme.text_paper)
                                    .hint_text(hint);
                                let resp = ui.add(text_edit);

                                let enter_submit = resp.has_focus()
                                    && ui.input(|i| {
                                        i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                                    });
                                if enter_submit {
                                    while input_prompt.ends_with('\n')
                                        || input_prompt.ends_with('\r')
                                    {
                                        input_prompt.pop();
                                    }
                                    if !input_prompt.trim().is_empty() {
                                        action = Some(ChatUiAction::SendPrompt);
                                    }
                                }

                                ui.add_space(6.0);

                                // Bottom Composer Toolbar: Clear Labeled Context Pills + Agent & Send Button
                                ui.horizontal(|ui| {
                                    let file_lbl = if thread.attached_files.is_empty() {
                                        format!("{} @File", Lucide::AT_SIGN)
                                    } else {
                                        format!(
                                            "{} @File ({})",
                                            Lucide::AT_SIGN,
                                            thread.attached_files.len()
                                        )
                                    };
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(file_lbl)
                                                    .color(if !thread.attached_files.is_empty() {
                                                        theme.accent_pop
                                                    } else {
                                                        theme.text_silver
                                                    })
                                                    .size(10.8),
                                            )
                                            .fill(theme.chip_bg)
                                            .rounding(Rounding::same(5.0)),
                                        )
                                        .on_hover_text("Đính kèm file trong Project hoặc Git Diff")
                                        .clicked()
                                    {
                                        composer_state.context_tab = 0;
                                        composer_state.show_context_popup =
                                            !composer_state.show_context_popup;
                                    }

                                    let skill_lbl = if thread.attached_skills.is_empty() {
                                        format!("{} $Skill", Lucide::WAND)
                                    } else {
                                        format!(
                                            "{} $Skill ({})",
                                            Lucide::WAND,
                                            thread.attached_skills.len()
                                        )
                                    };
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(skill_lbl)
                                                    .color(if !thread.attached_skills.is_empty() {
                                                        theme.accent_pop
                                                    } else {
                                                        theme.text_silver
                                                    })
                                                    .size(10.8),
                                            )
                                            .fill(theme.chip_bg)
                                            .rounding(Rounding::same(5.0)),
                                        )
                                        .on_hover_text("Kích hoạt Skill (SKILL.md)")
                                        .clicked()
                                    {
                                        composer_state.context_tab = 1;
                                        composer_state.show_context_popup =
                                            !composer_state.show_context_popup;
                                    }

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} /Lệnh",
                                                    Lucide::TERMINAL
                                                ))
                                                .color(theme.text_silver)
                                                .size(10.8),
                                            )
                                            .fill(theme.chip_bg)
                                            .rounding(Rounding::same(5.0)),
                                        )
                                        .on_hover_text(
                                            "Lệnh nhanh (/diff, /commit, /review, /kanban)",
                                        )
                                        .clicked()
                                    {
                                        composer_state.context_tab = 2;
                                        composer_state.show_context_popup =
                                            !composer_state.show_context_popup;
                                    }

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        // Send Button (Warm Terracotta Accent Circle)
                                        let can_send = !input_prompt.trim().is_empty();
                                        let send_bg = if can_send {
                                            theme.accent_pop
                                        } else {
                                            theme.chip_bg
                                        };
                                        let send_fg = if can_send {
                                            theme.on_accent
                                        } else {
                                            theme.text_silver
                                        };
                                        if ui
                                            .add_sized(
                                                vec2(28.0, 24.0),
                                                egui::Button::new(
                                                    egui::RichText::new(Lucide::ARROW_UP)
                                                        .color(send_fg)
                                                        .strong()
                                                        .size(13.0),
                                                )
                                                .fill(send_bg)
                                                .rounding(Rounding::same(6.0)),
                                            )
                                            .on_hover_text("Gửi yêu cầu (Enter)")
                                            .clicked()
                                            && can_send
                                        {
                                            action = Some(ChatUiAction::SendPrompt);
                                        }

                                        ui.add_space(4.0);

                                        // Agent Mode Selector Pill
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(format!(
                                                        "{} {} {}",
                                                        Lucide::SPARKLES,
                                                        thread.agent_mode.short_tag(),
                                                        Lucide::CHEVRON_DOWN
                                                    ))
                                                    .color(theme.text_paper)
                                                    .size(10.8),
                                                )
                                                .fill(theme.chip_bg)
                                                .rounding(Rounding::same(5.0)),
                                            )
                                            .on_hover_text("Chọn Agent hoặc mở Chat 1-1")
                                            .clicked()
                                        {
                                            composer_state.show_agent_popup =
                                                !composer_state.show_agent_popup;
                                        }

                                        ui.add_space(4.0);

                                        // Execution Mode Pill
                                        let (mode_ic, mode_lbl, next_mode) = match exec_mode {
                                            ExecutionMode::Auto => (
                                                Lucide::CPU,
                                                "Auto",
                                                ExecutionMode::Gatekeeper,
                                            ),
                                            ExecutionMode::Gatekeeper => (
                                                Lucide::SHIELD,
                                                "Gate",
                                                ExecutionMode::Bypass,
                                            ),
                                            ExecutionMode::Bypass => (
                                                Lucide::ZAP,
                                                "Fast",
                                                ExecutionMode::Auto,
                                            ),
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(format!(
                                                        "{} {}",
                                                        mode_ic, mode_lbl
                                                    ))
                                                    .color(theme.text_silver)
                                                    .size(10.5),
                                                )
                                                .fill(theme.chip_bg)
                                                .rounding(Rounding::same(5.0)),
                                            )
                                            .on_hover_text("Đổi chế độ thực thi (Auto / Gatekeeper / Fast)")
                                            .clicked()
                                        {
                                            action =
                                                Some(ChatUiAction::SetExecutionMode(next_mode));
                                        }
                                    });
                                });
                            });
                        });
                },
            );
        });
    });

    action
}

/// Parses inline Markdown (`**bold**` and `` `code` ``) into a clean `egui::text::LayoutJob`
/// so raw `**` or backticks are never dumped directly on screen.
pub fn parse_inline_markdown_job(
    text: &str,
    font_size: f32,
    base_color: Color32,
    theme: &CanhTheme,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut normal_buf = String::new();

    let flush_normal = |buf: &mut String, job: &mut LayoutJob| {
        if !buf.is_empty() {
            job.append(
                buf,
                0.0,
                TextFormat {
                    font_id: FontId::new(font_size, FontFamily::Proportional),
                    color: base_color,
                    ..Default::default()
                },
            );
            buf.clear();
        }
    };

    while i < n {
        // 1. Check for **bold**
        if i + 1 < n && chars[i] == '*' && chars[i + 1] == '*' {
            let mut end = i + 2;
            let mut found = false;
            while end + 1 < n {
                if chars[end] == '*' && chars[end + 1] == '*' {
                    found = true;
                    break;
                }
                end += 1;
            }
            if found && end > i + 2 {
                flush_normal(&mut normal_buf, &mut job);
                let bold_str: String = chars[i + 2..end].iter().collect();
                job.append(
                    &bold_str,
                    0.0,
                    TextFormat {
                        font_id: FontId::new(font_size + 0.3, FontFamily::Proportional),
                        color: theme.text_paper,
                        ..Default::default()
                    },
                );
                i = end + 2;
                continue;
            }
        }

        // 2. Check for `inline code`
        if chars[i] == '`' {
            let mut end = i + 1;
            let mut found = false;
            while end < n {
                if chars[end] == '`' {
                    found = true;
                    break;
                }
                end += 1;
            }
            if found && end > i + 1 {
                flush_normal(&mut normal_buf, &mut job);
                let code_str: String = chars[i + 1..end].iter().collect();
                job.append(
                    &code_str,
                    0.0,
                    TextFormat {
                        font_id: FontId::new(font_size - 0.5, FontFamily::Monospace),
                        color: theme.accent_pop,
                        background: theme.code_bg,
                        ..Default::default()
                    },
                );
                i = end + 1;
                continue;
            }
        }

        normal_buf.push(chars[i]);
        i += 1;
    }

    flush_normal(&mut normal_buf, &mut job);
    job
}

fn render_inline_markdown_line(
    ui: &mut Ui,
    theme: &CanhTheme,
    line: &str,
    font_size: f32,
    base_color: Color32,
) {
    let job = parse_inline_markdown_job(line, font_size, base_color, theme);
    ui.add(egui::Label::new(job).wrap());
}

fn render_formatted_message_content(
    ui: &mut Ui,
    theme: &CanhTheme,
    content: &str,
    action: &mut Option<ChatUiAction>,
) {
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if !in_code_block {
                in_code_block = true;
                code_lang = trimmed.trim_start_matches("```").trim().to_string();
                code_buf.clear();
            } else {
                render_code_block_card(ui, theme, &code_lang, code_buf.trim_end(), action);
                ui.add_space(6.0);
                in_code_block = false;
                code_buf.clear();
            }
            continue;
        }

        if in_code_block {
            code_buf.push_str(line);
            code_buf.push('\n');
            continue;
        }

        if trimmed.is_empty() {
            ui.add_space(4.0);
            continue;
        }

        // Headings (#, ##, ###)
        if let Some(h3) = trimmed.strip_prefix("### ") {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(h3.replace("**", ""))
                    .color(theme.text_paper)
                    .strong()
                    .size(13.5),
            );
            ui.add_space(2.0);
        } else if let Some(h2) = trimmed.strip_prefix("## ") {
            ui.add_space(5.0);
            ui.label(
                egui::RichText::new(h2.replace("**", ""))
                    .color(theme.text_paper)
                    .strong()
                    .size(14.5),
            );
            ui.add_space(2.0);
        } else if let Some(h1) = trimmed.strip_prefix("# ") {
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(h1.replace("**", ""))
                    .color(theme.text_paper)
                    .strong()
                    .size(15.5),
            );
            ui.add_space(3.0);
        } else if let Some(bullet) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("• "))
        {
            ui.horizontal_top(|ui| {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("•")
                        .color(theme.accent_pop)
                        .strong()
                        .size(13.0),
                );
                render_inline_markdown_line(ui, theme, bullet, 12.8, theme.text_paper);
            });
            ui.add_space(2.0);
        } else {
            render_inline_markdown_line(ui, theme, line, 12.8, theme.text_paper);
            ui.add_space(2.0);
        }
    }

    if in_code_block && !code_buf.is_empty() {
        render_code_block_card(ui, theme, &code_lang, code_buf.trim_end(), action);
    }
}

fn render_code_block_card(
    ui: &mut Ui,
    theme: &CanhTheme,
    code_lang: &str,
    block_text: &str,
    action: &mut Option<ChatUiAction>,
) {
    let lang_title = if code_lang.is_empty() {
        "code"
    } else {
        code_lang
    };
    Frame::none()
        .fill(theme.code_bg)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} {}", Lucide::CODE, lang_title))
                        .color(theme.accent_pop)
                        .monospace()
                        .strong()
                        .size(10.5),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_btn(
                        ui,
                        Lucide::COPY,
                        theme.text_silver,
                        theme.chip_bg,
                        "Sao chép đoạn code này",
                    )
                    .clicked()
                    {
                        ui.ctx().copy_text(block_text.to_string());
                    }
                    if icon_btn(
                        ui,
                        Lucide::EDIT,
                        theme.accent_pop,
                        theme.chip_bg,
                        "Đẩy sang Live Code Editor để chỉnh sửa & lưu file",
                    )
                    .clicked()
                    {
                        *action = Some(ChatUiAction::ApplyCodeBlockToEditor(
                            block_text.to_string(),
                        ));
                    }
                });
            });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(block_text)
                    .color(theme.text_paper)
                    .monospace()
                    .size(11.5),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_markdown_strips_bold_asterisks_and_backticks() {
        let theme = CanhTheme::warm_dark();
        let job = parse_inline_markdown_job(
            "**VICTINI IDE** sẵn sàng với `~/.local/bin/agy`!",
            13.0,
            theme.text_paper,
            &theme,
        );
        assert_eq!(job.text, "VICTINI IDE sẵn sàng với ~/.local/bin/agy!");
        assert!(!job.text.contains("**"));
        assert!(!job.text.contains('`'));
        assert_eq!(job.sections.len(), 4);
    }
}
