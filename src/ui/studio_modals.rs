use crate::ai::{AgentAvatarState, AvatarManager, MemoryReaper, QuotedContext, ALL_AGENTS};
use crate::config::{DiscoveredAgent, DiscoveredSkill, ProjectManager};
use crate::git::{DiffHunk, GitFileChange};
use crate::theme::{CanhTheme, Lucide, SharedTicker};
use crate::ui::layout_state::{CenterViewMode, LayoutState};
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};
use std::path::PathBuf;

pub enum StudioModalAction {
    CloseToChat,
    SwitchProject(PathBuf),
    AddNewProject(String),
    TogglePinProject(PathBuf),
    QuoteDiffToChat(QuotedContext),
    OpenFileInEditor {
        path: String,
        edit_mode: bool,
    },
    SaveEditedFile {
        path: String,
        content: String,
    },
    StageFileFromEditor(String),
    UnstageFileFromEditor(String),
    DiscardFileFromEditor(String),
    OpenExternalEditor(Option<String>),
    SaveSkill {
        name: String,
        desc: String,
        body: String,
    },
    SaveAgent {
        name: String,
        role: String,
        prompt: String,
    },
    DeleteSkill(PathBuf),
    DeleteAgent(PathBuf),
    InsertSkillIntoChat(String),
    OpenDedicatedAgentChat(String),
    SelectTheme(CanhTheme),
}

pub struct StudioEditorState {
    pub new_project_path: String,
    pub skill_name: String,
    pub skill_desc: String,
    pub skill_body: String,
    pub agent_name: String,
    pub agent_role: String,
    pub agent_prompt: String,
    pub is_editing_agent: bool,

    // Phase A.4: Live File & Diff Editor State
    pub active_file_path: String,
    pub file_editor_buffer: String,
    pub file_editor_dirty: bool,
    pub diff_edit_mode: bool, // false = Diff View, true = Live Code Editor
    pub file_filter_query: String,
    pub status_notice: Option<String>,
}

impl Default for StudioEditorState {
    fn default() -> Self {
        Self {
            new_project_path: "/Volumes/Data/".to_string(),
            skill_name: "vibe-ui-audit".to_string(),
            skill_desc: "Tự động kiểm tra giao diện & chống AI slop".to_string(),
            skill_body:
                "Kiểm tra độ tương phản, font Be Vietnam Pro và responsive 3 breakpoints."
                    .to_string(),
            agent_name: "fe_custom_specialist".to_string(),
            agent_role: "Chuyên gia tối ưu Rust WGPU & GitLab CI".to_string(),
            agent_prompt: "Thực thi code sạch, gọn theo nguyên tắc Ponytail Zero-Bloat."
                .to_string(),
            is_editing_agent: false,
            active_file_path: String::new(),
            file_editor_buffer: String::new(),
            file_editor_dirty: false,
            diff_edit_mode: false,
            file_filter_query: String::new(),
            status_notice: None,
        }
    }
}

