use crate::ai::{AgentTaskCard, KanbanColumn};
use crate::emulator::{EmulatorState, TouchRipple, ELEGANS_VIEWPORT_PRESETS};
use crate::git::{GitFileChange, WorktreeInfo};
use crate::theme::{
    CanhTheme, Lucide, CTL_DEFAULT, CTL_SM, FONT_CAPTION, FONT_FOOTNOTE, ROUND_CARD, ROUND_CTL,
    ROUND_PANEL, ROUND_XS, SPACE_1, SPACE_2, SPACE_3,
};
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
    pub active_tab: usize, // 0 = Git, 1 = Tasks, 2 = Preview
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
            active_tab: 0,
            show_branch_picker: false,
            show_worktree_manager: true,
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
    ui.spacing_mut().item_spacing = vec2(SPACE_2, SPACE_2);

    // ELEGANS FRAMED-SHEET RIGHT DOCK (16px ROUND_PANEL, 12px padding, 1px hairline border)
    Frame::none()
        .fill(theme.bg_field)
        .rounding(Rounding::same(ROUND_PANEL))
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::same(SPACE_3))
        .show(ui, |ui| {
            // 1. Top Inspector Title & Quick External Editor Link (28px height)
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(project_name)
                        .color(theme.text_paper)
                        .strong()
                        .size(FONT_FOOTNOTE),
                );
                ui.label(
                    egui::RichText::new(format!("· {}", commit_oid))
                        .color(theme.text_silver)
                        .monospace()
                        .size(FONT_CAPTION),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if icon_btn(
                        ui,
                        Lucide::EXTERNAL_LINK,
                        theme.text_silver,
                        Color32::TRANSPARENT,
                        "Mở thư mục Project bằng Cursor / VS Code / Zed",
                    )
                    .clicked()
                    {
                        action = Some(BeeBoxAction::OpenExternalEditor(None));
                    }
                });
            });

            ui.add_space(SPACE_2);

            // 2. Clean 3-Tab Segmented Switcher (Concentric 8px outer -> 4px inner, 24px CTL_SM)
            Frame::none()
                .fill(theme.surface_1)
                .rounding(Rounding::same(ROUND_CTL))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::same(SPACE_1))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(SPACE_1, 0.0);
                    ui.horizontal(|ui| {
                        let tab_w = (ui.available_width() - SPACE_2) / 3.0;
                        let tabs = [
                            (
                                0,
                                format!("{} Git · {}", Lucide::GIT_COMMIT, file_changes.len()),
                                "Quản lý Nhánh, Commit, Push/Pull & Thay đổi File",
                            ),
                            (
                                1,
                                format!("{} Tasks · {}", Lucide::KANBAN, kanban_tasks.len()),
                                "Tác vụ Kanban & Tiến trình Background Agent",
                            ),
                            (
                                2,
                                format!("{} Preview", Lucide::SMARTPHONE),
                                "Git Worktrees & Giả lập Mobile/Web Viewport",
                            ),
                        ];

                        for (idx, lbl, tip) in tabs {
                            let is_sel = beebox_state.active_tab == idx;
                            let btn = egui::Button::new(
                                egui::RichText::new(lbl)
                                    .color(if is_sel {
                                        theme.text_paper
                                    } else {
                                        theme.text_silver
                                    })
                                    .strong()
                                    .size(FONT_CAPTION),
                            )
                            .fill(if is_sel {
                                theme.surface_2
                            } else {
                                Color32::TRANSPARENT
                            })
                            .rounding(Rounding::same(ROUND_XS));

                            if ui
                                .add_sized(vec2(tab_w, CTL_SM), btn)
                                .on_hover_text(tip)
                                .clicked()
                            {
                                beebox_state.active_tab = idx;
                            }
                        }
                    });
                });

            ui.add_space(SPACE_2);

            // 3. Tab Content Scroll Area
            ScrollArea::vertical()
                .id_salt("elegans_beebox_inspector_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(SPACE_1, SPACE_1);
                    match beebox_state.active_tab {
                        0 => {
                            // =====================================================
                            // TAB 0: GIT SOURCE CONTROL (Branch, Commit, Changes)
                            // =====================================================
                            ui.horizontal(|ui| {
                                let branch_btn = egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{} {} {}",
                                        Lucide::GIT_BRANCH,
                                        branch_name,
                                        if beebox_state.show_branch_picker {
                                            Lucide::CHEVRON_DOWN
                                        } else {
                                            Lucide::CHEVRON_RIGHT
                                        }
                                    ))
                                    .color(theme.text_paper)
                                    .strong()
                                    .size(FONT_FOOTNOTE),
                                )
                                .fill(theme.surface_1)
                                .stroke(Stroke::new(1.0, theme.border))
                                .rounding(Rounding::same(ROUND_CTL));

                                if ui
                                    .add_sized(vec2(0.0, CTL_DEFAULT), branch_btn)
                                    .on_hover_text("Click để đổi nhánh hoặc tạo nhánh mới")
                                    .clicked()
                                {
                                    beebox_state.show_branch_picker =
                                        !beebox_state.show_branch_picker;
                                }

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if icon_btn(
                                        ui,
                                        Lucide::GIT_PR,
                                        theme.text_paper,
                                        theme.surface_1,
                                        "Tạo GitLab Merge Request cho nhánh này",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::OpenGitLabMr);
                                    }
                                });
                            });

                            if beebox_state.show_branch_picker {
                                ui.add_space(SPACE_1);
                                Frame::none()
                                    .fill(theme.surface_1)
                                    .rounding(Rounding::same(ROUND_CTL))
                                    .stroke(Stroke::new(1.0, theme.border))
                                    .inner_margin(Margin::same(SPACE_2))
                                    .show(ui, |ui| {
                                        for b in branches {
                                            let is_cur = b == branch_name;
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
                                                            .size(FONT_CAPTION),
                                                    )
                                                    .fill(Color32::TRANSPARENT),
                                                )
                                                .clicked()
                                                && !is_cur
                                            {
                                                beebox_state.show_branch_picker = false;
                                                action = Some(BeeBoxAction::SwitchBranch(b.clone()));
                                            }
                                        }

                                        ui.add_space(SPACE_1);
                                        ui.horizontal(|ui| {
                                            let w = (ui.available_width() - 36.0).max(60.0);
                                            Frame::none()
                                                .fill(theme.input_bg)
                                                .rounding(Rounding::same(ROUND_XS))
                                                .stroke(Stroke::new(1.0, theme.border))
                                                .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
                                                .show(ui, |ui| {
                                                    ui.add_sized(
                                                        vec2(w, 18.0),
                                                        egui::TextEdit::singleline(
                                                            &mut beebox_state.new_branch_name,
                                                        )
                                                        .hint_text("Tên nhánh mới...")
                                                        .frame(false)
                                                        .text_color(theme.text_paper),
                                                    );
                                                });
                                            if icon_btn(
                                                ui,
                                                Lucide::PLUS,
                                                theme.text_on_ink(),
                                                theme.text_paper,
                                                "Tạo & Checkout sang nhánh mới",
                                            )
                                            .clicked()
                                                && !beebox_state.new_branch_name.trim().is_empty()
                                            {
                                                let b = beebox_state
                                                    .new_branch_name
                                                    .trim()
                                                    .to_string();
                                                beebox_state.new_branch_name.clear();
                                                beebox_state.show_branch_picker = false;
                                                action = Some(BeeBoxAction::CreateBranch(b));
                                            }
                                        });
                                    });
                            }

                            ui.add_space(SPACE_2);

                            // Single-Level Commit Input & Ink Primary Commit CTA (No double-wrapped card)
                            Frame::none()
                                .fill(theme.input_bg)
                                .rounding(Rounding::same(ROUND_CTL))
                                .stroke(Stroke::new(1.0, theme.border))
                                .inner_margin(Margin::symmetric(SPACE_2, SPACE_2))
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(commit_message)
                                            .desired_rows(2)
                                            .desired_width(ui.available_width())
                                            .frame(false)
                                            .text_color(theme.text_paper)
                                            .hint_text("Nhập nội dung Commit (feat/fix)..."),
                                    );
                                });

                            ui.add_space(SPACE_1);

                            ui.horizontal(|ui| {
                                if icon_btn(
                                    ui,
                                    Lucide::WAND,
                                    theme.text_paper,
                                    theme.surface_1,
                                    "AI tự động viết Commit Message từ Diff hiện tại",
                                )
                                .clicked()
                                {
                                    action = Some(BeeBoxAction::AiGenerateCommitMsg);
                                }

                                // Elegans Ink Primary CTA: `theme.text_paper` fill with `theme.text_on_ink()` text
                                let commit_btn = egui::Button::new(
                                    egui::RichText::new(format!("{} Commit", Lucide::CHECK))
                                        .color(theme.text_on_ink())
                                        .strong()
                                        .size(FONT_CAPTION),
                                )
                                .fill(theme.text_paper)
                                .rounding(Rounding::same(ROUND_CTL));

                                if ui
                                    .add_sized(vec2(0.0, CTL_DEFAULT), commit_btn)
                                    .on_hover_text(
                                        "Commit các thay đổi (Tự động Stage nếu chưa Stage)",
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

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if icon_btn(
                                        ui,
                                        Lucide::ARCHIVE_RESTORE,
                                        theme.text_silver,
                                        theme.surface_1,
                                        "Git Stash Pop",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitStashPop);
                                    }
                                    if icon_btn(
                                        ui,
                                        Lucide::ARCHIVE,
                                        theme.text_silver,
                                        theme.surface_1,
                                        "Git Stash",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitStashSave);
                                    }
                                    if icon_btn(
                                        ui,
                                        Lucide::DOWNLOAD,
                                        theme.text_paper,
                                        theme.surface_1,
                                        "Git Pull Rebase",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitPull);
                                    }
                                    if icon_btn(
                                        ui,
                                        Lucide::UPLOAD,
                                        theme.text_paper,
                                        theme.surface_1,
                                        "Git Push lên origin",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::GitPush);
                                    }
                                });
                            });

                            ui.add_space(SPACE_3);

                            // Changes Section (Sentence case, tabular counts)
                            let total_add: usize = file_changes.iter().map(|c| c.added_lines).sum();
                            let total_rem: usize =
                                file_changes.iter().map(|c| c.removed_lines).sum();
                            let staged_files: Vec<&GitFileChange> =
                                file_changes.iter().filter(|c| c.is_staged).collect();
                            let unstaged_files: Vec<&GitFileChange> =
                                file_changes.iter().filter(|c| !c.is_staged).collect();

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Thay đổi ({})",
                                        file_changes.len()
                                    ))
                                    .color(theme.text_paper)
                                    .strong()
                                    .size(FONT_FOOTNOTE),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("-{}", total_rem))
                                            .color(theme.danger)
                                            .monospace()
                                            .size(FONT_CAPTION),
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("+{}", total_add))
                                            .color(theme.success)
                                            .monospace()
                                            .size(FONT_CAPTION),
                                    );
                                    if !unstaged_files.is_empty()
                                        && icon_btn(
                                            ui,
                                            Lucide::PLUS,
                                            theme.success,
                                            Color32::TRANSPARENT,
                                            "Stage tất cả file",
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
                                            Color32::TRANSPARENT,
                                            "Unstage tất cả file",
                                        )
                                        .clicked()
                                    {
                                        action = Some(BeeBoxAction::UnstageAll);
                                    }
                                });
                            });

                            ui.add_space(SPACE_1);
                            if file_changes.is_empty() {
                                // Elegans Smart Empty State: Single quiet muted line
                                ui.label(
                                    egui::RichText::new("Không có thay đổi chưa commit")
                                        .color(theme.text_silver)
                                        .size(FONT_CAPTION),
                                );
                            } else {
                                if !staged_files.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Đã stage ({})",
                                            staged_files.len()
                                        ))
                                        .color(theme.success)
                                        .strong()
                                        .size(FONT_CAPTION),
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
                                    ui.add_space(SPACE_2);
                                }

                                if !unstaged_files.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Chưa stage ({})",
                                            unstaged_files.len()
                                        ))
                                        .color(theme.text_silver)
                                        .strong()
                                        .size(FONT_CAPTION),
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
                        1 => {
                            // =====================================================
                            // TAB 1: KANBAN TASKS & BACKGROUND AGENT LINEAGE
                            // =====================================================
                            let active_tasks: Vec<&AgentTaskCard> = kanban_tasks
                                .iter()
                                .filter(|t| t.column == KanbanColumn::InProgress || t.is_running_bg)
                                .collect();

                            ui.label(
                                egui::RichText::new(format!(
                                    "Tiến độ tác vụ ({} đang chạy / {} tổng)",
                                    active_tasks.len(),
                                    kanban_tasks.len()
                                ))
                                .color(theme.text_silver)
                                .strong()
                                .size(FONT_CAPTION),
                            );
                            ui.add_space(SPACE_2);

                            for task in kanban_tasks {
                                let dot_col = match task.column {
                                    KanbanColumn::InProgress => theme.accent_pop,
                                    KanbanColumn::Verifying => theme.info,
                                    KanbanColumn::Done => theme.success,
                                    KanbanColumn::Backlog => theme.text_silver,
                                };

                                Frame::none()
                                    .fill(theme.surface_1)
                                    .rounding(Rounding::same(ROUND_CTL))
                                    .stroke(Stroke::new(1.0, theme.border))
                                    .inner_margin(Margin::symmetric(SPACE_2, SPACE_2))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let (r, _) = ui.allocate_exact_size(
                                                vec2(SPACE_2, SPACE_2),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(r.center(), 3.0, dot_col);

                                            let short_title: String =
                                                task.title.chars().take(24).collect();
                                            ui.label(
                                                egui::RichText::new(short_title)
                                                    .color(theme.text_paper)
                                                    .strong()
                                                    .size(FONT_CAPTION),
                                            );

                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    if !task.is_running_bg
                                                        && task.column != KanbanColumn::Done
                                                        && icon_btn(
                                                            ui,
                                                            Lucide::PLAY,
                                                            theme.text_paper,
                                                            theme.chip_bg,
                                                            "Chạy tác vụ này dưới nền (Background Agent)",
                                                        )
                                                        .clicked()
                                                    {
                                                        action = Some(
                                                            BeeBoxAction::RunBackgroundTask(
                                                                task.id.clone(),
                                                            ),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            egui::RichText::new(
                                                                task.column.title(),
                                                            )
                                                            .color(dot_col)
                                                            .size(FONT_CAPTION),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                    });
                                ui.add_space(SPACE_1);
                            }
                        }
                        _ => {
                            // =====================================================
                            // TAB 2: GIT WORKTREES & LIVE DEVICE VIEWPORT PREVIEW
                            // =====================================================
                            ui.label(
                                egui::RichText::new(format!(
                                    "Nhánh Git Worktree ({})",
                                    worktrees.len()
                                ))
                                .color(theme.text_silver)
                                .strong()
                                .size(FONT_CAPTION),
                            );
                            ui.add_space(SPACE_1);

                            Frame::none()
                                .fill(theme.surface_1)
                                .rounding(Rounding::same(ROUND_CTL))
                                .stroke(Stroke::new(1.0, theme.border))
                                .inner_margin(Margin::same(SPACE_2))
                                .show(ui, |ui| {
                                    for (idx, wt) in worktrees.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            let col = if wt.is_head {
                                                theme.text_paper
                                            } else {
                                                theme.text_silver
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
                                                        .size(FONT_CAPTION),
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
                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
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
                                                },
                                            );
                                        });
                                    }
                                    ui.add_space(SPACE_1);
                                    ui.horizontal(|ui| {
                                        let w = (ui.available_width() - 36.0).max(60.0);
                                        Frame::none()
                                            .fill(theme.input_bg)
                                            .rounding(Rounding::same(ROUND_XS))
                                            .stroke(Stroke::new(1.0, theme.border))
                                            .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
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
                                            theme.text_on_ink(),
                                            theme.text_paper,
                                            "Khởi tạo Git Worktree mới",
                                        )
                                        .clicked()
                                        {
                                            let b = if beebox_state
                                                .new_worktree_branch
                                                .trim()
                                                .is_empty()
                                            {
                                                format!("feat/wt-{}", worktrees.len() + 1)
                                            } else {
                                                beebox_state
                                                    .new_worktree_branch
                                                    .trim()
                                                    .to_string()
                                            };
                                            beebox_state.new_worktree_branch.clear();
                                            action = Some(BeeBoxAction::SetupWorktree(b));
                                        }
                                    });
                                });

                            ui.add_space(SPACE_3);

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Giả lập thiết bị")
                                        .color(theme.text_silver)
                                        .strong()
                                        .size(FONT_CAPTION),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if icon_btn(
                                        ui,
                                        Lucide::CAMERA,
                                        theme.text_paper,
                                        theme.surface_1,
                                        "Chụp & đồng bộ khung hình ADB thực tế",
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::CaptureAdbFrame);
                                    }
                                });
                            });

                            ui.add_space(SPACE_1);
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
                                            theme.text_on_ink()
                                        } else {
                                            theme.text_silver
                                        },
                                        if is_sel {
                                            theme.text_paper
                                        } else {
                                            theme.surface_1
                                        },
                                        p.name,
                                    )
                                    .clicked()
                                    {
                                        action = Some(BeeBoxAction::SwitchPreset(idx));
                                    }
                                }
                            });

                            ui.add_space(SPACE_1);
                            ui.horizontal(|ui| {
                                let url_w = (ui.available_width() - 36.0).max(80.0);
                                Frame::none()
                                    .fill(theme.input_bg)
                                    .rounding(Rounding::same(ROUND_XS))
                                    .stroke(Stroke::new(1.0, theme.border))
                                    .inner_margin(Margin::symmetric(SPACE_2, SPACE_1))
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
                                    theme.text_paper,
                                    theme.surface_1,
                                    "Mở Preview URL trên trình duyệt cách ly",
                                )
                                .clicked()
                                {
                                    action = Some(BeeBoxAction::OpenPreviewUrl(
                                        emu_state.preview_url.clone(),
                                    ));
                                }
                            });

                            ui.add_space(SPACE_2);
                            let current_preset =
                                ELEGANS_VIEWPORT_PRESETS[emu_state.active_preset_idx];
                            let aspect =
                                (current_preset.height as f32 / current_preset.width as f32)
                                    .clamp(0.65, 1.65);
                            let emu_w = (ui.available_width() - SPACE_2).clamp(150.0, 230.0);
                            let emu_h = (emu_w * aspect).clamp(120.0, 300.0);

                            Frame::none()
                                .fill(theme.code_bg)
                                .rounding(Rounding::same(ROUND_CARD))
                                .stroke(Stroke::new(1.0, theme.border_strong))
                                .inner_margin(Margin::same(SPACE_2))
                                .show(ui, |ui| {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        vec2(emu_w, emu_h),
                                        egui::Sense::click_and_drag(),
                                    );
                                    if resp.clicked() {
                                        if let Some(pos) = resp.interact_pointer_pos() {
                                            let rel_x = ((pos.x - rect.min.x) / rect.width())
                                                .clamp(0.0, 1.0);
                                            let rel_y = ((pos.y - rect.min.y) / rect.height())
                                                .clamp(0.0, 1.0);
                                            let tap_x =
                                                (rel_x * current_preset.width as f32) as u32;
                                            let tap_y =
                                                (rel_y * current_preset.height as f32) as u32;
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
                                    if let Some(ref frame_img) = emu_state.current_frame {
                                        let tex = ui.ctx().load_texture(
                                            "adb_live_frame",
                                            frame_img.clone(),
                                            TextureOptions::LINEAR,
                                        );
                                        painter.image(
                                            tex.id(),
                                            rect,
                                            egui::Rect::from_min_max(
                                                pos2(0.0, 0.0),
                                                pos2(1.0, 1.0),
                                            ),
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
                                            FontId::proportional(FONT_CAPTION),
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
                .size(FONT_CAPTION),
        );

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new(file_name)
                        .color(theme.text_paper)
                        .size(FONT_CAPTION),
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
            if f.is_staged {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(Lucide::MINUS)
                                .color(theme.warning)
                                .size(FONT_CAPTION),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Unstage file này")
                    .clicked()
                {
                    *action = Some(BeeBoxAction::UnstageFile(f.path.clone()));
                }
            } else {
                let is_confirming = confirm_discard_path.as_deref() == Some(f.path.as_str());
                if is_confirming {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(Lucide::CHECK)
                                    .color(Color32::WHITE)
                                    .size(FONT_CAPTION),
                            )
                            .fill(theme.danger)
                            .rounding(Rounding::same(ROUND_XS)),
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
                                .size(FONT_CAPTION),
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
                                .size(FONT_CAPTION),
                        )
                        .fill(Color32::TRANSPARENT),
                    )
                    .on_hover_text("Stage file này")
                    .clicked()
                {
                    *action = Some(BeeBoxAction::StageFile(f.path.clone()));
                }
            }

            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new(Lucide::EDIT)
                            .color(theme.text_silver)
                            .size(FONT_CAPTION),
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
