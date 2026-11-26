use crate::ai::{
    AgentAvatarState, ChatSessionManager, ElegansAgent, ExecutionMode, MessageRole, QuotedContext,
    ThreadAgentMode, ALL_AGENTS,
};
use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::GitFileChange;
use crate::theme::{CanhTheme, Lucide};
use crate::ui::header::project_badge_code;
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
    RunQuickSlashCommand(String),
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
        // 1. T3 Code Top Chat Header Breadcrumb + Dedicated Agent Banner
        ui.horizontal(|ui| {
            Frame::none()
                .fill(Color32::from_rgb(0x28, 0x0E, 0x10))
                .rounding(Rounding::same(3.0))
                .inner_margin(Margin::symmetric(4.0, 1.0))
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
                    && ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!("{} Stop", Lucide::STOP))
                                    .color(theme.text_paper)
                                    .size(10.5),
                            )
                            .fill(Color32::from_rgb(0x38, 0x12, 0x14))
                            .rounding(Rounding::same(4.0)),
                        )
                        .clicked()
                {
                    action = Some(ChatUiAction::StopStream);
                }

                // Active Agent Mode Pill in Top Right of Chat
                let mode_color = match thread.agent_mode {
                    ThreadAgentMode::SwarmAuto => theme.accent_pop,
                    ThreadAgentMode::DedicatedCore(_) => Color32::from_rgb(0x4E, 0xC9, 0xB0),
                    ThreadAgentMode::DedicatedCustom { .. } => Color32::from_rgb(0x51, 0xA2, 0xFF),
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
                        .fill(Color32::from_rgb(0x14, 0x14, 0x14))
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
        let reserved_bottom = if composer_state.show_context_popup || composer_state.show_agent_popup
        {
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
                            // User Message in dark rounded capsule on the right
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                Frame::none()
                                    .fill(Color32::from_rgb(0x16, 0x16, 0x16))
                                    .rounding(Rounding::same(14.0))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(0x24, 0x24, 0x24)))
                                    .inner_margin(Margin::symmetric(16.0, 9.0))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            if let Some(ref q) = msg.quoted {
                                                Frame::none()
                                                    .fill(Color32::from_rgb(0x10, 0x10, 0x10))
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
                            // T3 Code Assistant Message Layout with Real Telemetry & Collapsible Steps
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
                                            "Working ({:.1}s) · {} steps {}",
                                            msg.duration_secs,
                                            msg.steps.len(),
                                            chev
                                        )
                                    } else {
                                        format!(
                                            "Worked for {:.1}s · {} steps {}",
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
                                        .on_hover_text("Click để đóng/mở chi tiết các bước thực thi")
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

                                ui.add_space(4.0);

                                // Real Agent Identity Header Line
                                ui.horizontal(|ui| {
                                    let colors = [
                                        theme.accent_pop,
                                        Color32::from_rgb(0x51, 0xA2, 0xFF),
                                        Color32::from_rgb(0x4E, 0xC9, 0xB0),
                                    ];
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
                                            .color(Color32::from_rgb(0x51, 0xA2, 0xFF))
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
                                        .fill(Color32::from_rgb(0x0E, 0x0E, 0x0E))
                                        .rounding(Rounding::same(6.0))
                                        .stroke(Stroke::new(
                                            1.0,
                                            Color32::from_rgb(0x1A, 0x1A, 0x1A),
                                        ))
                                        .inner_margin(Margin::symmetric(10.0, 6.0))
                                        .show(ui, |ui| {
                                            for st in &msg.steps {
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(Lucide::CHECK)
                                                            .color(Color32::from_rgb(
                                                                0x4E, 0xC9, 0xB0,
                                                            ))
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
                                            egui::RichText::new(
                                                "AGY Subagent đang xử lý & stream kết quả...",
                                            )
                                            .color(theme.accent_pop)
                                            .size(10.5),
                                        );
                                        if ui.small_button("Stop").clicked() {
                                            action = Some(ChatUiAction::StopStream);
                                        }
                                    });
                                }

                                // Action buttons (Copy, Quote, Reply, +Kanban)
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Copy",
                                                    Lucide::COPY
                                                ))
                                                .color(theme.text_silver)
                                                .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x16, 0x16, 0x16)),
                                        )
                                        .clicked()
                                    {
                                        ui.ctx().copy_text(msg.content.clone());
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Quote",
                                                    Lucide::QUOTE
                                                ))
                                                .color(theme.text_silver)
                                                .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x16, 0x16, 0x16)),
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
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Reply",
                                                    Lucide::MESSAGE_SQUARE
                                                ))
                                                .color(theme.text_silver)
                                                .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x16, 0x16, 0x16)),
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
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} +Kanban",
                                                    Lucide::KANBAN
                                                ))
                                                .color(theme.text_silver)
                                                .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x16, 0x16, 0x16)),
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
                                .fill(Color32::from_rgb(0x11, 0x11, 0x11))
                                .rounding(Rounding::same(6.0))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
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
                .fill(Color32::from_rgb(0x13, 0x13, 0x13))
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(0x28, 0x28, 0x28)))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (t_idx, t_label) in [
                            (0, format!("{} @Files ({})", Lucide::FILE_TEXT, workspace_files.len())),
                            (1, format!("{} $Skills ({})", Lucide::CODE, skills.len())),
                            (2, format!("{} /Commands (6)", Lucide::TERMINAL)),
                        ] {
                            let is_tab = composer_state.context_tab == t_idx;
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(t_label)
                                            .color(if is_tab {
                                                Color32::BLACK
                                            } else {
                                                theme.text_paper
                                            })
                                            .strong()
                                            .size(10.5),
                                    )
                                    .fill(if is_tab {
                                        theme.accent_pop
                                    } else {
                                        Color32::from_rgb(0x1D, 0x1D, 0x1D)
                                    })
                                    .rounding(Rounding::same(5.0)),
                                )
                                .clicked()
                            {
                                composer_state.context_tab = t_idx;
                            }
                        }

                        ui.add_space(6.0);
                        Frame::none()
                            .fill(Color32::from_rgb(0x0A, 0x0A, 0x0A))
                            .rounding(Rounding::same(4.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.add_sized(
                                    vec2(150.0, 18.0),
                                    egui::TextEdit::singleline(&mut composer_state.context_filter)
                                        .hint_text("Filter...")
                                        .frame(false)
                                        .text_color(theme.text_paper),
                                );
                            });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(Lucide::X)
                                            .color(theme.text_silver)
                                            .size(11.0),
                                    )
                                    .fill(Color32::TRANSPARENT),
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
                                // @FILES PICKER (Changed files first, then workspace files)
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
                                                            Color32::from_rgb(0x4E, 0xC9, 0xB0)
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
                                                    Color32::from_rgb(0x4E, 0xC9, 0xB0)
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
                                                        Lucide::CODE
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
                                // /COMMANDS PICKER
                                let cmds = [
                                    ("/diff", "Đính kèm toàn bộ Git Diff hiện tại và yêu cầu AI review chi tiết"),
                                    ("/commit", "Stage tất cả file & tạo Git Commit tự động"),
                                    ("/review", "Chuyển sang 1-on-1 với fe_code_auditor để kiểm tra lỗi & bảo mật"),
                                    ("/editor", "Mở trình sửa code trực tiếp & xem Diff (Live Code Editor)"),
                                    ("/kanban", "Mở bảng điều phối Swarm Kanban Board"),
                                    ("/clear", "Làm mới phiên chat hiện tại"),
                                ];
                                for (cmd, desc) in cmds {
                                    if !q.is_empty() && !cmd.contains(&q) && !desc.to_lowercase().contains(&q) {
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
                .fill(Color32::from_rgb(0x13, 0x13, 0x13))
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(0x28, 0x28, 0x28)))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} CHỌN CHẾ ĐỘ CHAT: SWARM AUTO HOẶC CHAT 1-1 VỚI AGENT RIÊNG",
                                Lucide::BOT
                            ))
                            .color(theme.accent_pop)
                            .strong()
                            .size(10.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(Lucide::X)
                                            .color(theme.text_silver)
                                            .size(11.0),
                                    )
                                    .fill(Color32::TRANSPARENT),
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
                            // Option 1: Swarm Auto-Router
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
                                egui::RichText::new("CORE AGENTS (MỞ HOẶC KHOÁ CHAT 1-ON-1):")
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
                                                    Color32::from_rgb(0x4E, 0xC9, 0xB0)
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
                                        .color(Color32::from_rgb(0x51, 0xA2, 0xFF))
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

        // 5. T3 Code Floating Bottom Prompt Bar (Dark OLED Pill #111111)
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
            .rounding(Rounding::same(14.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
            .inner_margin(Margin::symmetric(14.0, 8.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    // Active Quote / Reply Banner
                    if let Some(ref q) = chat_mgr.active_quote {
                        ui.horizontal(|ui| {
                            let mode_lbl = if q.is_quote_block { "Quote" } else { "Reply" };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} {} ({}):",
                                    Lucide::QUOTE,
                                    mode_lbl,
                                    q.author_label
                                ))
                                .color(theme.accent_pop)
                                .strong()
                                .size(10.0),
                            );
                            let short_snip: String =
                                q.snippet.lines().next().unwrap_or("").chars().take(65).collect();
                            ui.label(
                                egui::RichText::new(short_snip)
                                    .color(theme.text_silver)
                                    .size(10.0),
                            );
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(Lucide::X)
                                            .color(theme.text_silver)
                                            .size(10.0),
                                    )
                                    .fill(Color32::TRANSPARENT),
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
                                            .color(Color32::from_rgb(0x4E, 0xC9, 0xB0))
                                            .size(9.8),
                                        )
                                        .fill(Color32::from_rgb(0x16, 0x22, 0x1E))
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
                                                Lucide::CODE,
                                                sk_name,
                                                Lucide::X
                                            ))
                                            .color(theme.accent_pop)
                                            .size(9.8),
                                        )
                                        .fill(Color32::from_rgb(0x24, 0x16, 0x12))
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

                    // Prompt Input - Frame false to prevent white box!
                    let hint = match &thread.agent_mode {
                        ThreadAgentMode::SwarmAuto => {
                            format!(
                                "Ask anything on branch '{}'. @ files, $ skills, / commands",
                                branch_name
                            )
                        }
                        ThreadAgentMode::DedicatedCore(ag) => {
                            format!(
                                "Chat 1-1 với {} ({}). Gõ @ files, $ skills, / commands...",
                                ag.name(),
                                ag.title()
                            )
                        }
                        ThreadAgentMode::DedicatedCustom { name, role, .. } => {
                            format!("Chat 1-1 với @{} ({}). Gõ @ files, $ skills...", name, role)
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

                    // Lower Action Row: Left = [+ @File] [$Skill] [/Cmd] | Right = [ExecMode] [Agent Selector] [Send ^]
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} @File", Lucide::PLUS))
                                        .color(theme.text_silver)
                                        .size(10.5),
                                )
                                .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                .rounding(Rounding::same(5.0)),
                            )
                            .on_hover_text("Đính kèm file trong Project hoặc Git Diff (@)")
                            .clicked()
                        {
                            composer_state.context_tab = 0;
                            composer_state.show_context_popup = !composer_state.show_context_popup;
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} $Skill", Lucide::CODE))
                                        .color(theme.text_silver)
                                        .size(10.5),
                                )
                                .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                .rounding(Rounding::same(5.0)),
                            )
                            .on_hover_text("Chọn Skill từ SKILL.md ($)")
                            .clicked()
                        {
                            composer_state.context_tab = 1;
                            composer_state.show_context_popup = !composer_state.show_context_popup;
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} /Cmd", Lucide::TERMINAL))
                                        .color(theme.text_silver)
                                        .size(10.5),
                                )
                                .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                .rounding(Rounding::same(5.0)),
                            )
                            .on_hover_text("Chạy lệnh nhanh (/diff, /commit, /review, /editor)")
                            .clicked()
                        {
                            composer_state.context_tab = 2;
                            composer_state.show_context_popup = !composer_state.show_context_popup;
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            // Rounded Up Arrow Send Button (Crisp White Circle with Lucide::ARROW_UP)
                            let (btn_rect, btn_resp) =
                                ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::click());
                            ui.painter()
                                .circle_filled(btn_rect.center(), 12.0, theme.text_paper);
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

                            ui.add_space(6.0);

                            // Interactive Agent & 1-on-1 Mode Selector Pill
                            let agent_pill_label = thread.agent_mode.badge_label(active_agent);
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} {} {}",
                                            Lucide::SPARKLES,
                                            agent_pill_label,
                                            Lucide::CHEVRON_DOWN
                                        ))
                                        .color(theme.text_paper)
                                        .size(10.5),
                                    )
                                    .fill(Color32::from_rgb(0x19, 0x19, 0x19))
                                    .rounding(Rounding::same(6.0)),
                                )
                                .on_hover_text("Click để chọn Agent trả lời hoặc mở Chat 1-1")
                                .clicked()
                            {
                                composer_state.show_agent_popup = !composer_state.show_agent_popup;
                            }

                            ui.add_space(4.0);

                            // Execution Mode Toggle Pill (Auto / Gatekeeper / Bypass)
                            let (mode_str, mode_col, next_mode) = match exec_mode {
                                ExecutionMode::Auto => (
                                    "Auto",
                                    Color32::from_rgb(0x4E, 0xC9, 0xB0),
                                    ExecutionMode::Gatekeeper,
                                ),
                                ExecutionMode::Gatekeeper => (
                                    "Gatekeeper",
                                    Color32::from_rgb(0xE5, 0xC0, 0x7B),
                                    ExecutionMode::Bypass,
                                ),
                                ExecutionMode::Bypass => {
                                    ("Bypass", theme.accent_pop, ExecutionMode::Auto)
                                }
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(mode_str).color(mode_col).size(10.0),
                                    )
                                    .fill(Color32::from_rgb(0x16, 0x16, 0x16))
                                    .rounding(Rounding::same(5.0)),
                                )
                                .on_hover_text("Click để đổi chế độ thực thi (Auto -> Gatekeeper -> Bypass)")
                                .clicked()
                            {
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
                // Render completed code block card
                let block_text = code_buf.trim_end().to_string();
                let lang_title = if code_lang.is_empty() {
                    "code".to_string()
                } else {
                    code_lang.clone()
                };
                Frame::none()
                    .fill(Color32::from_rgb(0x08, 0x08, 0x08))
                    .rounding(Rounding::same(8.0))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
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
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(format!(
                                                "{} Copy Code",
                                                Lucide::COPY
                                            ))
                                            .color(theme.text_silver)
                                            .size(9.8),
                                        )
                                        .fill(Color32::from_rgb(0x16, 0x16, 0x16)),
                                    )
                                    .clicked()
                                {
                                    ui.ctx().copy_text(block_text.clone());
                                }
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("Open Editor")
                                                .color(theme.text_silver)
                                                .size(9.8),
                                        )
                                        .fill(Color32::from_rgb(0x16, 0x16, 0x16)),
                                    )
                                    .clicked()
                                {
                                    *action = Some(ChatUiAction::OpenFileInEditor {
                                        path: String::new(),
                                        edit_mode: true,
                                    });
                                }
                            });
                        });
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&block_text)
                                .color(Color32::from_rgb(0xE6, 0xE6, 0xE6))
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
            .fill(Color32::from_rgb(0x08, 0x08, 0x08))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(code_buf.trim_end())
                        .color(Color32::from_rgb(0xE6, 0xE6, 0xE6))
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