/// Phase A.4: Dual-Mode Interactive Git Diff Viewer & Live Code Editor
pub fn render_git_diff_viewer(
    ui: &mut Ui,
    theme: &CanhTheme,
    diff_hunk: &Option<DiffHunk>,
    file_changes: &[GitFileChange],
    workspace_files: &[String],
    editor: &mut StudioEditorState,
) -> Option<StudioModalAction> {
    let mut action = None;

    // Keyboard shortcut Cmd+S / Ctrl+S to save directly in Live Editor
    let save_shortcut = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S));
    if save_shortcut && !editor.active_file_path.is_empty() && editor.diff_edit_mode {
        editor.file_editor_dirty = false;
        editor.status_notice = Some(format!("✓ Đã lưu file {}", editor.active_file_path));
        action = Some(StudioModalAction::SaveEditedFile {
            path: editor.active_file_path.clone(),
            content: editor.file_editor_buffer.clone(),
        });
    }

    ui.vertical(|ui| {
        // Top Header Bar
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} GIT DIFF & LIVE CODE EDITOR",
                    Lucide::CODE
                ))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
            );

            if let Some(ref notice) = editor.status_notice {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(notice)
                        .color(Color32::from_rgb(0x4E, 0xC9, 0xB0))
                        .size(11.0),
                );
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!("{} Quay lại Chat", Lucide::X))
                                .color(theme.text_paper)
                                .size(11.0),
                        )
                        .fill(Color32::from_rgb(0x18, 0x18, 0x18)),
                    )
                    .clicked()
                {
                    action = Some(StudioModalAction::CloseToChat);
                }

                if !editor.active_file_path.is_empty() {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("External IDE")
                                    .color(theme.text_silver)
                                    .size(10.5),
                            )
                            .fill(Color32::from_rgb(0x18, 0x18, 0x18)),
                        )
                        .on_hover_text("Mở file này bằng Cursor / VS Code")
                        .clicked()
                    {
                        action = Some(StudioModalAction::OpenExternalEditor(Some(
                            editor.active_file_path.clone(),
                        )));
                    }
                }

                if let Some(ref hunk) = diff_hunk {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Quote Diff vào Chat AI",
                                    Lucide::QUOTE
                                ))
                                .color(theme.accent_pop)
                                .strong()
                                .size(11.0),
                            )
                            .fill(Color32::from_rgb(0x1C, 0x14, 0x12)),
                        )
                        .clicked()
                    {
                        let snippet: String = hunk
                            .lines
                            .iter()
                            .filter(|l| l.tag != ' ')
                            .take(30)
                            .map(|l| format!("{} {}", l.tag, l.content))
                            .collect::<Vec<_>>()
                            .join("\n");
                        action = Some(StudioModalAction::QuoteDiffToChat(QuotedContext {
                            source_msg_id: hunk.file_path.clone(),
                            author_label: format!("Git Diff ({})", hunk.file_path),
                            snippet: if snippet.is_empty() {
                                editor
                                    .file_editor_buffer
                                    .lines()
                                    .take(25)
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            } else {
                                snippet
                            },
                            is_quote_block: true,
                        }));
                    }
                }
            });
        });

        ui.add_space(8.0);

        // Two-Column Layout: Left = Changed Files & Project Explorer | Right = Diff / Code Editor
        let total_h = (ui.available_height() - 10.0).max(200.0);
        ui.horizontal(|ui| {
            // LEFT PANE: Changed Files + Project File Picker
            Frame::none()
                .fill(Color32::from_rgb(0x0D, 0x0D, 0x0D))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    ui.set_width(220.0);
                    ui.set_min_height(total_h);

                    ui.vertical(|ui| {
                        // Filter search input
                        Frame::none()
                            .fill(Color32::from_rgb(0x14, 0x14, 0x14))
                            .rounding(Rounding::same(5.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::symmetric(6.0, 3.0))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut editor.file_filter_query)
                                        .hint_text("Filter files...")
                                        .frame(false)
                                        .text_color(theme.text_paper),
                                );
                            });

                        ui.add_space(6.0);

                        ScrollArea::vertical()
                            .id_salt("editor_left_files_scroll")
                            .max_height(total_h - 40.0)
                            .show(ui, |ui| {
                                let q = editor.file_filter_query.trim().to_lowercase();

                                // 1. Changed Files Section
                                ui.label(
                                    egui::RichText::new(format!(
                                        "GIT CHANGES ({})",
                                        file_changes.len()
                                    ))
                                    .color(theme.accent_pop)
                                    .strong()
                                    .size(10.0),
                                );
                                ui.add_space(3.0);

                                if file_changes.is_empty() {
                                    ui.label(
                                        egui::RichText::new("Không có file thay đổi")
                                            .color(theme.text_silver)
                                            .size(10.0),
                                    );
                                } else {
                                    for fc in file_changes {
                                        if !q.is_empty() && !fc.path.to_lowercase().contains(&q) {
                                            continue;
                                        }
                                        let is_sel = editor.active_file_path == fc.path;
                                        let bg = if is_sel {
                                            Color32::from_rgb(0x1C, 0x1C, 0x1C)
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        Frame::none()
                                            .fill(bg)
                                            .rounding(Rounding::same(4.0))
                                            .inner_margin(Margin::symmetric(4.0, 2.0))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    let st_col = match fc.status.as_str() {
                                                        "A" | "?" => {
                                                            Color32::from_rgb(0x4E, 0xC9, 0xB0)
                                                        }
                                                        "D" => Color32::from_rgb(0xE0, 0x48, 0x48),
                                                        _ => Color32::from_rgb(0xE5, 0xC0, 0x7B),
                                                    };
                                                    ui.label(
                                                        egui::RichText::new(&fc.status)
                                                            .color(st_col)
                                                            .monospace()
                                                            .strong()
                                                            .size(10.0),
                                                    );
                                                    let fname = fc
                                                        .path
                                                        .rsplit('/')
                                                        .next()
                                                        .unwrap_or(&fc.path);
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                egui::RichText::new(fname)
                                                                    .color(if is_sel {
                                                                        theme.accent_pop
                                                                    } else {
                                                                        theme.text_paper
                                                                    })
                                                                    .size(10.8),
                                                            )
                                                            .fill(Color32::TRANSPARENT),
                                                        )
                                                        .on_hover_text(&fc.path)
                                                        .clicked()
                                                    {
                                                        action = Some(
                                                            StudioModalAction::OpenFileInEditor {
                                                                path: fc.path.clone(),
                                                                edit_mode: editor.diff_edit_mode,
                                                            },
                                                        );
                                                    }
                                                });
                                            });
                                    }
                                }

                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(6.0);

                                // 2. All Workspace Files Section
                                ui.label(
                                    egui::RichText::new(format!(
                                        "WORKSPACE FILES ({})",
                                        workspace_files.len()
                                    ))
                                    .color(theme.text_silver)
                                    .strong()
                                    .size(10.0),
                                );
                                ui.add_space(3.0);

                                for wf in workspace_files.iter().take(120) {
                                    if !q.is_empty() && !wf.to_lowercase().contains(&q) {
                                        continue;
                                    }
                                    let is_sel = editor.active_file_path == *wf;
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(wf)
                                                    .color(if is_sel {
                                                        theme.accent_pop
                                                    } else {
                                                        theme.text_silver
                                                    })
                                                    .size(10.2),
                                            )
                                            .fill(Color32::TRANSPARENT),
                                        )
                                        .clicked()
                                    {
                                        action = Some(StudioModalAction::OpenFileInEditor {
                                            path: wf.clone(),
                                            edit_mode: true,
                                        });
                                    }
                                }
                            });
                    });
                });

            ui.add_space(8.0);

            // RIGHT PANE: Main Diff / Code Editor Canvas
            let right_w = ui.available_width().max(200.0);
            Frame::none()
                .fill(Color32::from_rgb(0x0D, 0x0D, 0x0D))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.set_width(right_w - 20.0);
                    ui.set_min_height(total_h);

                    ui.vertical(|ui| {
                        // Editor Toolbar: File Name + Mode Switcher + Stage / Discard / Save
                        ui.horizontal(|ui| {
                            let title_path = if !editor.active_file_path.is_empty() {
                                editor.active_file_path.as_str()
                            } else if let Some(ref h) = diff_hunk {
                                h.file_path.as_str()
                            } else {
                                "Chưa chọn file"
                            };

                            ui.label(
                                egui::RichText::new(format!("{} {}", Lucide::FILE_TEXT, title_path))
                                    .color(theme.text_paper)
                                    .strong()
                                    .size(12.0),
                            );

                            if editor.file_editor_dirty {
                                ui.label(
                                    egui::RichText::new("● Unsaved")
                                        .color(theme.accent_pop)
                                        .strong()
                                        .size(10.5),
                                );
                            }

                            if let Some(ref hunk) = diff_hunk {
                                ui.label(
                                    egui::RichText::new(format!("+{}", hunk.added_count))
                                        .color(Color32::from_rgb(0x4E, 0xC9, 0xB0))
                                        .monospace()
                                        .strong()
                                        .size(11.0),
                                );
                                ui.label(
                                    egui::RichText::new(format!("-{}", hunk.removed_count))
                                        .color(Color32::from_rgb(0xE0, 0x48, 0x48))
                                        .monospace()
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Save Button
                                if !editor.active_file_path.is_empty() {
                                    let save_btn = egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} Save File (⌘S)",
                                            Lucide::SAVE
                                        ))
                                        .color(Color32::BLACK)
                                        .strong()
                                        .size(10.5),
                                    )
                                    .fill(if editor.file_editor_dirty {
                                        theme.accent_pop
                                    } else {
                                        Color32::from_rgb(0x4E, 0xC9, 0xB0)
                                    })
                                    .rounding(Rounding::same(5.0));

                                    if ui.add(save_btn).clicked() {
                                        editor.file_editor_dirty = false;
                                        editor.status_notice = Some(format!(
                                            "✓ Đã lưu {}",
                                            editor.active_file_path
                                        ));
                                        action = Some(StudioModalAction::SaveEditedFile {
                                            path: editor.active_file_path.clone(),
                                            content: editor.file_editor_buffer.clone(),
                                        });
                                    }

                                    // Stage / Discard buttons if file is in file_changes
                                    if let Some(fc) = file_changes
                                        .iter()
                                        .find(|c| c.path == editor.active_file_path)
                                    {
                                        if fc.is_staged {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("Unstage")
                                                            .color(theme.text_silver)
                                                            .size(10.5),
                                                    )
                                                    .fill(Color32::from_rgb(0x1A, 0x1A, 0x1A)),
                                                )
                                                .clicked()
                                            {
                                                action =
                                                    Some(StudioModalAction::UnstageFileFromEditor(
                                                        fc.path.clone(),
                                                    ));
                                            }
                                        } else {
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("+ Stage")
                                                            .color(Color32::from_rgb(
                                                                0x4E, 0xC9, 0xB0,
                                                            ))
                                                            .size(10.5),
                                                    )
                                                    .fill(Color32::from_rgb(0x16, 0x22, 0x1E)),
                                                )
                                                .clicked()
                                            {
                                                action =
                                                    Some(StudioModalAction::StageFileFromEditor(
                                                        fc.path.clone(),
                                                    ));
                                            }
                                        }
                                    }
                                }

                                ui.add_space(8.0);

                                // Mode Switcher: [Live Code Editor] vs [Diff View]
                                let edit_btn = egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{} Live Code Editor",
                                        Lucide::CODE
                                    ))
                                    .color(if editor.diff_edit_mode {
                                        Color32::BLACK
                                    } else {
                                        theme.text_paper
                                    })
                                    .strong()
                                    .size(10.5),
                                )
                                .fill(if editor.diff_edit_mode {
                                    theme.accent_pop
                                } else {
                                    Color32::from_rgb(0x1A, 0x1A, 0x1A)
                                })
                                .rounding(Rounding::same(5.0));

                                if ui.add(edit_btn).clicked() {
                                    editor.diff_edit_mode = true;
                                }

                                let diff_btn = egui::Button::new(
                                    egui::RichText::new(format!("{} Diff View", Lucide::LAYERS))
                                        .color(if !editor.diff_edit_mode {
                                            Color32::BLACK
                                        } else {
                                            theme.text_paper
                                        })
                                        .strong()
                                        .size(10.5),
                                )
                                .fill(if !editor.diff_edit_mode {
                                    theme.accent_pop
                                } else {
                                    Color32::from_rgb(0x1A, 0x1A, 0x1A)
                                })
                                .rounding(Rounding::same(5.0));

                                if ui.add(diff_btn).clicked() {
                                    editor.diff_edit_mode = false;
                                }
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Main Content Body
                        let body_h = (total_h - 55.0).max(150.0);
                        if editor.diff_edit_mode {
                            // LIVE CODE EDITOR MODE
                            if editor.active_file_path.is_empty() {
                                ui.label(
                                    egui::RichText::new(
                                        "Chọn một file từ cột bên trái để bắt đầu chỉnh sửa trực tiếp.",
                                    )
                                    .color(theme.text_silver),
                                );
                            } else {
                                ScrollArea::vertical()
                                    .id_salt("live_code_editor_scroll")
                                    .auto_shrink([false, false])
                                    .max_height(body_h)
                                    .show(ui, |ui| {
                                        let line_count =
                                            editor.file_editor_buffer.lines().count().max(1);
                                        ui.horizontal_top(|ui| {
                                            // Line numbers gutter
                                            let gutter: String = (1..=line_count)
                                                .map(|n| format!("{:>4}", n))
                                                .collect::<Vec<_>>()
                                                .join("\n");
                                            ui.label(
                                                egui::RichText::new(gutter)
                                                    .color(Color32::from_rgb(0x55, 0x55, 0x55))
                                                    .monospace()
                                                    .size(11.5),
                                            );
                                            ui.add_space(8.0);

                                            // Dark Code Editor Area
                                            let resp = ui.add(
                                                egui::TextEdit::multiline(
                                                    &mut editor.file_editor_buffer,
                                                )
                                                .code_editor()
                                                .desired_rows(30)
                                                .desired_width(ui.available_width())
                                                .frame(false)
                                                .text_color(theme.text_paper),
                                            );
                                            if resp.changed() {
                                                editor.file_editor_dirty = true;
                                            }
                                        });
                                    });
                            }
                        } else {
                            // UNIFIED DIFF VIEW MODE
                            if let Some(ref hunk) = diff_hunk {
                                ScrollArea::vertical()
                                    .id_salt("inline_diff_lines_scroll")
                                    .auto_shrink([false, false])
                                    .max_height(body_h)
                                    .show(ui, |ui| {
                                        for line in &hunk.lines {
                                            let (bg, fg) = match line.tag {
                                                '+' => (
                                                    Color32::from_rgba_unmultiplied(
                                                        0x1E, 0x3A, 0x24, 110,
                                                    ),
                                                    Color32::from_rgb(0x78, 0xE3, 0x99),
                                                ),
                                                '-' => (
                                                    Color32::from_rgba_unmultiplied(
                                                        0x46, 0x1A, 0x1E, 110,
                                                    ),
                                                    Color32::from_rgb(0xF0, 0x80, 0x88),
                                                ),
                                                _ => (Color32::TRANSPARENT, theme.text_silver),
                                            };
                                            Frame::none().fill(bg).show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    let old_ln = line
                                                        .old_line
                                                        .map(|n| format!("{:>4}", n))
                                                        .unwrap_or_else(|| "    ".to_string());
                                                    let new_ln = line
                                                        .new_line
                                                        .map(|n| format!("{:>4}", n))
                                                        .unwrap_or_else(|| "    ".to_string());
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "{} {} {}",
                                                            old_ln, new_ln, line.tag
                                                        ))
                                                        .color(Color32::from_rgb(0x66, 0x66, 0x66))
                                                        .monospace()
                                                        .size(11.0),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(&line.content)
                                                            .color(fg)
                                                            .monospace()
                                                            .size(11.5),
                                                    );
                                                });
                                            });
                                        }
                                    });
                            } else {
                                ui.label(
                                    egui::RichText::new(
                                        "Chọn một file trong danh sách GIT CHANGES hoặc WORKSPACE FILES bên trái để xem Diff hoặc sửa code trực tiếp.",
                                    )
                                    .color(theme.text_silver),
                                );
                            }
                        }
                    });
                });
        });
    });
    action
}

