use crate::ai::{
    AgentAvatarState, ChatSessionManager, ElegansAgent, ExecutionMode, MessageRole, QuotedContext,
    ThreadAgentMode, ALL_AGENTS,
};
use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::GitFileChange;
use crate::theme::{
    CanhTheme, Lucide, CTL_DEFAULT, CTL_SM, FONT_BODY, FONT_CALLOUT, FONT_CAPTION, FONT_FOOTNOTE,
    FONT_HEADLINE, ROUND_CARD, ROUND_CTL, ROUND_XS, SPACE_1, SPACE_2, SPACE_3, SPACE_4, SPACE_6,
};
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

    ui.spacing_mut().item_spacing = vec2(SPACE_2, SPACE_2);

    // Centered Reading Column (Max 780px on 4px grid with generous whitespace)
    let total_w = ui.available_width();
    let reading_w = total_w.min(780.0);
    let side_pad = (((total_w - reading_w) * 0.5) / SPACE_1).floor() * SPACE_1;

    ui.vertical(|ui| {
        // 1. Quiet Flat Session Header (28px height, centered in reading column)
        ui.horizontal(|ui| {
            ui.add_space(side_pad.max(0.0));
            ui.allocate_ui_with_layout(
                vec2(reading_w, CTL_DEFAULT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.label(
                        egui::RichText::new(&thread.title)
                            .color(theme.text_paper)
                            .strong()
                            .size(FONT_CALLOUT),
                    );
                    ui.label(
                        egui::RichText::new(format!("· {} / {}", project_name, branch_name))
                            .color(theme.text_silver)
                            .size(FONT_FOOTNOTE),
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

                        let mode_btn = egui::Button::new(
                            egui::RichText::new(format!(
                                "{} {} {}",
                                Lucide::BOT,
                                thread.agent_mode.badge_label(active_agent),
                                Lucide::CHEVRON_DOWN
                            ))
                            .color(theme.text_paper)
                            .size(FONT_CAPTION),
                        )
                        .fill(theme.surface_1)
                        .stroke(Stroke::new(1.0, theme.border))
                        .rounding(Rounding::same(ROUND_CTL));

                        if ui
                            .add_sized(vec2(0.0, CTL_DEFAULT), mode_btn)
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

        ui.add_space(SPACE_2);

        // 2. Flat-at-Rest Chat Timeline Scroll Area
        let reserved_bottom =
            if composer_state.show_context_popup || composer_state.show_agent_popup {
                288.0
            } else {
                136.0
            };
        let scroll_h = (ui.available_height() - reserved_bottom).max(120.0);

        ScrollArea::vertical()
            .id_salt("elegans_center_chat_scroll")
            .auto_shrink([false, false])
            .max_height(scroll_h)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(side_pad.max(0.0));
                    ui.allocate_ui_with_layout(
                        vec2(reading_w, ui.available_height()),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_width(reading_w);
                            ui.add_space(SPACE_2);

                            for msg in &thread.messages {
                                match msg.role {
                                    MessageRole::User => {
                                        // User Prompt Capsule: Flat surface_2 fill, 12px ROUND_CARD, 16x12 padding
                                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                            let max_bubble_w = (reading_w * 0.76).max(240.0);
                                            Frame::none()
                                                .fill(theme.surface_2)
                                                .rounding(Rounding::same(ROUND_CARD))
                                                .stroke(Stroke::new(1.0, theme.border))
                                                .inner_margin(Margin::symmetric(SPACE_4, SPACE_3))
                                                .show(ui, |ui| {
                                                    ui.set_max_width(max_bubble_w);
                                                    ui.vertical(|ui| {
                                                        if let Some(ref q) = msg.quoted {
                                                            Frame::none()
                                                                .fill(theme.surface_1)
                                                                .rounding(Rounding::same(ROUND_CTL))
                                                                .inner_margin(Margin::symmetric(
                                                                    SPACE_2, SPACE_1,
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
                                                                        .color(theme.text_paper)
                                                                        .strong()
                                                                        .size(FONT_CAPTION),
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
                                                                        .size(FONT_CAPTION),
                                                                    );
                                                                });
                                                            ui.add_space(SPACE_1);
                                                        }
                                                        render_inline_markdown_line(
                                                            ui,
                                                            theme,
                                                            &msg.content,
                                                            FONT_BODY,
                                                            theme.text_paper,
                                                        );
                                                    });
                                                });
                                        });
                                        ui.add_space(SPACE_4);
                                    }
                                    MessageRole::Assistant => {
                                        // Elegans Flat-at-Rest Assistant Response: NO bulky outer box!
                                        // Flows directly on the Framed-Sheet with clean 4px grid spacing.
                                        let is_expanded = msg.is_streaming
                                            || composer_state
                                                .expanded_step_msgs
                                                .contains(&msg.id);

                                        ui.vertical(|ui| {
                                            ui.set_width(reading_w);

                                            // Header Line: [Agent Icon & Name] + [Step Pill] + [Time]
                                            ui.horizontal(|ui| {
                                                let agent_header_name = match &thread.agent_mode {
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
                                                    .size(FONT_FOOTNOTE),
                                                );
                                                ui.label(
                                                    egui::RichText::new(msg.agent.title())
                                                        .color(theme.text_silver)
                                                        .size(FONT_CAPTION),
                                                );

                                                if !msg.steps.is_empty() {
                                                    ui.add_space(SPACE_1);
                                                    let chev = if is_expanded {
                                                        Lucide::CHEVRON_DOWN
                                                    } else {
                                                        Lucide::CHEVRON_RIGHT
                                                    };
                                                    let step_pill_txt = if msg.is_streaming {
                                                        format!(
                                                            "{} Đang xử lý · {:.1}s ({} bước)",
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

                                                    let pill_btn = egui::Button::new(
                                                        egui::RichText::new(step_pill_txt)
                                                            .color(if msg.is_streaming {
                                                                theme.accent_pop
                                                            } else {
                                                                theme.text_silver
                                                            })
                                                            .size(FONT_CAPTION),
                                                    )
                                                    .fill(theme.surface_1)
                                                    .rounding(Rounding::same(ROUND_XS));

                                                    if ui
                                                        .add_sized(vec2(0.0, CTL_SM), pill_btn)
                                                        .on_hover_text(
                                                            "Click để đóng/mở chi tiết các bước thực thi",
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
                                                            egui::RichText::new(&msg.timestamp)
                                                                .color(theme.text_silver)
                                                                .size(FONT_CAPTION),
                                                        );
                                                    },
                                                );
                                            });

                                            // Collapsible Steps Drawer (8px ROUND_CTL, 12x8 padding)
                                            if is_expanded && !msg.steps.is_empty() {
                                                ui.add_space(SPACE_2);
                                                Frame::none()
                                                    .fill(theme.surface_1)
                                                    .rounding(Rounding::same(ROUND_CTL))
                                                    .stroke(Stroke::new(1.0, theme.border))
                                                    .inner_margin(Margin::symmetric(
                                                        SPACE_3, SPACE_2,
                                                    ))
                                                    .show(ui, |ui| {
                                                        for st in &msg.steps {
                                                            ui.horizontal_wrapped(|ui| {
                                                                ui.label(
                                                                    egui::RichText::new(
                                                                        Lucide::CHECK,
                                                                    )
                                                                    .color(theme.success)
                                                                    .size(FONT_CAPTION),
                                                                );
                                                                ui.label(
                                                                    egui::RichText::new(&st.label)
                                                                        .color(theme.text_paper)
                                                                        .strong()
                                                                        .size(FONT_CAPTION),
                                                                );
                                                                ui.label(
                                                                    egui::RichText::new(&st.detail)
                                                                        .color(theme.text_silver)
                                                                        .size(FONT_CAPTION),
                                                                );
                                                            });
                                                        }
                                                    });
                                            }

                                            ui.add_space(SPACE_2);

                                            // Main Assistant Content (13px FONT_BODY, parsed Markdown)
                                            render_formatted_message_content(
                                                ui,
                                                theme,
                                                &msg.content,
                                                &mut action,
                                            );

                                            if msg.is_streaming {
                                                ui.add_space(SPACE_2);
                                                ui.horizontal(|ui| {
                                                    let (r, _) = ui.allocate_exact_size(
                                                        vec2(SPACE_2, SPACE_2),
                                                        egui::Sense::hover(),
                                                    );
                                                    ui.painter().circle_filled(
                                                        r.center(),
                                                        3.0,
                                                        theme.accent_pop,
                                                    );
                                                    ui.label(
                                                        egui::RichText::new("Đang trả lời...")
                                                            .color(theme.accent_pop)
                                                            .size(FONT_CAPTION),
                                                    );
                                                });
                                            }

                                            // Quiet Action Row (28px icon buttons)
                                            ui.add_space(SPACE_1);
                                            ui.horizontal(|ui| {
                                                ui.spacing_mut().item_spacing =
                                                    vec2(SPACE_1, 0.0);
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
                                                    action =
                                                        Some(ChatUiAction::SendMessageToKanban {
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
                                                        });
                                                }
                                            });
                                        });
                                        ui.add_space(SPACE_4);
                                    }
                                    MessageRole::System => {
                                        Frame::none()
                                            .fill(theme.surface_1)
                                            .rounding(Rounding::same(ROUND_CTL))
                                            .stroke(Stroke::new(1.0, theme.border))
                                            .inner_margin(Margin::symmetric(SPACE_3, SPACE_2))
                                            .show(ui, |ui| {
                                                render_formatted_message_content(
                                                    ui,
                                                    theme,
                                                    &msg.content,
                                                    &mut action,
                                                );
                                            });
                                        ui.add_space(SPACE_3);
                                    }
                                }
                            }

                            // 2b. Quick-Start Cards (Sentence case heading, strict 4px grid, ROUND_CTL = 8px)
                            if thread.messages.len() <= 1 {
                                ui.add_space(SPACE_2);
                                ui.label(
                                    egui::RichText::new("Gợi ý tác vụ nhanh")
                                        .color(theme.text_silver)
                                        .strong()
                                        .size(FONT_FOOTNOTE),
                                );
                                ui.add_space(SPACE_2);

                                let card_w = ((reading_w - SPACE_2) * 0.5).max(180.0);
                                let quick_actions = [
                                    (
                                        Lucide::GIT_COMMIT,
                                        "Review Git Diff hiện tại",
                                        "Đính kèm thay đổi chưa commit và kiểm tra lỗi",
                                        "/diff",
                                    ),
                                    (
                                        Lucide::KANBAN,
                                        "Mở bảng điều phối Swarm Kanban",
                                        "Quản lý tiến độ tác vụ 4 cột & chạy Agent ngầm",
                                        "/kanban",
                                    ),
                                    (
                                        Lucide::SHIELD,
                                        "Quét bảo mật & AST Blast Radius",
                                        "Kiểm tra lộ Secret và thống kê kiến trúc dự án",
                                        "/env-guard",
                                    ),
                                    (
                                        Lucide::CODE,
                                        "Mở Live Code & Diff Editor",
                                        "So sánh Diff trực quan và chỉnh sửa trực tiếp",
                                        "/editor",
                                    ),
                                ];

                                for pair in quick_actions.chunks(2) {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing = vec2(SPACE_2, SPACE_2);
                                        for (ic, title, sub, cmd) in pair {
                                            let q_resp = Frame::none()
                                                .fill(theme.surface_1)
                                                .rounding(Rounding::same(ROUND_CTL))
                                                .stroke(Stroke::new(1.0, theme.border))
                                                .inner_margin(Margin::symmetric(SPACE_3, SPACE_2))
                                                .show(ui, |ui| {
                                                    ui.set_width((card_w - SPACE_6).max(140.0));
                                                    ui.horizontal(|ui| {
                                                        ui.label(
                                                            egui::RichText::new(*ic)
                                                                .color(theme.text_paper)
                                                                .size(FONT_CALLOUT),
                                                        );
                                                        ui.vertical(|ui| {
                                                            ui.label(
                                                                egui::RichText::new(*title)
                                                                    .color(theme.text_paper)
                                                                    .strong()
                                                                    .size(FONT_FOOTNOTE),
                                                            );
                                                            ui.label(
                                                                egui::RichText::new(*sub)
                                                                    .color(theme.text_silver)
                                                                    .size(FONT_CAPTION),
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
                                        }
                                    });
                                    ui.add_space(SPACE_2);
                                }
                            }
                        },
                    );
                });
            });

        ui.add_space(SPACE_2);

        // 3. Centered Bottom Composer & Popovers (Concentric R_outer=12px -> R_inner=8px/4px)
        ui.horizontal(|ui| {
            ui.add_space(side_pad.max(0.0));
            ui.allocate_ui_with_layout(
                vec2(reading_w, ui.available_height()),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.set_width(reading_w);

                    // 3a. Context Popover (@Files / $Skills / /Commands)
                    if composer_state.show_context_popup {
                        Frame::none()
                            .fill(theme.surface_3)
                            .rounding(Rounding::same(ROUND_CARD))
                            .stroke(Stroke::new(1.0, theme.border_strong))
                            .inner_margin(Margin::same(SPACE_3))
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
                                            format!("{} Skills ({})", Lucide::WAND, skills.len()),
                                            "Kích hoạt Skill ($Skills)",
                                        ),
                                        (
                                            2,
                                            format!("{} Lệnh nhanh", Lucide::TERMINAL),
                                            "Chạy lệnh nhanh (/Commands)",
                                        ),
                                    ] {
                                        let is_tab = composer_state.context_tab == t_idx;
                                        let tab_btn = egui::Button::new(
                                            egui::RichText::new(t_icon)
                                                .color(if is_tab {
                                                    theme.text_on_ink()
                                                } else {
                                                    theme.text_paper
                                                })
                                                .strong()
                                                .size(FONT_CAPTION),
                                        )
                                        .fill(if is_tab {
                                            theme.text_paper
                                        } else {
                                            theme.chip_bg
                                        })
                                        .rounding(Rounding::same(ROUND_XS));

                                        if ui
                                            .add_sized(vec2(0.0, CTL_SM), tab_btn)
                                            .on_hover_text(t_tip)
                                            .clicked()
                                        {
                                            composer_state.context_tab = t_idx;
                                        }
                                    }

                                    ui.add_space(SPACE_2);
                                    Frame::none()
                                        .fill(theme.input_bg)
                                        .rounding(Rounding::same(ROUND_XS))
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                vec2(180.0, 16.0),
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

                                ui.add_space(SPACE_2);
                                let q = composer_state.context_filter.trim().to_lowercase();

                                ScrollArea::vertical()
                                    .id_salt("chat_context_popover_scroll")
                                    .max_height(120.0)
                                    .show(ui, |ui| match composer_state.context_tab {
                                        0 => {
                                            if !file_changes.is_empty() {
                                                ui.label(
                                                    egui::RichText::new(
                                                        "File đang thay đổi trong Git",
                                                    )
                                                    .color(theme.text_silver)
                                                    .strong()
                                                    .size(FONT_CAPTION),
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
                                                                .size(FONT_CAPTION),
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
                                                ui.add_space(SPACE_1);
                                            }

                                            ui.label(
                                                egui::RichText::new("Tất cả file trong dự án")
                                                    .color(theme.text_silver)
                                                    .strong()
                                                    .size(FONT_CAPTION),
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
                                                            .size(FONT_CAPTION),
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
                                                            .size(FONT_CAPTION),
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
                                                    "Đính kèm Git Diff hiện tại và yêu cầu AI review",
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
                                                            .size(FONT_CAPTION),
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
                        ui.add_space(SPACE_1);
                    }

                    // 3b. Dedicated Agent 1-on-1 & Swarm Selector Popover
                    if composer_state.show_agent_popup {
                        Frame::none()
                            .fill(theme.surface_3)
                            .rounding(Rounding::same(ROUND_CARD))
                            .stroke(Stroke::new(1.0, theme.border_strong))
                            .inner_margin(Margin::same(SPACE_3))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} Chọn chế độ điều phối hoặc Chat 1-1",
                                            Lucide::BOT
                                        ))
                                        .color(theme.text_paper)
                                        .strong()
                                        .size(FONT_FOOTNOTE),
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

                                ui.add_space(SPACE_2);
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
                                                    .size(FONT_FOOTNOTE),
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

                                        ui.add_space(SPACE_1);
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
                                                        .size(FONT_CAPTION),
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
                                            ui.add_space(SPACE_1);
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
                                                            .size(FONT_CAPTION),
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
                        ui.add_space(SPACE_1);
                    }

                    // 3c. Elegans Concentric Composer Card (Outer ROUND_CARD = 12px, Padding = 12px, Inner Controls = ROUND_CTL 8px / ROUND_XS 4px)
                    let border_col = if !input_prompt.trim().is_empty() {
                        theme.border_strong
                    } else {
                        theme.border
                    };
                    Frame::none()
                        .fill(theme.surface_1)
                        .rounding(Rounding::same(ROUND_CARD))
                        .stroke(Stroke::new(1.0, border_col))
                        .inner_margin(Margin::same(SPACE_3))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
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
                                            .color(theme.text_paper)
                                            .strong()
                                            .size(FONT_CAPTION),
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
                                                .size(FONT_CAPTION),
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
                                    ui.add_space(SPACE_1);
                                }

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
                                                        .size(FONT_CAPTION),
                                                    )
                                                    .fill(theme.chip_bg)
                                                    .rounding(Rounding::same(ROUND_XS)),
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
                                                        .size(FONT_CAPTION),
                                                    )
                                                    .fill(theme.chip_bg)
                                                    .rounding(Rounding::same(ROUND_XS)),
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
                                    ui.add_space(SPACE_1);
                                }

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

                                ui.add_space(SPACE_2);

                                // Bottom Composer Controls (24px CTL_SM pills + 28px Ink Primary Send CTA)
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing = vec2(SPACE_1, 0.0);

                                    let file_lbl = if thread.attached_files.is_empty() {
                                        format!("{} @File", Lucide::AT_SIGN)
                                    } else {
                                        format!(
                                            "{} @File · {}",
                                            Lucide::AT_SIGN,
                                            thread.attached_files.len()
                                        )
                                    };
                                    if ui
                                        .add_sized(
                                            vec2(0.0, CTL_SM),
                                            egui::Button::new(
                                                egui::RichText::new(file_lbl)
                                                    .color(if !thread.attached_files.is_empty() {
                                                        theme.text_paper
                                                    } else {
                                                        theme.text_silver
                                                    })
                                                    .size(FONT_CAPTION),
                                            )
                                            .fill(theme.chip_bg)
                                            .rounding(Rounding::same(ROUND_XS)),
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
                                            "{} $Skill · {}",
                                            Lucide::WAND,
                                            thread.attached_skills.len()
                                        )
                                    };
                                    if ui
                                        .add_sized(
                                            vec2(0.0, CTL_SM),
                                            egui::Button::new(
                                                egui::RichText::new(skill_lbl)
                                                    .color(if !thread.attached_skills.is_empty() {
                                                        theme.text_paper
                                                    } else {
                                                        theme.text_silver
                                                    })
                                                    .size(FONT_CAPTION),
                                            )
                                            .fill(theme.chip_bg)
                                            .rounding(Rounding::same(ROUND_XS)),
                                        )
                                        .on_hover_text("Kích hoạt Skill (SKILL.md)")
                                        .clicked()
                                    {
                                        composer_state.context_tab = 1;
                                        composer_state.show_context_popup =
                                            !composer_state.show_context_popup;
                                    }

                                    if ui
                                        .add_sized(
                                            vec2(0.0, CTL_SM),
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} /Lệnh",
                                                    Lucide::TERMINAL
                                                ))
                                                .color(theme.text_silver)
                                                .size(FONT_CAPTION),
                                            )
                                            .fill(theme.chip_bg)
                                            .rounding(Rounding::same(ROUND_XS)),
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
                                        ui.spacing_mut().item_spacing = vec2(SPACE_1, 0.0);

                                        // Elegans Primary CTA: Ink fill (`theme.text_paper`) with `theme.text_on_ink()` icon
                                        let can_send = !input_prompt.trim().is_empty();
                                        let send_bg = if can_send {
                                            theme.text_paper
                                        } else {
                                            theme.chip_bg
                                        };
                                        let send_fg = if can_send {
                                            theme.text_on_ink()
                                        } else {
                                            theme.text_silver
                                        };
                                        if ui
                                            .add_sized(
                                                vec2(CTL_DEFAULT, CTL_SM),
                                                egui::Button::new(
                                                    egui::RichText::new(Lucide::ARROW_UP)
                                                        .color(send_fg)
                                                        .strong()
                                                        .size(FONT_FOOTNOTE),
                                                )
                                                .fill(send_bg)
                                                .rounding(Rounding::same(ROUND_XS)),
                                            )
                                            .on_hover_text("Gửi yêu cầu (Enter)")
                                            .clicked()
                                            && can_send
                                        {
                                            action = Some(ChatUiAction::SendPrompt);
                                        }

                                        // Agent Mode Selector Pill
                                        if ui
                                            .add_sized(
                                                vec2(0.0, CTL_SM),
                                                egui::Button::new(
                                                    egui::RichText::new(format!(
                                                        "{} {} {}",
                                                        Lucide::SPARKLES,
                                                        thread.agent_mode.short_tag(),
                                                        Lucide::CHEVRON_DOWN
                                                    ))
                                                    .color(theme.text_paper)
                                                    .size(FONT_CAPTION),
                                                )
                                                .fill(theme.chip_bg)
                                                .rounding(Rounding::same(ROUND_XS)),
                                            )
                                            .on_hover_text("Chọn Agent hoặc mở Chat 1-1")
                                            .clicked()
                                        {
                                            composer_state.show_agent_popup =
                                                !composer_state.show_agent_popup;
                                        }

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
                                            .add_sized(
                                                vec2(0.0, CTL_SM),
                                                egui::Button::new(
                                                    egui::RichText::new(format!(
                                                        "{} {}",
                                                        mode_ic, mode_lbl
                                                    ))
                                                    .color(theme.text_silver)
                                                    .size(FONT_CAPTION),
                                                )
                                                .fill(theme.chip_bg)
                                                .rounding(Rounding::same(ROUND_XS)),
                                            )
                                            .on_hover_text(
                                                "Đổi chế độ thực thi (Auto / Gatekeeper / Fast)",
                                            )
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
                        font_id: FontId::new(font_size, FontFamily::Proportional),
                        color: theme.text_paper,
                        ..Default::default()
                    },
                );
                i = end + 2;
                continue;
            }
        }

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
                        font_id: FontId::new(FONT_FOOTNOTE, FontFamily::Monospace),
                        color: theme.text_paper,
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
                ui.add_space(SPACE_2);
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
            ui.add_space(SPACE_1);
            continue;
        }

        if let Some(h3) = trimmed.strip_prefix("### ") {
            ui.add_space(SPACE_1);
            ui.label(
                egui::RichText::new(h3.replace("**", ""))
                    .color(theme.text_paper)
                    .strong()
                    .size(FONT_CALLOUT),
            );
            ui.add_space(SPACE_1);
        } else if let Some(h2) = trimmed.strip_prefix("## ") {
            ui.add_space(SPACE_2);
            ui.label(
                egui::RichText::new(h2.replace("**", ""))
                    .color(theme.text_paper)
                    .strong()
                    .size(FONT_HEADLINE),
            );
            ui.add_space(SPACE_1);
        } else if let Some(h1) = trimmed.strip_prefix("# ") {
            ui.add_space(SPACE_2);
            ui.label(
                egui::RichText::new(h1.replace("**", ""))
                    .color(theme.text_paper)
                    .strong()
                    .size(FONT_HEADLINE),
            );
            ui.add_space(SPACE_1);
        } else if let Some(bullet) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("• "))
        {
            ui.horizontal_top(|ui| {
                ui.add_space(SPACE_2);
                ui.label(
                    egui::RichText::new("•")
                        .color(theme.text_silver)
                        .strong()
                        .size(FONT_BODY),
                );
                render_inline_markdown_line(ui, theme, bullet, FONT_BODY, theme.text_paper);
            });
            ui.add_space(SPACE_1);
        } else {
            render_inline_markdown_line(ui, theme, line, FONT_BODY, theme.text_paper);
            ui.add_space(SPACE_1);
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
        .rounding(Rounding::same(ROUND_CTL))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::same(SPACE_3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} {}", Lucide::CODE, lang_title))
                        .color(theme.text_silver)
                        .monospace()
                        .strong()
                        .size(FONT_CAPTION),
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
                        theme.text_paper,
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
            ui.add_space(SPACE_1);
            ui.label(
                egui::RichText::new(block_text)
                    .color(theme.text_paper)
                    .monospace()
                    .size(FONT_FOOTNOTE),
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
            FONT_BODY,
            theme.text_paper,
            &theme,
        );
        assert_eq!(job.text, "VICTINI IDE sẵn sàng với ~/.local/bin/agy!");
        assert!(!job.text.contains("**"));
        assert!(!job.text.contains('`'));
        assert_eq!(job.sections.len(), 4);
    }
}
