use crate::ai::{AgentTaskCard, KanbanColumn};
use crate::emulator::{EmulatorState, TouchRipple, ELEGANS_VIEWPORT_PRESETS};
use crate::git::{GitFileChange, WorktreeInfo};
use crate::theme::{CanhTheme, Lucide};
use egui::{
    pos2, vec2, Align, Align2, Color32, FontId, Frame, Layout, Margin, Pos2, Rounding, ScrollArea,
    Stroke, Ui,
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

    // T3 CODE EXACT BEEBOX CONTAINER (#111111)
    Frame::none()
        .fill(Color32::from_rgb(0x11, 0x11, 0x11))
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(0x19, 0x19, 0x19)))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_salt("t3_beebox_panel_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // 1. Header: BeeBox · project_name ... [folder icon]
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(14.0, 14.0), egui::Sense::hover());
                        ui.painter().rect_stroke(
                            rect,
                            Rounding::same(2.0),
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
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(Lucide::FOLDER)
                                            .color(theme.text_silver)
                                            .size(12.0),
                                    )
                                    .fill(Color32::TRANSPARENT),
                                )
                                .on_hover_text("Quản lý Projects & Workspaces")
                                .clicked()
                            {
                                action = Some(BeeBoxAction::OpenProjectsModal);
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 2. Action Accordions: Open in editor & Setup Worktree
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(Lucide::CODE).color(theme.text_silver).size(11.0));
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
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("VSCode/Cursor")
                                            .color(theme.text_silver)
                                            .size(9.5),
                                    )
                                    .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                    .rounding(Rounding::same(4.0)),
                                )
                                .on_hover_text("Mở thư mục project bằng Cursor / VS Code")
                                .clicked()
                            {
                                action = Some(BeeBoxAction::OpenExternalEditor(None));
                            }
                        });
                    });

                    ui.add_space(4.0);

                    // Setup Worktree Accordion Header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::GIT_BRANCH)
                                .color(theme.text_silver)
                                .size(11.0),
                        );
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "Setup Worktree ({})",
                                        worktrees.len()
                                    ))
                                    .color(theme.text_paper)
                                    .size(11.0),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            beebox_state.show_worktree_manager = !beebox_state.show_worktree_manager;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let chev = if beebox_state.show_worktree_manager {
                                Lucide::CHEVRON_DOWN
                            } else {
                                Lucide::CHEVRON_RIGHT
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(chev)
                                            .color(theme.text_silver)
                                            .size(10.0),
                                    )
                                    .fill(Color32::TRANSPARENT),
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
                            .fill(Color32::from_rgb(0x0A, 0x0A, 0x0A))
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
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
                                                && ui
                                                    .add(
                                                        egui::Button::new(
                                                            egui::RichText::new(Lucide::TRASH)
                                                                .color(theme.text_silver)
                                                                .size(10.0),
                                                        )
                                                        .fill(Color32::TRANSPARENT),
                                                    )
                                                    .on_hover_text("Xóa Worktree này")
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
                                    let w = (ui.available_width() - 48.0).max(60.0);
                                    Frame::none()
                                        .fill(Color32::from_rgb(0x12, 0x12, 0x12))
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
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("+ WT")
                                                    .color(Color32::BLACK)
                                                    .strong()
                                                    .size(10.0),
                                            )
                                            .fill(theme.accent_pop)
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .clicked()
                                    {
                                        let b = if beebox_state.new_worktree_branch.trim().is_empty()
                                        {
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

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 3. Git Branch Switcher & Creator
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::GIT_BRANCH)
                                .color(theme.accent_pop)
                                .size(11.0),
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
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(chev)
                                            .color(theme.text_silver)
                                            .size(10.0),
                                    )
                                    .fill(Color32::TRANSPARENT),
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
                            .fill(Color32::from_rgb(0x0A, 0x0A, 0x0A))
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                for b in branches {
                                    let is_cur = b == branch_name;
                                    ui.horizontal(|ui| {
                                        let label_txt = if is_cur {
                                            format!("{} {} (active)", Lucide::CHECK, b)
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
                                    let w = (ui.available_width() - 56.0).max(60.0);
                                    Frame::none()
                                        .fill(Color32::from_rgb(0x12, 0x12, 0x12))
                                        .rounding(Rounding::same(4.0))
                                        .stroke(Stroke::new(1.0, theme.border))
                                        .inner_margin(Margin::symmetric(6.0, 2.0))
                                        .show(ui, |ui| {
                                            ui.add_sized(
                                                vec2(w, 18.0),
                                                egui::TextEdit::singleline(
                                                    &mut beebox_state.new_branch_name,
                                                )
                                                .hint_text("New branch name...")
                                                .frame(false)
                                                .text_color(theme.text_paper),
                                            );
                                        });
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Checkout")
                                                    .color(Color32::BLACK)
                                                    .strong()
                                                    .size(9.5),
                                            )
                                            .fill(theme.accent_pop)
                                            .rounding(Rounding::same(4.0)),
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

                    ui.add_space(5.0);

                    // 4. Commit / Push / Pull / Stash / GitLab MR Accordion
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(Lucide::GIT_COMMIT)
                                .color(theme.text_silver)
                                .size(11.0),
                        );
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Commit & Git Operations")
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
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!("{} MR", Lucide::GIT_PR))
                                            .color(theme.accent_pop)
                                            .size(10.0),
                                    )
                                    .fill(Color32::from_rgb(0x1F, 0x12, 0x10))
                                    .rounding(Rounding::same(4.0)),
                                )
                                .on_hover_text("Mở GitLab Merge Request")
                                .clicked()
                            {
                                action = Some(BeeBoxAction::OpenGitLabMr);
                            }
                        });
                    });

                    if beebox_state.show_commit_box {
                        ui.add_space(4.0);
                        Frame::none()
                            .fill(Color32::from_rgb(0x0A, 0x0A, 0x0A))
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1C, 0x1C, 0x1C)))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                // Dark Commit Message Input
                                Frame::none()
                                    .fill(Color32::from_rgb(0x12, 0x12, 0x12))
                                    .rounding(Rounding::same(5.0))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(0x20, 0x20, 0x20)))
                                    .inner_margin(Margin::symmetric(8.0, 5.0))
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(commit_message)
                                                .desired_rows(2)
                                                .desired_width(ui.available_width())
                                                .frame(false)
                                                .text_color(theme.text_paper)
                                                .hint_text("Commit message (feat/fix/chore)..."),
                                        );
                                    });

                                ui.add_space(5.0);

                                // Row 1: AI Generate Commit Msg + Commit Button
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} AI Msg",
                                                    Lucide::SPARKLES
                                                ))
                                                .color(theme.accent_pop)
                                                .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .on_hover_text("Tự động sinh Commit Message từ Diff hiện tại")
                                        .clicked()
                                    {
                                        action = Some(BeeBoxAction::AiGenerateCommitMsg);
                                    }

                                    let can_commit =
                                        !commit_message.trim().is_empty() || !file_changes.is_empty();
                                    if ui
                                        .add_enabled(
                                            can_commit,
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Commit",
                                                    Lucide::CHECK
                                                ))
                                                .color(Color32::BLACK)
                                                .strong()
                                                .size(10.5),
                                            )
                                            .fill(theme.accent_pop)
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .clicked()
                                    {
                                        let msg = if commit_message.trim().is_empty() {
                                            format!("feat({}): update workspace changes", branch_name)
                                        } else {
                                            commit_message.trim().to_string()
                                        };
                                        action = Some(BeeBoxAction::Commit(msg));
                                    }
                                });

                                ui.add_space(4.0);

                                // Row 2: Push | Pull | Stash | Pop
                                ui.horizontal_wrapped(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Push",
                                                    Lucide::ARROW_UP
                                                ))
                                                .color(theme.text_paper)
                                                .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .on_hover_text("git push -u origin <branch>")
                                        .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitPush);
                                    }

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Pull",
                                                    Lucide::REFRESH
                                                ))
                                                .color(theme.text_paper)
                                                .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .on_hover_text("git pull --rebase --autostash")
                                        .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitPull);
                                    }

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Stash")
                                                    .color(theme.text_silver)
                                                    .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .on_hover_text("Lưu tạm thay đổi (git stash push -u)")
                                        .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitStashSave);
                                    }

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Pop")
                                                    .color(theme.text_silver)
                                                    .size(10.0),
                                            )
                                            .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                            .rounding(Rounding::same(4.0)),
                                        )
                                        .on_hover_text("Khôi phục Stash (git stash pop)")
                                        .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitStashPop);
                                    }
                                });
                            });
                    }

                    ui.add_space(6.0);

                    // 5. Real-time Git Changes Inspector (Staged & Working Tree)
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
                                    .color(Color32::from_rgb(0xE0, 0x48, 0x48))
                                    .monospace()
                                    .size(10.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("+{}", total_add))
                                    .color(Color32::from_rgb(0x4E, 0xC9, 0xB0))
                                    .monospace()
                                    .size(10.5),
                            );
                            if !unstaged_files.is_empty()
                                && ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("+All")
                                                .color(theme.text_silver)
                                                .size(9.5),
                                        )
                                        .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                        .rounding(Rounding::same(3.0)),
                                    )
                                    .on_hover_text("Stage tất cả thay đổi")
                                    .clicked()
                            {
                                action = Some(BeeBoxAction::StageAll);
                            }
                            if !staged_files.is_empty()
                                && ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("-All")
                                                .color(theme.text_silver)
                                                .size(9.5),
                                        )
                                        .fill(Color32::from_rgb(0x18, 0x18, 0x18))
                                        .rounding(Rounding::same(3.0)),
                                    )
                                    .on_hover_text("Unstage tất cả file")
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
                                    .color(Color32::from_rgb(0x4E, 0xC9, 0xB0))
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
                            KanbanColumn::Verifying => Color32::from_rgb(0x51, 0xA2, 0xFF),
                            KanbanColumn::Done => Color32::from_rgb(0x4E, 0xC9, 0xB0),
                            KanbanColumn::Backlog => Color32::from_rgb(0x80, 0x80, 0x80),
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
                                    && ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(Lucide::PLAY)
                                                    .color(theme.accent_pop)
                                                    .size(9.5),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .on_hover_text("Chạy tác vụ này dưới nền (Background Agent)")
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

                    // 7. Live Touch Viewport Canvas
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Live Touch Viewport")
                                        .color(theme.text_silver)
                                        .size(10.5),
                                )
                                .fill(Color32::TRANSPARENT),
                            )
                            .clicked()
                        {
                            beebox_state.show_emulator = !beebox_state.show_emulator;
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Snap")
                                            .color(theme.text_paper)
                                            .size(10.0),
                                    )
                                    .fill(Color32::from_rgb(0x19, 0x19, 0x19)),
                                )
                                .clicked()
                            {
                                action = Some(BeeBoxAction::CaptureAdbFrame);
                            }
                        });
                    });

                    if beebox_state.show_emulator {
                        ui.add_space(4.0);

                        let current_preset = ELEGANS_VIEWPORT_PRESETS[emu_state.active_preset_idx];
                        let emu_w = (ui.available_width() - 8.0).clamp(150.0, 220.0);
                        let emu_h = emu_w * 1.35;

                        Frame::none()
                            .fill(Color32::from_rgb(0x06, 0x06, 0x06))
                            .rounding(Rounding::same(14.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x1E, 0x1E)))
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
                                painter.text(
                                    rect.center(),
                                    Align2::CENTER_CENTER,
                                    format!(
                                        "{}\n{}x{}\nLive Tap Ready",
                                        current_preset
                                            .name
                                            .split('(')
                                            .next()
                                            .unwrap_or(current_preset.name),
                                        current_preset.width,
                                        current_preset.height
                                    ),
                                    FontId::proportional(10.0),
                                    theme.text_silver,
                                );

                                if let Some(ref touch) = emu_state.last_touch {
                                    let tap_pt = pos2(
                                        rect.min.x + touch.norm_pos.x * rect.width(),
                                        rect.min.y + touch.norm_pos.y * rect.height(),
                                    );
                                    painter.circle_filled(tap_pt, 4.0, theme.accent_pop);
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
        "A" | "?" => Color32::from_rgb(0x4E, 0xC9, 0xB0),
        "D" => Color32::from_rgb(0xE0, 0x48, 0x48),
        _ => Color32::from_rgb(0xE5, 0xC0, 0x7B),
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
            // Stage / Unstage toggle
            if f.is_staged {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("-")
                                .color(theme.text_silver)
                                .strong()
                                .size(11.0),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Unstage file này")
                    .clicked()
                {
                    *action = Some(BeeBoxAction::UnstageFile(f.path.clone()));
                }
            } else {
                // Discard button with confirmation guard
                let is_confirming = confirm_discard_path.as_deref() == Some(f.path.as_str());
                if is_confirming {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Confirm?")
                                    .color(Color32::WHITE)
                                    .strong()
                                    .size(9.0),
                            )
                            .fill(theme.tag_red)
                            .rounding(Rounding::same(3.0)),
                        )
                        .on_hover_text("Click lần nữa để khôi phục file về HEAD")
                        .clicked()
                    {
                        *confirm_discard_path = None;
                        *action = Some(BeeBoxAction::DiscardFile(f.path.clone()));
                    }
                } else if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::TRASH)
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
                                .color( Color32::from_rgb(0x4E, 0xC9, 0xB0))
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

            // Direct Live Edit Button
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(Lucide::CODE)
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