/// Phase 3: Multi-Project Workspace Manager
pub fn render_projects_manager(
    ui: &mut Ui,
    theme: &CanhTheme,
    project_mgr: &ProjectManager,
    editor: &mut StudioEditorState,
) -> Option<StudioModalAction> {
    let mut action = None;
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} MULTI-PROJECT & WORKSPACE MANAGER",
                    Lucide::FOLDER
                ))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("✕ Quay lại Chat").clicked() {
                    action = Some(StudioModalAction::CloseToChat);
                }
            });
        });

        ui.add_space(8.0);
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("+ Mở hoặc thêm đường dẫn Project:")
                            .color(theme.text_paper)
                            .size(11.5),
                    );
                    let w = (ui.available_width() - 120.0).max(140.0);
                    ui.add_sized(
                        vec2(w, 24.0),
                        egui::TextEdit::singleline(&mut editor.new_project_path)
                            .hint_text("/Volumes/Data/ten-du-an"),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Mở Project")
                                    .color(Color32::BLACK)
                                    .strong(),
                            )
                            .fill(theme.accent_pop),
                        )
                        .clicked()
                    {
                        action = Some(StudioModalAction::AddNewProject(
                            editor.new_project_path.clone(),
                        ));
                    }
                });
            });

        ui.add_space(10.0);
        ScrollArea::vertical()
            .id_salt("projects_registry_scroll")
            .show(ui, |ui| {
                for proj in &project_mgr.projects {
                    let is_active = proj.path == project_mgr.active_project;
                    Frame::none()
                        .fill(if is_active {
                            Color32::from_rgb(0x16, 0x16, 0x16)
                        } else {
                            Color32::from_rgb(0x0E, 0x0E, 0x0E)
                        })
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(
                            1.0,
                            if is_active {
                                theme.accent_pop
                            } else {
                                theme.border
                            },
                        ))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} {}",
                                        if proj.pinned {
                                            Lucide::PIN
                                        } else {
                                            Lucide::FOLDER
                                        },
                                        proj.name
                                    ))
                                    .color(if is_active {
                                        theme.accent_pop
                                    } else {
                                        theme.text_paper
                                    })
                                    .strong()
                                    .size(12.5),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "({} {})",
                                        Lucide::GIT_BRANCH,
                                        proj.last_branch
                                    ))
                                    .color(theme.text_silver)
                                    .size(11.0),
                                );
                                ui.label(
                                    egui::RichText::new(proj.path.to_string_lossy())
                                        .color(theme.text_silver)
                                        .size(10.5),
                                );

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if !is_active
                                        && ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("Chuyển sang Project này")
                                                        .color(Color32::BLACK)
                                                        .strong()
                                                        .size(10.5),
                                                )
                                                .fill(theme.accent_pop),
                                            )
                                            .clicked()
                                    {
                                        action = Some(StudioModalAction::SwitchProject(
                                            proj.path.clone(),
                                        ));
                                    } else if is_active {
                                        ui.label(
                                            egui::RichText::new("● Đang mở")
                                                .color(theme.accent_pop)
                                                .strong()
                                                .size(10.5),
                                        );
                                    }

                                    if ui
                                        .small_button(if proj.pinned {
                                            "Bỏ ghim"
                                        } else {
                                            "Ghim"
                                        })
                                        .clicked()
                                    {
                                        action = Some(StudioModalAction::TogglePinProject(
                                            proj.path.clone(),
                                        ));
                                    }
                                });
                            });
                        });
                    ui.add_space(6.0);
                }
            });
    });
    action
}

