use crate::ai::{AgentTaskCard, KanbanColumn};
use crate::emulator::{EmulatorState, TouchRipple, ELEGANS_VIEWPORT_PRESETS};
use crate::git::{GitFileChange, WorktreeInfo};
use crate::theme::{CanhTheme, Lucide};
use crate::ui::header::icon_btn;
use egui::{
    pos2, vec2, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rounding, ScrollArea,
    Stroke, TextureOptions, Ui,
};
use std::path::PathBuf;

pub enum BeeBoxAction {
    OpenProjectsModal,
    OpenBuiltInEditor,
    OpenExternalEditor(Option<String>),
    InspectFile { path: String, edit_mode: bool },
    StageFile(String),
    UnstageFile(String),
    StageAll,
    UnstageAll,
    DiscardFile(String),
    SwitchBranch(String),
    CreateBranch(String),
    SetupWorktree(String),
    SwitchWorktree(PathBuf),
    RemoveWorktree(PathBuf),
    AiGenerateCommitMsg,
    Commit(String),
    GitPush,
    GitPull,
    GitStashSave,
    GitStashPop,
    OpenGitLabMr,
    TriggerTap(u32, u32),
    SwitchPreset(usize),
    CaptureAdbFrame,
    OpenPreviewUrl(String),
    RunBackgroundTask(String),
}

pub struct BeeBoxUiState {
    pub show_branch_picker: bool,
    pub show_worktree_manager: bool,
    pub show_commit_box: bool,
    pub show_changes_list: bool,
    pub show_emulator: bool,
    pub new_branch_name: String,
    pub new_worktree_branch: String,
    pub confirm_discard_path: Option<String>,
}

