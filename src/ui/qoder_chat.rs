use crate::ai::{
    AgentAvatarState, ChatSessionManager, ElegansAgent, ExecutionMode, MessageRole, QuotedContext,
    ThreadAgentMode, ALL_AGENTS,
};
use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use crate::ui::header::{icon_btn, project_badge_code};
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};

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
    pub collapsed_step_msgs: Vec<String>,
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
    let badge = project_badge_code(project_name);

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

    ui.vertical(|ui| {
        // 1. T3 Code Top Chat Header Breadcrumb + Dedicated Agent Icon Pill
        ui.horizontal(|ui| {
            Frame::none()
                .fill(theme.tag_red_bg)
                .rounding(Rounding::same(3.5))
                .inner_margin(Margin::symmetric(4.5, 1.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&badge)
                            .color(theme.tag_red)
                            .strong()
                            .size(9.5),
                    );
                });
            ui.label(
                egui::RichText::new(project_name)
                    .color(theme.text_silver)
                    .size(11.0),
            );
            ui.label(egui::RichText::new("/").color(theme.text_silver).size(11.0));
            ui.label(
                egui::RichText::new(&thread.title)
                    .color(theme.text_paper)
                    .strong()
                    .size(12.0),
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

                // Active Agent Mode Pill in Top Right of Chat
                let mode_color = match thread.agent_mode {
                    ThreadAgentMode::SwarmAuto => theme.accent_pop,
                    ThreadAgentMode::DedicatedCore(_) => theme.success,
                    ThreadAgentMode::DedicatedCustom { .. } => theme.info,
                };
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!(
                                "{} {}",
                                Lucide::BOT,
                                thread.agent_mode.badge_label(active_agent)
                            ))
                            .color(mode_color)
                            .strong()
                            .size(10.5),
                        )
                        .fill(theme.chip_bg)
                        .rounding(Rounding::same(5.0)),
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
        });

        ui.add_space(8.0);

        // 2. Chat Timeline Area
        let reserved_bottom =
            if composer_state.show_context_popup || composer_state.show_agent_popup {
                265.0
            } else {
                115.0
            };
        let scroll_h = (ui.available_height() - reserved_bottom).max(100.0);

        ScrollArea::vertical()
            .id_salt("t3_center_chat_scroll")
            .auto_shrink([false, false])
            .max_height(scroll_h)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for msg in &thread.messages {
                    match msg.role {
                        MessageRole::User => {
                            // User Message in rounded capsule on the right
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                Frame::none()
                                    .fill(theme.surface_2)
                                    .rounding(Rounding::same(14.0))
                                    .stroke(Stroke::new(1.0, theme.border))
                                    .inner_margin(Margin::symmetric(16.0, 9.0))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            if let Some(ref q) = msg.quoted {
                                                Frame::none()
                                                    .fill(theme.surface_1)
                                                    .rounding(Rounding::same(6.0))
                                                    .inner_margin(Margin::symmetric(8.0, 4.0))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            egui::RichText::new(format!(
                                                                "{} {}",
                                                                Lucide::QUOTE,
                                                                q.author_label
                                                            ))
                                                            .color(theme.accent_pop)
                                                            .strong()
                                                            .size(10.0),
                                                        );
                                                        let short_q: String =
                                                            q.snippet.chars().take(120).collect();
                                                        ui.label(
                                                            egui::RichText::new(short_q)
                                                                .color(theme.text_silver)
                                                                .monospace()
                                                                .size(10.0),
                                                        );
                                                    });
                                                ui.add_space(4.0);
                                            }
                                            ui.label(
                                                egui::RichText::new(&msg.content)
                                                    .color(theme.text_paper)
                                                    .size(12.2),
                                            );
                                        });
                                    });
                            });
                            ui.add_space(14.0);
                        }
                        MessageRole::Assistant => {
                            let is_collapsed =
                                composer_state.collapsed_step_msgs.contains(&msg.id);

                            ui.vertical(|ui| {
                                // Clickable Worked duration & steps toggle
                                ui.horizontal(|ui| {
                                    let chev = if is_collapsed {
                                        Lucide::CHEVRON_RIGHT
                                    } else {
                                        Lucide::CHEVRON_DOWN
                                    };
                                    let work_label = if msg.is_streaming {
                                        format!(
                                            "{} {:.1}s · {} {}",
                                            Lucide::CLOCK,
                                            msg.duration_secs,
                                            msg.steps.len(),
                                            chev
                                        )
                                    } else {
                                        format!(
                                            "{} {:.1}s · {} {}",
                                            Lucide::CLOCK,
                                            msg.duration_secs.max(0.2),
                                            msg.steps.len(),
                                            chev
                                        )
                                    };

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(work_label)
                                                    .color(theme.text_silver)
                                                    .size(10.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .on_hover_text(
                                            "Click để đóng/mở chi tiết các bước thực thi",
                                        )
                                        .clicked()
                                    {
                                        if let Some(pos) = composer_state
                                            .collapsed_step_msgs
                                            .iter()
                                            .position(|id| id == &msg.id)
                                        {
                                            composer_state.collapsed_step_msgs.remove(pos);
                                        } else {
                                            composer_state
                                                .collapsed_step_msgs
                                                .push(msg.id.clone());
                                        }
                                    }
                                });

                                ui.add_space(3.0);

                                // Real Agent Identity Header Line
                                ui.horizontal(|ui| {
                                    let colors = [theme.accent_pop, theme.info, theme.success];
                                    for c in colors {
                                        let (r, _) = ui.allocate_exact_size(
                                            vec2(7.0, 7.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().circle_filled(r.center(), 2.5, c);
                                    }

                                    ui.add_space(4.0);
                                    let agent_header_name = match &thread.agent_mode {
                                        ThreadAgentMode::DedicatedCustom { name, .. } => {
                                            format!("@{}", name)
                                        }
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
                                        .size(11.5),
                                    );
                                    ui.label(
                                        egui::RichText::new(msg.agent.title())
                                            .color(theme.info)
                                            .size(10.5),
                                    );

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            egui::RichText::new(&msg.timestamp)
                                                .color(theme.text_silver)
                                                .size(10.0),
                                        );
                                    });
                                });

                                // Collapsible Execution Steps Telemetry Box
                                if !is_collapsed && !msg.steps.is_empty() {
                                    ui.add_space(4.0);
                                    Frame::none()
                                        .fill(theme.surface_1)
                                        .rounding(Rounding::same(6.0))
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .inner_margin(Margin::symmetric(10.0, 6.0))
                                        .show(ui, |ui| {
                                            for st in &msg.steps {
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(Lucide::CHECK)
                                                            .color(theme.success)
                                                            .size(10.0),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(&st.label)
                                                            .color(theme.text_paper)
                                                            .strong()
                                                            .size(10.2),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(&st.detail)
                                                            .color(theme.text_silver)
                                                            .size(10.0),
                                                    );
                                                });
                                            }
                                        });
                                }

                                ui.add_space(8.0);

                                // Main assistant content with Fenced Code Block Rendering
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
                                            egui::RichText::new("Streaming...")
                                                .color(theme.accent_pop)
                                                .size(10.5),
                                        );
                                        if icon_btn(
                                            ui,
                                            Lucide::STOP,
                                            Color32::WHITE,
                                            theme.danger,
                                            "Dừng stream",
                                        )
                                        .clicked()
                                        {
                                            action = Some(ChatUiAction::StopStream);
                                        }
                                    });
                                }

                                // 100% Icon-First Action Bar: [Copy] [Quote] [Reply] [Kanban]
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    if icon_btn(
                                        ui,
                                        Lucide::COPY,
                                        theme.text_silver,
                                        theme.chip_bg,
                                        "Sao chép toàn bộ tin nhắn này",
                                    )
                                    .clicked()
                                    {
                                        ui.ctx().copy_text(msg.content.clone());
                                    }
                                    if icon_btn(
                                        ui,
                                        Lucide::QUOTE,
                                        theme.text_silver,
                                        theme.chip_bg,
                                        "Trích dẫn (Quote) đoạn tin nhắn này vào câu hỏi tiếp theo",
                                    )
                                    .clicked()
                                    {
                                        action = Some(ChatUiAction::SetQuote(QuotedContext {
                                            source_msg_id: msg.id.clone(),
                                            author_label: msg.agent.name().to_string(),
                                            snippet: msg.content.clone(),
                                            is_quote_block: true,
                                        }));
                                    }
                                    if icon_btn(
                                        ui,
                                        Lucide::REPLY,
                                        theme.text_silver,
                                        theme.chip_bg,
                                        "Trả lời (Reply) trực tiếp vào tin nhắn này",
                                    )
                                    .clicked()
                                    {
                                        action = Some(ChatUiAction::SetQuote(QuotedContext {
                                            source_msg_id: msg.id.clone(),
                                            author_label: msg.agent.name().to_string(),
                                            snippet: msg.content.clone(),
                                            is_quote_block: false,
                                        }));
                                    }
                                    if icon_btn(
                                        ui,
                                        Lucide::KANBAN,
                                        theme.text_silver,
                                        theme.chip_bg,
                                        "Đẩy nội dung này thành thẻ tác vụ trên Swarm Kanban",
                                    )
                                    .clicked()
                                    {
                                        action = Some(ChatUiAction::SendMessageToKanban {
                                            title: msg
                                                .content
                                                .lines()
                                                .next()
                                                .unwrap_or("Task")
                                                .chars()
                                                .take(45)
                                                .collect(),
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
                                .fill(theme.panel_bg)
                                .rounding(Rounding::same(6.0))
                                .stroke(Stroke::new(1.0, theme.border))
                                .inner_margin(Margin::symmetric(10.0, 5.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(&msg.content)
                                            .color(theme.text_silver)
                                            .size(10.5),
                                    );
                                });
                            ui.add_space(8.0);
                        }
                    }
                }
            });

        ui.add_space(4.0);

        // 3. Interactive Floating Popover: @Files / $Skills / /Commands
        if composer_state.show_context_popup {
            Frame::none()
                .fill(theme.surface_3)
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(1.0, theme.border_strong))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (t_idx, t_icon, t_tip) in [
                            (
                                0,
                                format!("{} {}", Lucide::AT_SIGN, workspace_files.len()),
                                "Đính kèm File vào ngữ cảnh (@Files)",
                            ),
                            (
                                1,
                                format!("{} {}", Lucide::WAND, skills.len()),
                                "Kích hoạt Skill ($Skills)",
                            ),
                            (
                                2,
                                format!("{} 8", Lucide::TERMINAL),
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
                                            .size(10.5),
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
                            .rounding(Rounding::same(4.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.add_sized(
                                    vec2(160.0, 18.0),
                                    egui::TextEdit::singleline(&mut composer_state.context_filter)
                                        .hint_text("Filter...")
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
                        .max_height(110.0)
                        .show(ui, |ui| match composer_state.context_tab {
                            0 => {
                                // @FILES PICKER
                                if !file_changes.is_empty() {
                                    ui.label(
                                        egui::RichText::new("CHANGED IN GIT:")
                                            .color(theme.accent_pop)
                                            .strong()
                                            .size(9.5),
                                    );
                                    for fc in file_changes {
                                        if !q.is_empty() && !fc.path.to_lowercase().contains(&q) {
                                            continue;
                                        }
                                        let attached = thread.attached_files.contains(&fc.path);
                                        ui.horizontal(|ui| {
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
                                                        .size(10.5),
                                                    )
                                                    .fill(Color32::TRANSPARENT),
                                                )
                                                .clicked()
                                            {
                                                if input_prompt.ends_with('@') {
                                                    input_prompt.pop();
                                                }
                                                action = Some(ChatUiAction::ToggleAttachFile(
                                                    fc.path.clone(),
                                                ));
                                            }
                                        });
                                    }
                                    ui.add_space(4.0);
                                }

                                ui.label(
                                    egui::RichText::new("ALL PROJECT FILES:")
                                        .color(theme.text_silver)
                                        .strong()
                                        .size(9.5),
                                );
                                for wf in workspace_files.iter().take(80) {
                                    if !q.is_empty() && !wf.to_lowercase().contains(&q) {
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
                                                .size(10.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        if input_prompt.ends_with('@') {
                                            input_prompt.pop();
                                        }
                                        action = Some(ChatUiAction::ToggleAttachFile(wf.clone()));
                                    }
                                }
                            }
                            1 => {
                                // $SKILLS PICKER
                                for sk in skills {
                                    if !q.is_empty()
                                        && !sk.name.to_lowercase().contains(&q)
                                        && !sk.description.to_lowercase().contains(&q)
                                    {
                                        continue;
                                    }
                                    let attached = thread.attached_skills.contains(&sk.name);
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
                                                .size(10.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        if input_prompt.ends_with('$') {
                                            input_prompt.pop();
                                        }
                                        action =
                                            Some(ChatUiAction::ToggleAttachSkill(sk.name.clone()));
                                    }
                                }
                            }
                            _ => {
                                // /COMMANDS PICKER (including Elegans Env Guard & AST Blast)
                                let cmds = [
                                    (
                                        "/diff",
                                        "Đính kèm toàn bộ Git Diff hiện tại và yêu cầu AI review",
                                    ),
                                    ("/commit", "Stage tất cả file & tạo Git Commit tự động"),
                                    (
                                        "/review",
                                        "Chuyển sang 1-on-1 với fe_code_auditor để kiểm tra lỗi",
                                    ),
                                    (
                                        "/env-guard",
                                        "Quét bảo mật Elegans: Tìm khóa API / Secret bị lộ trong code",
                                    ),
                                    (
                                        "/ast-blast",
                                        "Phân tích đồ thị phụ thuộc AST Blast Radius của Project",
                                    ),
                                    (
                                        "/editor",
                                        "Mở trình sửa code trực tiếp & xem Diff (Live Code Editor)",
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
                                                egui::RichText::new(format!("{} — {}", cmd, desc))
                                                    .color(theme.text_paper)
                                                    .size(10.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        composer_state.show_context_popup = false;
                                        action =
                                            Some(ChatUiAction::RunQuickSlashCommand(cmd.to_string()));
                                    }
                                }
                            }
                        });
                });
            ui.add_space(4.0);
        }

        // 4. Interactive Floating Popover: Dedicated Agent 1-on-1 & Swarm Selector
        if composer_state.show_agent_popup {
            Frame::none()
                .fill(theme.surface_3)
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(1.0, theme.border_strong))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} CHỌN AGENT (SWARM AUTO HOẶC CHAT 1-1 RIÊNG)",
                                Lucide::BOT
                            ))
                            .color(theme.accent_pop)
                            .strong()
                            .size(10.5),
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
                        .max_height(115.0)
                        .show(ui, |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} Swarm Auto-Router (Tự động chọn Agent theo câu hỏi)",
                                            Lucide::SPARKLES
                                        ))
                                        .color(
                                            if matches!(thread.agent_mode, ThreadAgentMode::SwarmAuto)
                                            {
                                                theme.accent_pop
                                            } else {
                                                theme.text_paper
                                            },
                                        )
                                        .strong()
                                        .size(10.8),
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
                            ui.label(
                                egui::RichText::new("CORE AGENTS (CHAT 1-ON-1):")
                                    .color(theme.text_silver)
                                    .strong()
                                    .size(9.5),
                            );

                            for &ag in ALL_AGENTS {
                                let is_cur =
                                    thread.agent_mode == ThreadAgentMode::DedicatedCore(ag);
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} {} — {} ({})",
                                                    ag.icon(),
                                                    ag.name(),
                                                    ag.title(),
                                                    ag.ponytail_tier()
                                                ))
                                                .color(if is_cur {
                                                    theme.success
                                                } else {
                                                    theme.text_paper
                                                })
                                                .size(10.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        composer_state.show_agent_popup = false;
                                        action = Some(ChatUiAction::OpenDedicatedCoreAgent(ag));
                                    }
                                });
                            }

                            if !custom_agents.is_empty() {
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("CUSTOM SUBAGENTS (.claude/agents/*.md):")
                                        .color(theme.info)
                                        .strong()
                                        .size(9.5),
                                );
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
                                                .size(10.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        composer_state.show_agent_popup = false;
                                        action =
                                            Some(ChatUiAction::OpenDedicatedCustomAgent(idx));
                                    }
                                }
                            }
                        });
                });
            ui.add_space(4.0);
        }

        // 5. T3 Code Floating Bottom Prompt Bar (Adapts cleanly to Dark Mode & Light Mode)
        Frame::none()
            .fill(theme.panel_bg)
            .rounding(Rounding::same(14.0))
            .stroke(Stroke::new(1.0, theme.border_strong))
            .inner_margin(Margin::symmetric(14.0, 8.0))
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
                                egui::RichText::new(format!("{} {}:", mode_ic, q.author_label))
                                    .color(theme.accent_pop)
                                    .strong()
                                    .size(10.0),
                            );
                            let short_snip: String = q
                                .snippet
                                .lines()
                                .next()
                                .unwrap_or("")
                                .chars()
                                .take(65)
                                .collect();
                            ui.label(
                                egui::RichText::new(short_snip)
                                    .color(theme.text_silver)
                                    .size(10.0),
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
                        ui.add_space(3.0);
                    }

                    // Attached @Files & $Skills Chips Row
                    if !thread.attached_files.is_empty() || !thread.attached_skills.is_empty() {
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
                                            .color(theme.success)
                                            .size(9.8),
                                        )
                                        .fill(theme.chip_bg)
                                        .rounding(Rounding::same(4.0)),
                                    )
                                    .on_hover_text("Click để gỡ file này khỏi ngữ cảnh")
                                    .clicked()
                                {
                                    action = Some(ChatUiAction::ToggleAttachFile(f_path.clone()));
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
                                            .size(9.8),
                                        )
                                        .fill(theme.chip_bg)
                                        .rounding(Rounding::same(4.0)),
                                    )
                                    .on_hover_text("Click để gỡ Skill này khỏi ngữ cảnh")
                                    .clicked()
                                {
                                    action =
                                        Some(ChatUiAction::ToggleAttachSkill(sk_name.clone()));
                                }
                            }
                        });
                        ui.add_space(4.0);
                    }

                    // Prompt Input
                    let hint = match &thread.agent_mode {
                        ThreadAgentMode::SwarmAuto => {
                            format!(
                                "Ask anything on '{}'. @ files, $ skills, / commands",
                                branch_name
                            )
                        }
                        ThreadAgentMode::DedicatedCore(ag) => {
                            format!("1-1 {} ({}). @ files, $ skills...", ag.name(), ag.title())
                        }
                        ThreadAgentMode::DedicatedCustom { name, role, .. } => {
                            format!("1-1 @{} ({}). @ files, $ skills...", name, role)
                        }
                    };

                    let text_edit = egui::TextEdit::multiline(input_prompt)
                        .desired_rows(1)
                        .desired_width(ui.available_width())
                        .frame(false)
                        .text_color(theme.text_paper)
                        .hint_text(hint);
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

                    // Lower Action Row: 100% Icon-First Controls
                    ui.horizontal(|ui| {
                        if icon_btn(
                            ui,
                            Lucide::AT_SIGN,
                            if !thread.attached_files.is_empty() {
                                theme.success
                            } else {
                                theme.text_silver
                            },
                            theme.chip_bg,
                            "Đính kèm file trong Project hoặc Git Diff (@File)",
                        )
                        .clicked()
                        {
                            composer_state.context_tab = 0;
                            composer_state.show_context_popup = !composer_state.show_context_popup;
                        }

                        if icon_btn(
                            ui,
                            Lucide::WAND,
                            if !thread.attached_skills.is_empty() {
                                theme.accent_pop
                            } else {
                                theme.text_silver
                            },
                            theme.chip_bg,
                            "Chọn Skill từ SKILL.md ($Skill)",
                        )
                        .clicked()
                        {
                            composer_state.context_tab = 1;
                            composer_state.show_context_popup = !composer_state.show_context_popup;
                        }

                        if icon_btn(
                            ui,
                            Lucide::TERMINAL,
                            theme.text_silver,
                            theme.chip_bg,
                            "Lệnh nhanh (/diff, /commit, /review, /env-guard, /ast-blast)",
                        )
                        .clicked()
                        {
                            composer_state.context_tab = 2;
                            composer_state.show_context_popup = !composer_state.show_context_popup;
                        }

                        if icon_btn(
                            ui,
                            Lucide::SHIELD,
                            theme.text_silver,
                            theme.chip_bg,
                            "Quét bảo mật Elegans (Env Guard & AST Blast Radius)",
                        )
                        .clicked()
                        {
                            action = Some(ChatUiAction::RunSecurityDiagnostics);
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // Rounded Up Arrow Send Button
                            let (btn_rect, btn_resp) =
                                ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::click());
                            ui.painter()
                                .circle_filled(btn_rect.center(), 12.0, theme.text_paper);
                            ui.painter().text(
                                btn_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                Lucide::ARROW_UP,
                                egui::FontId::proportional(13.0),
                                theme.bg_field,
                            );

                            if btn_resp.on_hover_text("Gửi yêu cầu (Enter)").clicked()
                                && !input_prompt.trim().is_empty()
                            {
                                action = Some(ChatUiAction::SendPrompt);
                            }

                            ui.add_space(6.0);

                            // Compact Agent 1-on-1 Mode Selector Pill
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
                                        .size(10.5),
                                    )
                                    .fill(theme.chip_bg)
                                    .rounding(Rounding::same(6.0)),
                                )
                                .on_hover_text("Click để chọn Agent trả lời hoặc mở Chat 1-1")
                                .clicked()
                            {
                                composer_state.show_agent_popup = !composer_state.show_agent_popup;
                            }

                            ui.add_space(4.0);

                            // Execution Mode Icon Button (CPU = Auto, SHIELD = Gatekeeper, ZAP = Bypass)
                            let (mode_ic, mode_col, mode_tip, next_mode) = match exec_mode {
                                ExecutionMode::Auto => (
                                    Lucide::CPU,
                                    theme.success,
                                    "Chế độ: Auto (Click để chuyển sang Gatekeeper)",
                                    ExecutionMode::Gatekeeper,
                                ),
                                ExecutionMode::Gatekeeper => (
                                    Lucide::SHIELD,
                                    theme.warning,
                                    "Chế độ: Gatekeeper (Click để chuyển sang Bypass)",
                                    ExecutionMode::Bypass,
                                ),
                                ExecutionMode::Bypass => (
                                    Lucide::ZAP,
                                    theme.accent_pop,
                                    "Chế độ: Bypass tốc độ tối đa (Click để chuyển sang Auto)",
                                    ExecutionMode::Auto,
                                ),
                            };
                            if icon_btn(ui, mode_ic, mode_col, theme.chip_bg, mode_tip).clicked() {
                                action = Some(ChatUiAction::SetExecutionMode(next_mode));
                            }
                        });
                    });
                });
            });
    });

    action
}