/// Phase 7: Agent & Skill Studio (Create, Edit, Delete, Preview Animated Avatars & Launch 1-1 Chat)
pub fn render_agent_skill_studio(
    ui: &mut Ui,
    theme: &CanhTheme,
    elapsed_secs: f32,
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
    editor: &mut StudioEditorState,
) -> Option<StudioModalAction> {
    let mut action = None;
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} AGENT & SKILL STUDIO — SWARM & CUSTOM SUBAGENTS",
                    Lucide::BOT
                ))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("✕ Quay lại Chat").clicked() {
                    action = Some(StudioModalAction::CloseToChat);
                }
            });
        });

        ui.add_space(8.0);

        // 1. Core Swarm Agents Gallery with 1-on-1 Dedicated Chat Button
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(
                        "Core Swarm Agents (Click 'Chat 1-1' để mở phiên chat chuyên biệt với từng Agent):",
                    )
                    .color(theme.text_silver)
                    .strong()
                    .size(11.0),
                );
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    for &ag in ALL_AGENTS {
                        Frame::none()
                            .fill(Color32::from_rgb(0x17, 0x17, 0x17))
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x22, 0x22)))
                            .inner_margin(Margin::symmetric(10.0, 6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    AvatarManager::render_animated_avatar(
                                        ui,
                                        ag,
                                        AgentAvatarState::Answering,
                                        30.0,
                                        elapsed_secs,
                                        theme.accent_pop,
                                    );
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{} {}",
                                                ag.icon(),
                                                ag.name()
                                            ))
                                            .color(theme.accent_pop)
                                            .strong()
                                            .size(11.0),
                                        );
                                        ui.label(
                                            egui::RichText::new(ag.title())
                                                .color(theme.text_paper)
                                                .size(10.0),
                                        );
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new(format!(
                                                        "{} Chat 1-1",
                                                        Lucide::MESSAGE_SQUARE
                                                    ))
                                                    .color(Color32::BLACK)
                                                    .strong()
                                                    .size(9.5),
                                                )
                                                .fill(theme.accent_pop)
                                                .rounding(Rounding::same(4.0)),
                                            )
                                            .clicked()
                                        {
                                            action = Some(
                                                StudioModalAction::OpenDedicatedAgentChat(
                                                    ag.name().to_string(),
                                                ),
                                            );
                                        }
                                    });
                                });
                            });
                    }
                });
            });

        ui.add_space(10.0);

        // 2. Editor Form to Create / Update Skill or Custom Agent
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            !editor.is_editing_agent,
                            format!("{} Tạo / Sửa Skill (SKILL.md)", Lucide::CODE),
                        )
                        .clicked()
                    {
                        editor.is_editing_agent = false;
                    }
                    if ui
                        .selectable_label(
                            editor.is_editing_agent,
                            format!("{} Tạo / Sửa Subagent (.claude/agents/*.md)", Lucide::BOT),
                        )
                        .clicked()
                    {
                        editor.is_editing_agent = true;
                    }
                });
                ui.add_space(6.0);

                if !editor.is_editing_agent {
                    ui.horizontal(|ui| {
                        ui.label("Tên Skill:");
                        ui.add_sized(
                            vec2(180.0, 22.0),
                            egui::TextEdit::singleline(&mut editor.skill_name),
                        );
                        ui.label("Mô tả:");
                        ui.add_sized(
                            vec2(260.0, 22.0),
                            egui::TextEdit::singleline(&mut editor.skill_desc),
                        );
                    });
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut editor.skill_body)
                            .desired_rows(3)
                            .desired_width(ui.available_width())
                            .hint_text("Nội dung hướng dẫn SKILL.md..."),
                    );
                    ui.add_space(4.0);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Lưu Skill vào Project",
                                    Lucide::SAVE
                                ))
                                .color(Color32::BLACK)
                                .strong(),
                            )
                            .fill(theme.accent_pop),
                        )
                        .clicked()
                    {
                        action = Some(StudioModalAction::SaveSkill {
                            name: editor.skill_name.clone(),
                            desc: editor.skill_desc.clone(),
                            body: editor.skill_body.clone(),
                        });
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Tên Agent:");
                        ui.add_sized(
                            vec2(180.0, 22.0),
                            egui::TextEdit::singleline(&mut editor.agent_name),
                        );
                        ui.label("Vai trò (Role):");
                        ui.add_sized(
                            vec2(260.0, 22.0),
                            egui::TextEdit::singleline(&mut editor.agent_role),
                        );
                    });
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut editor.agent_prompt)
                            .desired_rows(3)
                            .desired_width(ui.available_width())
                            .hint_text("System prompt & quy tắc của Subagent..."),
                    );
                    ui.add_space(4.0);
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(format!(
                                    "{} Lưu Subagent vào Project",
                                    Lucide::SAVE
                                ))
                                .color(Color32::BLACK)
                                .strong(),
                            )
                            .fill(theme.accent_pop),
                        )
                        .clicked()
                    {
                        action = Some(StudioModalAction::SaveAgent {
                            name: editor.agent_name.clone(),
                            role: editor.agent_role.clone(),
                            prompt: editor.agent_prompt.clone(),
                        });
                    }
                }
            });

        ui.add_space(8.0);

        // 3. Discovered Skills & Agents List
        ScrollArea::vertical()
            .id_salt("studio_discovered_list_scroll")
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Danh sách Skills trong Project ({})",
                        skills.len()
                    ))
                    .color(theme.accent_pop)
                    .strong()
                    .size(11.5),
                );
                for sk in skills {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{} ${}", Lucide::CODE, sk.name))
                                .color(theme.text_paper)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(&sk.description)
                                .color(theme.text_silver)
                                .size(11.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button(Lucide::TRASH).clicked() {
                                action = Some(StudioModalAction::DeleteSkill(sk.path.clone()));
                            }
                            if ui.small_button("Sửa").clicked() {
                                editor.is_editing_agent = false;
                                editor.skill_name = sk.name.clone();
                                editor.skill_desc = sk.description.clone();
                                editor.skill_body = sk.content.clone();
                            }
                            if ui.small_button("Dùng trong Chat").clicked() {
                                action = Some(StudioModalAction::InsertSkillIntoChat(format!(
                                    "${} ",
                                    sk.name
                                )));
                            }
                        });
                    });
                }

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("Danh sách Custom Subagents ({})", agents.len()))
                        .color(theme.accent_pop)
                        .strong()
                        .size(11.5),
                );
                for ag in agents {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{} @{}", Lucide::BOT, ag.name))
                                .color(theme.text_paper)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(&ag.role)
                                .color(theme.text_silver)
                                .size(11.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button(Lucide::TRASH).clicked() {
                                action = Some(StudioModalAction::DeleteAgent(ag.path.clone()));
                            }
                            if ui.small_button("Sửa").clicked() {
                                editor.is_editing_agent = true;
                                editor.agent_name = ag.name.clone();
                                editor.agent_role = ag.role.clone();
                                editor.agent_prompt = ag.content.clone();
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{} Chat 1-1",
                                            Lucide::MESSAGE_SQUARE
                                        ))
                                        .color(Color32::BLACK)
                                        .strong()
                                        .size(10.0),
                                    )
                                    .fill(theme.accent_pop),
                                )
                                .clicked()
                            {
                                action = Some(StudioModalAction::OpenDedicatedAgentChat(
                                    ag.name.clone(),
                                ));
                            }
                        });
                    });
                }
            });
    });
    action
}