impl Default for BeeBoxUiState {
    fn default() -> Self {
        Self {
            show_branch_picker: false,
            show_worktree_manager: false,
            show_commit_box: true,
            show_changes_list: true,
            show_emulator: true,
            new_branch_name: String::new(),
            new_worktree_branch: String::new(),
            confirm_discard_path: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_beebox_panel(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    project_name: &str,
    branch_name: &str,
    commit_oid: &str,
    branches: &[String],
    worktrees: &[WorktreeInfo],
    file_changes: &[GitFileChange],
    commit_message: &mut String,
    beebox_state: &mut BeeBoxUiState,
    kanban_tasks: &[AgentTaskCard],
    emu_state: &mut EmulatorState,
) -> Option<BeeBoxAction> {
    let mut action = None;

    // T3 CODE BEEBOX CONTAINER (Adapts dynamically to Dark Mode & Light Mode)
    Frame::none()
        .fill(theme.panel_bg)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("t3_beebox_panel_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // 1. Header: BeeBox · project_name ... [Folder Icon]
                    ui.horizontal(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
                        ui.painter().rect_stroke(
                            rect,
                            Rounding::same(2.5),
                            Stroke::new(1.2, theme.accent_pop),
                        );

                        ui.add_space(3.0);
                        ui.label(
                            egui::RichText::new("BeeBox")
                                .color(theme.text_paper)
                                .strong()
                                .size(12.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("· {}", project_name))
                                .color(theme.text_silver)
                                .size(10.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::FOLDER,
                                theme.text_silver,
                                Color32::TRANSPARENT,
                                "Quản lý Projects & Workspaces",
                            )
                            .clicked()
                            {
                                action = Some(BeeBoxAction::OpenProjectsModal);
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // 2. Action Accordions: Open in editor & Setup Worktree (Icon-First)
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::CODE)
                                .color(theme.text_silver)
                                .size(11.5),
                        );
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Open in editor")
                                        .color(theme.text_paper)
                                        .size(11.0),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .on_hover_text("Mở trình sửa code & xem Diff trực tiếp")
                            .clicked()
                        {
                            action = Some(BeeBoxAction::OpenBuiltInEditor);
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::EXTERNAL_LINK,
                                theme.text_silver,
                                theme.chip_bg,
                                "Mở thư mục project bằng Cursor / VS Code / Zed",
                            )
                            .clicked()
                            {
                                action = Some(BeeBoxAction::OpenExternalEditor(None));
                            }
                        });
                    });

                    ui.add_space(3.0);

                    // Setup Worktree Accordion Header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::GIT_BRANCH)
                                .color(theme.text_silver)
                                .size(11.5),
                        );
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "Worktrees ({})",
                                        worktrees.len()
                                    ))
                                    .color(theme.text_paper)
                                    .size(11.0),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            beebox_state.show_worktree_manager =
                                !beebox_state.show_worktree_manager;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let chev = if beebox_state.show_worktree_manager {
                                Lucide::CHEVRON_DOWN
                            } else {
                                Lucide::CHEVRON_RIGHT
                            };
                            if icon_btn(
                                ui,
                                chev,
                                theme.text_silver,
                                Color32::TRANSPARENT,
                                "Đóng/mở danh sách Git Worktrees",
                            )
                            .clicked()
                            {
                                beebox_state.show_worktree_manager =
                                    !beebox_state.show_worktree_manager;
                            }
                        });
                    });

                    if beebox_state.show_worktree_manager {
                        ui.add_space(4.0);
                        Frame::none()
                            .fill(theme.surface_1)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                for (idx, wt) in worktrees.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        let col = if wt.is_head {
                                            theme.accent_pop
                                        } else {
                                            theme.text_paper
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(format!(
                                                        "{} {}",
                                                        Lucide::FOLDER,
                                                        wt.name
                                                    ))
                                                    .color(col)
                                                    .size(10.5),
                                                )
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .on_hover_text(wt.path.to_string_lossy())
                                            .clicked()
                                        {
                                            action = Some(BeeBoxAction::SwitchWorktree(
                                                wt.path.clone(),
                                            ));
                                        }
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if idx > 0
                                                && icon_btn(
                                                    ui,
                                                    Lucide::TRASH,
                                                    theme.danger,
                                                    Color32::TRANSPARENT,
                                                    "Xóa Worktree này",
                                                )
                                                .clicked()
                                            {
                                                action = Some(BeeBoxAction::RemoveWorktree(
                                                    wt.path.clone(),
                                                ));
                                            }
                                        });
                                    });
                                }
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let w = (ui.available_width() - 36.0).max(60.0);
                                    Frame::none()
                                        .fill(theme.input_bg)
                                        .rounding(Rounding::same(4.0))
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .inner_margin(Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                vec2(w, 18.0),
                                                egui::TextEdit::singleline(
                                                    &mut beebox_state.new_worktree_branch,
                                                )
                                                .hint_text("feat/new-wt...")
                                                .frame(false)
                                                .text_color(theme.text_paper),
                                            );
                                        });
                                    if icon_btn(
                                        ui,
                                        Lucide::FOLDER_PLUS,
                                        theme.on_accent,
                                        theme.accent_pop,
                                        "Khởi tạo Git Worktree mới song song",
                                    )
                                    .clicked()
                                    {
                                        let b =
                                            if beebox_state.new_worktree_branch.trim().is_empty() {
                                                format!("feat/wt-{}", worktrees.len() + 1)
                                            } else {
                                                beebox_state.new_worktree_branch.trim().to_string()
                                            };
                                        beebox_state.new_worktree_branch.clear();
                                        action = Some(BeeBoxAction::SetupWorktree(b));
                                    }
                                });
                            });
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 3. Git Branch Switcher & Creator
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::GIT_BRANCH)
                                .color(theme.accent_pop)
                                .size(11.5),
                        );
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(branch_name)
                                        .color(theme.text_paper)
                                        .strong()
                                        .size(11.5),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .on_hover_text("Click để chuyển nhánh hoặc tạo nhánh mới")
                            .clicked()
                        {
                            beebox_state.show_branch_picker = !beebox_state.show_branch_picker;
                        }
                        ui.label(
                            egui::RichText::new(format!("({})", commit_oid))
                                .color(theme.text_silver)
                                .monospace()
                                .size(9.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let chev = if beebox_state.show_branch_picker {
                                Lucide::CHEVRON_DOWN
                            } else {
                                Lucide::CHEVRON_RIGHT
                            };
                            if icon_btn(
                                ui,
                                chev,
                                theme.text_silver,
                                Color32::TRANSPARENT,
                                "Chọn / Tạo nhánh Git",
                            )
                            .clicked()
                            {
                                beebox_state.show_branch_picker = !beebox_state.show_branch_picker;
                            }
                        });
                    });

                    if beebox_state.show_branch_picker {
                        ui.add_space(4.0);
                        Frame::none()
                            .fill(theme.surface_1)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                for b in branches {
                                    let is_cur = b == branch_name;
                                    ui.horizontal(|ui| {
                                        let label_txt = if is_cur {
                                            format!("{} {}", Lucide::CHECK, b)
                                        } else {
                                            format!("  {}", b)
                                        };
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(label_txt)
                                                        .color(if is_cur {
                                                            theme.accent_pop
                                                        } else {
                                                            theme.text_paper
                                                        })
                                                        .size(10.5),
                                                )
                                                .fill(Color32::TRANSPARENT),
                                            )
                                            .clicked()
                                            && !is_cur
                                        {
                                            beebox_state.show_branch_picker = false;
                                            action = Some(BeeBoxAction::SwitchBranch(b.clone()));
                                        }
                                    });
                                }

                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    let w = (ui.available_width() - 36.0).max(60.0);
                                    Frame::none()
                                        .fill(theme.input_bg)
                                        .rounding(Rounding::same(4.0))
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .inner_margin(Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                vec2(w, 18.0),
                                                egui::TextEdit::singleline(
                                                    &mut beebox_state.new_branch_name,
                                                )
                                                .hint_text("New branch...")
                                                .frame(false)
                                                .text_color(theme.text_paper),
                                            );
                                        });
                                    if icon_btn(
                                        ui,
                                        Lucide::PLUS,
                                        theme.on_accent,
                                        theme.accent_pop,
                                        "Tạo & Checkout sang nhánh mới",
                                    )
                                    .clicked()
                                        && !beebox_state.new_branch_name.trim().is_empty()
                                    {
                                        let b = beebox_state.new_branch_name.trim().to_string();
                                        beebox_state.new_branch_name.clear();
                                        beebox_state.show_branch_picker = false;
                                        action = Some(BeeBoxAction::CreateBranch(b));
                                    }
                                });
                            });
                    }

                    ui.add_space(4.0);

                    // 4. Commit / Push / Pull / Stash / GitLab MR (100% Icon-First Action Bar)
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::GIT_COMMIT)
                                .color(theme.text_silver)
                                .size(11.5),
                        );
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Git Operations")
                                        .color(theme.text_paper)
                                        .size(11.0),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            beebox_state.show_commit_box = !beebox_state.show_commit_box;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::GIT_PR,
                                theme.accent_pop,
                                theme.accent_surface,
                                "Tạo GitLab Merge Request cho nhánh hiện tại",
                            )
                            .clicked()
                            {
                                action = Some(BeeBoxAction::OpenGitLabMr);
                            }
                        });
                    });

                    if beebox_state.show_commit_box {
                        ui.add_space(4.0);
                        Frame::none()
                            .fill(theme.surface_1)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                // Commit Message Input
                                Frame::none()
                                    .fill(theme.input_bg)
                                    .rounding(Rounding::same(5.0))
                                    .stroke(Stroke::new(1.0, theme.border))
                                    .inner_margin(Margin::symmetric(8.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(commit_message)
                                                .desired_rows(2)
                                                .desired_width(ui.available_width())
                                                .frame(false)
                                                .text_color(theme.text_paper)
                                                .hint_text("Commit message..."),
                                        );
                                    });

                                ui.add_space(5.0);

                                // 7-Icon Git Action Bar: [Wand AI] [Check Commit] [Upload Push] [Download Pull] [Archive Stash] [ArchiveRestore Pop] [GitPR MR]
                                ui.horizontal(|ui| {
                                    if icon_btn(
                                        ui,
                                        Lucide::WAND,
                                        theme.accent_pop,
                                        theme.chip_bg,
                                        "AI tự động sinh Commit Message từ Diff hiện tại",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::AiGenerateCommitMsg);
                                    }

                                    if icon_btn(
                                        ui,
                                        Lucide::CHECK,
                                        theme.on_accent,
                                        theme.accent_pop,
                                        "Commit các thay đổi (Tự động Stage nếu chưa Stage)",
                                    )
                                    .clicked()
                                    {
                                        let msg = if commit_message.trim().is_empty() {
                                            format!(
                                                "feat({}): update workspace changes",
                                                branch_name
                                            )
                                        } else {
                                            commit_message.trim().to_string()
                                        };
                                        action = Some(BeeBoxAction::Commit(msg));
                                    }

                                    if icon_btn(
                                        ui,
                                        Lucide::UPLOAD,
                                        theme.text_paper,
                                        theme.chip_bg,
                                        "Git Push lên origin (git push -u origin)",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitPush);
                                    }

                                    if icon_btn(
                                        ui,
                                        Lucide::DOWNLOAD,
                                        theme.text_paper,
                                        theme.chip_bg,
                                        "Git Pull Rebase (git pull --rebase --autostash)",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitPull);
                                    }

                                    if icon_btn(
                                        ui,
                                        Lucide::ARCHIVE,
                                        theme.text_silver,
                                        theme.chip_bg,
                                        "Git Stash (Lưu tạm thay đổi chưa commit)",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitStashSave);
                                    }

                                    if icon_btn(
                                        ui,
                                        Lucide::ARCHIVE_RESTORE,
                                        theme.text_silver,
                                        theme.chip_bg,
                                        "Git Stash Pop (Khôi phục thay đổi từ Stash)",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitStashPop);
                                    }
                                });
                            });
                    }

                    ui.add_space(6.0);

                    // 5. Real-time Git Changes Inspector (Icon-First Controls)
                    let total_add: usize = file_changes.iter().map(|c| c.added_lines).sum();
                    let total_rem: usize = file_changes.iter().map(|c| c.removed_lines).sum();
                    let staged_files: Vec<&GitFileChange> =
                        file_changes.iter().filter(|c| c.is_staged).collect();
                    let unstaged_files: Vec<&GitFileChange> =
                        file_changes.iter().filter(|c| !c.is_staged).collect();

                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "Changes ({})",
                                        file_changes.len()
                                    ))
                                    .color(theme.text_paper)
                                    .strong()
                                    .size(11.0),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            beebox_state.show_changes_list = !beebox_state.show_changes_list;
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("-{}", total_rem))
                                    .color(theme.danger)
                                    .monospace()
                                    .size(10.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("+{}", total_add))
                                    .color(theme.success)
                                    .monospace()
                                    .size(10.5),
                            );
                            if !unstaged_files.is_empty()
                                && icon_btn(
                                    ui,
                                    Lucide::PLUS,
                                    theme.success,
                                    theme.chip_bg,
                                    "Stage tất cả file thay đổi",
                                )
                                .clicked()
                            {
                                action = Some(BeeBoxAction::StageAll);
                            }
                            if !staged_files.is_empty()
                                && icon_btn(
                                    ui,
                                    Lucide::MINUS,
                                    theme.warning,
                                    theme.chip_bg,
                                    "Unstage tất cả file",
                                )
                                .clicked()
                            {
                                action = Some(BeeBoxAction::UnstageAll);
                            }
                        });
                    });

                    if beebox_state.show_changes_list {
                        ui.add_space(4.0);
                        if file_changes.is_empty() {
                            ui.label(
                                egui::RichText::new("Working tree clean")
                                    .color(theme.text_silver)
                                    .size(10.5),
                            );
                        } else {
                            // Staged Section
                            if !staged_files.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "STAGED ({})",
                                        staged_files.len()
                                    ))
                                    .color(theme.success)
                                    .strong()
                                    .size(9.5),
                                );
                                for f in &staged_files {
                                    render_file_change_row(
                                        ui,
                                        theme,
                                        f,
                                        &mut beebox_state.confirm_discard_path,
                                        &mut action,
                                    );
                                }
                                ui.add_space(4.0);
                            }

                            // Unstaged Section
                            if !unstaged_files.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "WORKING TREE ({})",
                                        unstaged_files.len()
                                    ))
                                    .color(theme.text_silver)
                                    .strong()
                                    .size(9.5),
                                );
                                for f in &unstaged_files {
                                    render_file_change_row(
                                        ui,
                                        theme,
                                        f,
                                        &mut beebox_state.confirm_discard_path,
                                        &mut action,
                                    );
                                }
                            }
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 6. Lineage - Real Active & Pending Tasks
                    let active_tasks: Vec<&AgentTaskCard> = kanban_tasks
                        .iter()
                        .filter(|t| t.column == KanbanColumn::InProgress || t.is_running_bg)
                        .collect();

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Lineage · {} active / {} total",
                                active_tasks.len(),
                                kanban_tasks.len()
                            ))
                            .color(theme.text_silver)
                            .size(10.5),
                        );
                    });

                    ui.add_space(4.0);

                    for task in kanban_tasks.iter().take(5) {
                        let dot_col = match task.column {
                            KanbanColumn::InProgress => theme.accent_pop,
                            KanbanColumn::Verifying => theme.info,
                            KanbanColumn::Done => theme.success,
                            KanbanColumn::Backlog => theme.text_silver,
                        };
                        ui.horizontal(|ui| {
                            let (r, _) =
                                ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                            ui.painter().circle_filled(r.center(), 2.5, dot_col);

                            let short_title: String = task.title.chars().take(22).collect();
                            ui.label(
                                egui::RichText::new(short_title)
                                    .color(theme.text_paper)
                                    .size(10.5),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if !task.is_running_bg
                                    && task.column != KanbanColumn::Done
                                    && icon_btn(
                                        ui,
                                        Lucide::PLAY,
                                        theme.accent_pop,
                                        Color32::TRANSPARENT,
                                        "Chạy tác vụ này dưới nền (Background Agent)",
                                    )
                                    .clicked()
                                {
                                    action =
                                        Some(BeeBoxAction::RunBackgroundTask(task.id.clone()));
                                } else {
                                    ui.label(
                                        egui::RichText::new(task.column.title())
                                            .color(theme.text_silver)
                                            .size(9.0),
                                    );
                                }
                            });
                        });
                        ui.add_space(2.0);
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 7. Live Touch Viewport Canvas with 5-Device Preset Switcher & Real ADB Texture
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Live Touch Viewport")
                                        .color(theme.text_paper)
                                        .strong()
                                        .size(10.5),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            beebox_state.show_emulator = !beebox_state.show_emulator;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::CAMERA,
                                theme.text_paper,
                                theme.chip_bg,
                                "Chụp & đồng bộ khung hình ADB thực tế (Snap)",
                            )
                            .clicked()
                            {
                                action = Some(BeeBoxAction::CaptureAdbFrame);
                            }
                        });
                    });

                    if beebox_state.show_emulator {
                        ui.add_space(4.0);

                        // 5-Device Preset Icon Bar
                        ui.horizontal(|ui| {
                            let preset_icons = [
                                (0, Lucide::SMARTPHONE),
                                (1, Lucide::SMARTPHONE),
                                (2, Lucide::TABLET),
                                (3, Lucide::LAPTOP),
                                (4, Lucide::ACTIVITY),
                            ];
                            for (idx, ic) in preset_icons {
                                let p = ELEGANS_VIEWPORT_PRESETS[idx];
                                let is_sel = emu_state.active_preset_idx == idx;
                                if icon_btn(
                                    ui,
                                    ic,
                                    if is_sel {
                                        theme.on_accent
                                    } else {
                                        theme.text_silver
                                    },
                                    if is_sel {
                                        theme.accent_pop
                                    } else {
                                        theme.chip_bg
                                    },
                                    p.name,
                                )
                                .clicked()
                                {
                                    action = Some(BeeBoxAction::SwitchPreset(idx));
                                }
                            }
                        });

                        ui.add_space(4.0);

                        // Preview URL Quick Row
                        ui.horizontal(|ui| {
                            let url_w = (ui.available_width() - 34.0).max(80.0);
                            Frame::none()
                                .fill(theme.input_bg)
                                .rounding(Rounding::same(4.0))
                                .stroke(Stroke::new(1.0, theme.border))
                                .inner_margin(Margin::symmetric(6.0, 2.0))
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        vec2(url_w, 16.0),
                                        egui::TextEdit::singleline(&mut emu_state.preview_url)
                                            .hint_text("http://localhost:3000")
                                            .frame(false)
                                            .text_color(theme.text_paper),
                                    );
                                });
                            if icon_btn(
                                ui,
                                Lucide::EXTERNAL_LINK,
                                theme.accent_pop,
                                theme.chip_bg,
                                "Mở Preview URL trên trình duyệt cách ly",
                            )
                            .clicked()
                            {
                                action = Some(BeeBoxAction::OpenPreviewUrl(
                                    emu_state.preview_url.clone(),
                                ));
                            }
                        });

                        ui.add_space(4.0);

                        let current_preset = ELEGANS_VIEWPORT_PRESETS[emu_state.active_preset_idx];
                        let aspect = (current_preset.height as f32 / current_preset.width as f32)
                            .clamp(0.65, 1.65);
                        let emu_w = (ui.available_width() - 8.0).clamp(150.0, 230.0);
                        let emu_h = (emu_w * aspect).clamp(120.0, 310.0);

                        Frame::none()
                            .fill(theme.code_bg)
                            .rounding(Rounding::same(14.0))
                            .stroke(Stroke::new(1.0, theme.border_strong))
                            .inner_margin(Margin::same(6.0))
                            .show(ui, |ui| {
                                let (rect, resp) = ui.allocate_exact_size(
                                    vec2(emu_w, emu_h),
                                    egui::Sense::click_and_drag(),
                                );
                                if resp.clicked() {
                                    if let Some(pos) = resp.interact_pointer_pos() {
                                        let rel_x =
                                            ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                                        let rel_y =
                                            ((pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                                        let tap_x = (rel_x * current_preset.width as f32) as u32;
                                        let tap_y = (rel_y * current_preset.height as f32) as u32;
                                        emu_state.last_touch = Some(TouchRipple {
                                            norm_pos: Pos2::new(rel_x, rel_y),
                                            device_x: tap_x,
                                            device_y: tap_y,
                                            created_at: elapsed_secs,
                                        });
                                        action = Some(BeeBoxAction::TriggerTap(tap_x, tap_y));
                                    }
                                }

                                let painter = ui.painter_at(rect);

                                // Render real ADB screenshot texture if available!
                                if let Some(ref frame_img) = emu_state.current_frame {
                                    let tex = ui.ctx().load_texture(
                                        "adb_live_frame",
                                        frame_img.clone(),
                                        TextureOptions::LINEAR,
                                    );
                                    painter.image(
                                        tex.id(),
                                        rect,
                                        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                        Color32::WHITE,
                                    );
                                } else {
                                    painter.text(
                                        rect.center(),
                                        Align2::CENTER_CENTER,
                                        format!(
                                            "{}\n{}×{}\n{}",
                                            current_preset
                                                .name
                                                .split('(')
                                                .next()
                                                .unwrap_or(current_preset.name)
                                                .trim(),
                                            current_preset.width,
                                            current_preset.height,
                                            emu_state.status_text
                                        ),
                                        FontId::proportional(10.0),
                                        theme.text_silver,
                                    );
                                }

                                if let Some(ref touch) = emu_state.last_touch {
                                    let tap_pt = pos2(
                                        rect.min.x + touch.norm_pos.x * rect.width(),
                                        rect.min.y + touch.norm_pos.y * rect.height(),
                                    );
                                    painter.circle_filled(tap_pt, 5.0, theme.accent_pop);
                                    painter.circle_stroke(
                                        tap_pt,
                                        9.0,
                                        Stroke::new(1.2, theme.accent_pop),
                                    );
                                }
                            });
                    }
                });
        });

    action
}

