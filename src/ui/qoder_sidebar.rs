use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::{GitFileChange, WorktreeInfo};
use crate::theme::CanhTheme;
use crate::ui::layout_state::CenterViewMode;
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};

pub enum SidebarAction {
    ToggleMiniMode,
    NewChatThread,
    SelectChatThread(usize),
    SwitchCenterView(CenterViewMode),
    SelectWorktree(String),
    CreateNewBranch(String),
    StageFile(String),
    UnstageFile(String),
    DiscardFile(String),
    StageAll,
    ViewFileDiff(String),
    CommitWithMessage(String),
    AiGenerateCommitMsg,
    GitPush,
    GitPull,
    CreateGitLabMr,
    OpenSkillStudioNewSkill,
    OpenSkillStudioNewAgent,
    InspectSkill(usize),
    InspectAgent(usize),
}

#[allow(clippy::too_many_arguments)]
pub fn render_qoder_sidebar(
    ui: &mut Ui,
    theme: &CanhTheme,
    is_mini: bool,
    active_mode: CenterViewMode,
    search_query: &mut String,
    worktrees: &[WorktreeInfo],
    branches: &[String],
    new_branch_input: &mut String,
    file_changes: &[GitFileChange],
    commit_message: &mut String,
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
    thread_titles: &[(usize, String, String)],
    active_thread_idx: usize,
) -> Option<SidebarAction> {
    let mut action = None;

    // MINI RAIL MODE (54px Compact Responsive Mode)
    if is_mini {
        ui.vertical_centered(|ui| {
            ui.add_space(4.0);
            if ui
                .button(egui::RichText::new("⚡").color(theme.accent_pop).size(16.0))
                .on_hover_text("Mở rộng Sidebar")
                .clicked()
            {
                action = Some(SidebarAction::ToggleMiniMode);
            }
            ui.add_space(8.0);
            if ui
                .button(egui::RichText::new("＋").color(theme.text_paper).size(15.0))
                .on_hover_text("Tạo phiên Chat mới")
                .clicked()
            {
                action = Some(SidebarAction::NewChatThread);
            }
            ui.add_space(6.0);
            if ui
                .button(egui::RichText::new("💬").size(14.0))
                .on_hover_text("Chat Timeline")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::ChatTimeline));
            }
            ui.add_space(6.0);
            if ui
                .button(egui::RichText::new("📊").size(14.0))
                .on_hover_text("Swarm Kanban Board")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::KanbanBoard));
            }
            ui.add_space(6.0);
            if ui
                .button(egui::RichText::new("📁").size(14.0))
                .on_hover_text("Quản lý Projects")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::ProjectsModal));
            }
            ui.add_space(6.0);
            if ui
                .button(egui::RichText::new("🤖").size(14.0))
                .on_hover_text("Agent & Skill Studio")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::AgentSkillStudio));
            }
            ui.add_space(6.0);
            if ui
                .button(egui::RichText::new("⚙").size(14.0))
                .on_hover_text("Settings")
                .clicked()
            {
                action = Some(SidebarAction::SwitchCenterView(CenterViewMode::SettingsModal));
            }
        });
        return action;
    }

    // FULL RESPONSIVE SIDEBAR MODE
    ScrollArea::vertical()
        .id_salt("victini_left_sidebar_main_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. Top Mode Pill (Qoder style: [⚡ Coding] + Collapse Button)
            Frame::none()
                .fill(Color32::from_rgb(0x13, 0x16, 0x20))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x28, 0x38)))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("⚡ Coding Studio")
                                .color(theme.accent_pop)
                                .strong()
                                .size(12.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .button(egui::RichText::new("«").color(theme.text_silver).size(11.0))
                                .on_hover_text("Thu gọn thành Mini-Rail")
                                .clicked()
                            {
                                action = Some(SidebarAction::ToggleMiniMode);
                            }
                        });
                    });
                });

            ui.add_space(6.0);

            // 2. Quick Search Filter
            ui.add_sized(
                vec2(ui.available_width(), 24.0),
                egui::TextEdit::singleline(search_query)
                    .hint_text("🔍 Filter files, skills, chats..."),
            );
            let q_lower = search_query.trim().to_lowercase();

            ui.add_space(6.0);

            // 3. Navigation Actions
            let nav_items = [
                ("＋ New Vibe Task", CenterViewMode::ChatTimeline, true),
                ("📁 Projects & Workspaces", CenterViewMode::ProjectsModal, false),
                ("📊 Swarm Kanban Board", CenterViewMode::KanbanBoard, false),
                ("🤖 Agent & Skill Studio", CenterViewMode::AgentSkillStudio, false),
            ];

            for (label, target_mode, is_new_btn) in nav_items {
                let is_selected = !is_new_btn && active_mode == target_mode;
                let bg = if is_selected {
                    Color32::from_rgb(0x18, 0x1C, 0x26)
                } else {
                    Color32::TRANSPARENT
                };

                let resp = Frame::none()
                    .fill(bg)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::symmetric(8.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(label)
                                    .color(if is_new_btn {
                                        theme.accent_pop
                                    } else if is_selected {
                                        theme.text_paper
                                    } else {
                                        theme.text_silver
                                    })
                                    .strong()
                                    .size(11.5),
                            );
                        });
                    });

                if resp.response.interact(egui::Sense::click()).clicked() {
                    if is_new_btn {
                        action = Some(SidebarAction::NewChatThread);
                    } else {
                        action = Some(SidebarAction::SwitchCenterView(target_mode));
                    }
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // 4. WORKTREES & BRANCHES
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("WORKTREES & BRANCHES ({})", branches.len()))
                        .color(theme.text_silver)
                        .strong()
                        .size(10.0),
                );
            });
            ui.add_space(3.0);

            for wt in worktrees {
                if !q_lower.is_empty() && !wt.name.to_lowercase().contains(&q_lower) {
                    continue;
                }
                let is_head = wt.is_head;
                let bg = if is_head {
                    Color32::from_rgb(0x16, 0x1A, 0x24)
                } else {
                    Color32::TRANSPARENT
                };
                let border = if is_head {
                    Stroke::new(1.0, theme.accent_pop)
                } else {
                    Stroke::NONE
                };

                let resp = Frame::none()
                    .fill(bg)
                    .rounding(Rounding::same(6.0))
                    .stroke(border)
                    .inner_margin(Margin::symmetric(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let dot_col = if is_head {
                                theme.accent_pop
                            } else {
                                theme.text_silver
                            };
                            ui.label(egui::RichText::new("●").color(dot_col).size(8.0));
                            ui.label(
                                egui::RichText::new(&wt.name)
                                    .color(if is_head {
                                        theme.text_paper
                                    } else {
                                        theme.text_silver
                                    })
                                    .size(11.0),
                            );
                        });
                    });

                if resp.response.interact(egui::Sense::click()).clicked() {
                    action = Some(SidebarAction::SelectWorktree(wt.name.clone()));
                }
            }

            // Quick create branch box
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                let w = (ui.available_width() - 48.0).max(60.0);
                ui.add_sized(
                    vec2(w, 20.0),
                    egui::TextEdit::singleline(new_branch_input).hint_text("feat/new-branch..."),
                );
                if ui
                    .button(egui::RichText::new("+ Branch").size(9.5))
                    .clicked()
                    && !new_branch_input.trim().is_empty()
                {
                    action = Some(SidebarAction::CreateNewBranch(
                        new_branch_input.trim().to_string(),
                    ));
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // 5. GIT SOURCE CONTROL (File Changes, Stage/Unstage/Diff, Commit, Push, Pull, MR)
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("GIT CHANGES ({})", file_changes.len()))
                        .color(theme.text_silver)
                        .strong()
                        .size(10.0),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(egui::RichText::new("+ Stage All").size(9.5))
                        .clicked()
                    {
                        action = Some(SidebarAction::StageAll);
                    }
                });
            });

            ui.add_space(4.0);
            if file_changes.is_empty() {
                ui.label(
                    egui::RichText::new("✓ Working tree clean")
                        .color(Color32::from_rgb(0x4E, 0xC9, 0xB0))
                        .size(10.5),
                );
            } else {
                ScrollArea::vertical()
                    .id_salt("git_changes_sidebar_scroll")
                    .max_height(125.0)
                    .show(ui, |ui| {
                        for fc in file_changes {
                            if !q_lower.is_empty() && !fc.path.to_lowercase().contains(&q_lower) {
                                continue;
                            }
                            let status_col = match fc.status.as_str() {
                                "A" => Color32::from_rgb(0x98, 0xC3, 0x79),
                                "M" => Color32::from_rgb(0xE5, 0xC0, 0x7B),
                                "D" => Color32::from_rgb(0xE0, 0x6C, 0x75),
                                _ => theme.text_silver,
                            };

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&fc.status)
                                        .color(status_col)
                                        .strong()
                                        .size(10.0),
                                );

                                // Short filename clickable for Inline Diff
                                let display_name = fc
                                    .path
                                    .rsplit('/')
                                    .next()
                                    .unwrap_or(&fc.path);
                                if ui
                                    .link(
                                        egui::RichText::new(display_name)
                                            .color(if fc.is_staged {
                                                theme.accent_pop
                                            } else {
                                                theme.text_paper
                                            })
                                            .size(10.5),
                                    )
                                    .on_hover_text(format!(
                                        "{} (+{} -{}) — Click xem Inline Diff",
                                        fc.path, fc.added_lines, fc.removed_lines
                                    ))
                                    .clicked()
                                {
                                    action = Some(SidebarAction::ViewFileDiff(fc.path.clone()));
                                }

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui
                                        .small_button("↺")
                                        .on_hover_text("Discard thay đổi")
                                        .clicked()
                                    {
                                        action = Some(SidebarAction::DiscardFile(fc.path.clone()));
                                    }
                                    if fc.is_staged {
                                        if ui
                                            .small_button("−")
                                            .on_hover_text("Unstage file")
                                            .clicked()
                                        {
                                            action =
                                                Some(SidebarAction::UnstageFile(fc.path.clone()));
                                        }
                                    } else if ui
                                        .small_button("＋")
                                        .on_hover_text("Stage file")
                                        .clicked()
                                    {
                                        action = Some(SidebarAction::StageFile(fc.path.clone()));
                                    }
                                });
                            });
                        }
                    });
            }

            // Commit Composer + AI Auto-Message + Push/Pull/MR
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let w = (ui.available_width() - 34.0).max(60.0);
                ui.add_sized(
                    vec2(w, 22.0),
                    egui::TextEdit::singleline(commit_message)
                        .hint_text("Commit message (feat: ...)"),
                );
                if ui
                    .button(egui::RichText::new("✨").size(10.5))
                    .on_hover_text("Tự động sinh Commit Message chuẩn Conventional Commit từ Diff")
                    .clicked()
                {
                    action = Some(SidebarAction::AiGenerateCommitMsg);
                }
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .button(
                        egui::RichText::new("✓ Commit")
                            .color(Color32::BLACK)
                            .strong()
                            .size(10.5),
                    )
                    .clicked()
                {
                    let msg = if commit_message.trim().is_empty() {
                        "feat(victini): autonomous vibe update".to_string()
                    } else {
                        commit_message.trim().to_string()
                    };
                    action = Some(SidebarAction::CommitWithMessage(msg));
                }
                if ui
                    .button(egui::RichText::new("⬆ Push").size(10.5))
                    .on_hover_text("git push -u origin <branch>")
                    .clicked()
                {
                    action = Some(SidebarAction::GitPush);
                }
                if ui
                    .button(egui::RichText::new("⬇ Pull").size(10.5))
                    .on_hover_text("git pull --rebase")
                    .clicked()
                {
                    action = Some(SidebarAction::GitPull);
                }
            });

            ui.add_space(4.0);
            if ui
                .add_sized(
                    vec2(ui.available_width(), 24.0),
                    egui::Button::new(
                        egui::RichText::new("🦊 Push & Create GitLab MR")
                            .color(Color32::BLACK)
                            .strong()
                            .size(10.5),
                    )
                    .fill(theme.accent_pop)
                    .rounding(Rounding::same(6.0)),
                )
                .clicked()
            {
                action = Some(SidebarAction::CreateGitLabMr);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // 6. PROJECT SKILLS & AGENTS
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "SKILLS ({}) & AGENTS ({})",
                        skills.len(),
                        agents.len()
                    ))
                    .color(theme.text_silver)
                    .strong()
                    .size(10.0),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .button(egui::RichText::new("+ Skill").size(9.5))
                        .clicked()
                    {
                        action = Some(SidebarAction::OpenSkillStudioNewSkill);
                    }
                    if ui
                        .button(egui::RichText::new("+ Agent").size(9.5))
                        .clicked()
                    {
                        action = Some(SidebarAction::OpenSkillStudioNewAgent);
                    }
                });
            });

            ui.add_space(4.0);
            ScrollArea::vertical()
                .id_salt("skills_agents_sidebar_scroll")
                .max_height(100.0)
                .show(ui, |ui| {
                    for (idx, sk) in skills.iter().enumerate() {
                        if !q_lower.is_empty() && !sk.name.to_lowercase().contains(&q_lower) {
                            continue;
                        }
                        if ui
                            .selectable_label(
                                false,
                                egui::RichText::new(format!("🔧 ${}", sk.name))
                                    .color(theme.text_paper)
                                    .size(10.5),
                            )
                            .on_hover_text(&sk.description)
                            .clicked()
                        {
                            action = Some(SidebarAction::InspectSkill(idx));
                        }
                    }
                    for (idx, ag) in agents.iter().enumerate() {
                        if !q_lower.is_empty() && !ag.name.to_lowercase().contains(&q_lower) {
                            continue;
                        }
                        if ui
                            .selectable_label(
                                false,
                                egui::RichText::new(format!("🤖 @{}", ag.name))
                                    .color(theme.accent_pop)
                                    .size(10.5),
                            )
                            .on_hover_text(&ag.role)
                            .clicked()
                        {
                            action = Some(SidebarAction::InspectAgent(idx));
                        }
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // 7. RECENT CONVERSATIONS (Persisted Threads)
            ui.label(
                egui::RichText::new(format!("CONVERSATIONS ({})", thread_titles.len()))
                    .color(theme.text_silver)
                    .strong()
                    .size(10.0),
            );
            ui.add_space(3.0);
            ScrollArea::vertical()
                .id_salt("recent_conv_sidebar_scroll")
                .max_height(100.0)
                .show(ui, |ui| {
                    for (idx, title, updated) in thread_titles {
                        if !q_lower.is_empty() && !title.to_lowercase().contains(&q_lower) {
                            continue;
                        }
                        let is_active = *idx == active_thread_idx;
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    is_active,
                                    egui::RichText::new(title)
                                        .color(if is_active {
                                            theme.accent_pop
                                        } else {
                                            theme.text_silver
                                        })
                                        .size(10.5),
                                )
                                .clicked()
                            {
                                action = Some(SidebarAction::SelectChatThread(*idx));
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(updated)
                                        .color(theme.text_silver)
                                        .size(9.0),
                                );
                            });
                        });
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .button(
                        egui::RichText::new("⚙ Settings & Reaper")
                            .size(10.5)
                            .color(theme.text_silver),
                    )
                    .clicked()
                {
                    action = Some(SidebarAction::SwitchCenterView(CenterViewMode::SettingsModal));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("v1.0 Full")
                            .color(theme.accent_pop)
                            .size(9.5),
                    );
                });
            });
        });

    action
}