/// Phase 12: Settings, Memory Reaper & 12 Canh Giờ Palette Switcher
pub fn render_settings_modal(
    ui: &mut Ui,
    theme: &CanhTheme,
    layout: &mut LayoutState,
    reaper: &mut MemoryReaper,
    _ticker: &SharedTicker,
) -> Option<StudioModalAction> {
    let mut action = None;
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} VICTINI SETTINGS, MEMORY REAPER & 12 CANH GIỜ",
                    Lucide::SETTINGS
                ))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("✕ Quay lại Chat").clicked() {
                    action = Some(StudioModalAction::CloseToChat);
                }
            });
        });

        ui.add_space(10.0);
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("1. Memory Reaper & Background Idle Scheduler")
                        .color(theme.text_paper)
                        .strong(),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        "Thời gian chờ trước khi đưa IDE về trạng thái Ngủ (giảm RAM < 45MB):",
                    );
                    let mut secs = reaper.idle_threshold_secs as u32;
                    if ui
                        .add(egui::Slider::new(&mut secs, 10..=180).suffix(" giây"))
                        .changed()
                    {
                        reaper.idle_threshold_secs = secs as u64;
                    }
                });
                ui.label(
                    egui::RichText::new(format!(
                        "• AGY Core Binary: {:?}\n• Browser Safe Profile: {:?} (Bảo vệ tuyệt đối Dad Profile)",
                        crate::config::paths::get_agy_path(),
                        crate::config::paths::get_browser_profile_dir()
                    ))
                    .color(theme.text_silver)
                    .size(11.0),
                );
            });

        ui.add_space(10.0);
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x11, 0x11))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("2. Bảng Màu 12 Canh Giờ & T3 Code OLED")
                        .color(theme.text_paper)
                        .strong(),
                );
                ui.checkbox(
                    &mut layout.auto_canh_gio,
                    "Tự động đổi màu theo giờ hệ thống thực tế",
                );
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    for t in CanhTheme::all_themes() {
                        let is_cur = t.id == theme.id;
                        let btn = egui::Button::new(
                            egui::RichText::new(t.name)
                                .color(if is_cur {
                                    Color32::BLACK
                                } else {
                                    t.text_paper
                                })
                                .size(11.0),
                        )
                        .fill(if is_cur { t.accent_pop } else { t.panel_bg });

                        if ui.add(btn).clicked() {
                            layout.auto_canh_gio = false;
                            action = Some(StudioModalAction::SelectTheme(t));
                        }
                    }
                });
            });

        ui.add_space(10.0);
        if ui.button("Quay về màn hình Chat chính").clicked() {
            layout.center_mode = CenterViewMode::ChatTimeline;
            action = Some(StudioModalAction::CloseToChat);
        }
    });
    action
}