fn render_file_change_row(
    ui: &mut Ui,
    theme: &CanhTheme,
    f: &GitFileChange,
    confirm_discard_path: &mut Option<String>,
    action: &mut Option<BeeBoxAction>,
) {
    let status_col = match f.status.as_str() {
        "A" | "?" => theme.success,
        "D" => theme.danger,
        _ => theme.warning,
    };

    let file_name = f.path.rsplit('/').next().unwrap_or(&f.path);

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(&f.status)
                .color(status_col)
                .monospace()
                .strong()
                .size(10.0),
        );

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(file_name)
                        .color(theme.text_paper)
                        .size(10.8),
                )
                .fill(Color32::TRANSPARENT),
            )
            .on_hover_text(format!(
                "{}\n+{} -{}\nClick để xem Diff & sửa code",
                f.path, f.added_lines, f.removed_lines
            ))
            .clicked()
        {
            *action = Some(BeeBoxAction::InspectFile {
                path: f.path.clone(),
                edit_mode: false,
            });
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Stage / Unstage Icon Toggle
            if f.is_staged {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::MINUS)
                                .color(theme.warning)
                                .size(10.5),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Unstage file này")
                    .clicked()
                {
                    *action = Some(BeeBoxAction::UnstageFile(f.path.clone()));
                }
            } else {
                // Discard Icon Button with confirmation guard
                let is_confirming = confirm_discard_path.as_deref() == Some(f.path.as_str());
                if is_confirming {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(Lucide::CHECK)
                                    .color(Color32::WHITE)
                                    .size(10.0),
                            )
                            .fill(theme.danger)
                            .rounding(Rounding::same(3.0)),
                        )
                        .on_hover_text("Xác nhận khôi phục file về HEAD")
                        .clicked()
                    {
                        *confirm_discard_path = None;
                        *action = Some(BeeBoxAction::DiscardFile(f.path.clone()));
                    }
                } else if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::ROTATE_CCW)
                                .color(theme.text_silver)
                                .size(10.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Discard thay đổi của file này")
                    .clicked()
                {
                    *confirm_discard_path = Some(f.path.clone());
                }

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::PLUS)
                                .color(theme.success)
                                .size(10.5),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Stage file này")
                    .clicked()
                {
                    *action = Some(BeeBoxAction::StageFile(f.path.clone()));
                }
            }

            // Direct Live Edit Icon Button
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(Lucide::EDIT)
                            .color(theme.text_silver)
                            .size(10.0),
                    )
                    .fill(Color32::TRANSPARENT),
                )
                .on_hover_text("Sửa trực tiếp nội dung file (Live Code Editor)")
                .clicked()
            {
                *action = Some(BeeBoxAction::InspectFile {
                    path: f.path.clone(),
                    edit_mode: true,
                });
            }
        });
    });
}