fn render_formatted_message_content(
    ui: &mut Ui,
    theme: &CanhTheme,
    content: &str,
    action: &mut Option<ChatUiAction>,
) {
    if !content.contains("```") {
        ui.label(
            egui::RichText::new(content)
                .color(theme.text_paper)
                .size(12.3),
        );
        return;
    }

    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut text_buf = String::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if !in_code_block {
                if !text_buf.trim().is_empty() {
                    ui.label(
                        egui::RichText::new(text_buf.trim_end())
                            .color(theme.text_paper)
                            .size(12.3),
                    );
                    ui.add_space(4.0);
                }
                text_buf.clear();
                in_code_block = true;
                code_lang = trimmed.trim_start_matches("```").trim().to_string();
                code_buf.clear();
            } else {
                // Render completed code block card with Icon-First Copy & Apply-to-Editor buttons
                let block_text = code_buf.trim_end().to_string();
                let lang_title = if code_lang.is_empty() {
                    "code".to_string()
                } else {
                    code_lang.clone()
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
                                    .size(10.0),
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
                                    ui.ctx().copy_text(block_text.clone());
                                }
                                if icon_btn(
                                    ui,
                                    Lucide::EDIT,
                                    theme.accent_pop,
                                    theme.chip_bg,
                                    "Đẩy đoạn code này sang Live Code Editor để chỉnh sửa & lưu file",
                                )
                                .clicked()
                                {
                                    *action = Some(ChatUiAction::ApplyCodeBlockToEditor(
                                        block_text.clone(),
                                    ));
                                }
                            });
                        });
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&block_text)
                                .color(theme.text_paper)
                                .monospace()
                                .size(11.3),
                        );
                    });
                ui.add_space(6.0);
                in_code_block = false;
                code_buf.clear();
            }
        } else if in_code_block {
            code_buf.push_str(line);
            code_buf.push('\n');
        } else {
            text_buf.push_str(line);
            text_buf.push('\n');
        }
    }

    if in_code_block && !code_buf.is_empty() {
        Frame::none()
            .fill(theme.code_bg)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(code_buf.trim_end())
                        .color(theme.text_paper)
                        .monospace()
                        .size(11.3),
                );
            });
    } else if !text_buf.trim().is_empty() {
        ui.label(
            egui::RichText::new(text_buf.trim_end())
                .color(theme.text_paper)
                .size(12.3),
        );
    }
}
