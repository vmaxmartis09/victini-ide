use crate::ai::{AgentAvatarState, AvatarManager, MemoryReaper, QuotedContext, ALL_AGENTS};
use crate::config::{
    ConflictStrategy, DiscoveredAgent, DiscoveredSkill, ImportEvent, ProjectManager,
    ScanRepoResult, SkillScope, SkillSourceMeta, TrashedItem,
};
use crate::git::{DiffHunk, GitFileChange};
use crate::theme::{CanhTheme, Lucide, SharedTicker};
use crate::ui::header::icon_btn;
use crate::ui::layout_state::LayoutState;
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;

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
    CreateNewFile(String),
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
        scope: SkillScope,
    },
    SaveAgent {
        name: String,
        role: String,
        prompt: String,
        scope: SkillScope,
    },
    ToggleSkillEnabled {
        path: PathBuf,
        enabled: bool,
    },
    DeleteSkill(PathBuf),
    DeleteAgent(PathBuf),
    RestoreTrashed(usize),
    ScanGitHubRepo {
        url: String,
        branch: String,
    },
    InstallScannedSkills,
    UpdateSkillFromSource {
        path: PathBuf,
        source: SkillSourceMeta,
    },
    InsertSkillIntoChat(String),
    OpenDedicatedAgentChat(String),
    RunSecurityDiagnostics,
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
    pub selected_scope: SkillScope,
    pub studio_tab: usize, // 0 = Create Skill, 1 = Create Agent, 2 = Import GitHub Repo

    // Live File & Diff Editor State
    pub active_file_path: String,
    pub file_editor_buffer: String,
    pub file_editor_dirty: bool,
    pub diff_edit_mode: bool, // false = Diff View, true = Live Code Editor
    pub file_filter_query: String,
    pub new_file_path_input: String,
    pub show_new_file_input: bool,
    pub status_notice: Option<String>,

    // GitHub Skill Importer & Trash Undo State
    pub github_repo_url: String,
    pub github_branch: String,
    pub import_conflict_strategy: ConflictStrategy,
    pub import_progress: Option<(String, f32)>,
    pub scan_result: Option<ScanRepoResult>,
    pub import_receiver: Option<std_mpsc::Receiver<ImportEvent>>,
    pub trashed_items: Vec<TrashedItem>,
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
            selected_scope: SkillScope::Project,
            studio_tab: 0,
            active_file_path: String::new(),
            file_editor_buffer: String::new(),
            file_editor_dirty: false,
            diff_edit_mode: false,
            file_filter_query: String::new(),
            new_file_path_input: String::new(),
            show_new_file_input: false,
            status_notice: None,
            github_repo_url: "nextlevelbuilder/ui-ux-pro-max-skill".to_string(),
            github_branch: String::new(),
            import_conflict_strategy: ConflictStrategy::Overwrite,
            import_progress: None,
            scan_result: None,
            import_receiver: None,
            trashed_items: Vec::new(),
        }
    }
}

/// Phase A.4 & 3.3: Dual-Mode Interactive Git Diff Viewer & Live Code Editor (Icon-First + Light/Dark Ready)
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
        // Top Header Bar (100% Icon-First Action Controls)
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
                        .color(theme.success)
                        .size(11.0),
                );
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_btn(
                    ui,
                    Lucide::X,
                    theme.text_paper,
                    theme.chip_bg,
                    "Quay lại màn hình Chat chính",
                )
                .clicked()
                {
                    action = Some(StudioModalAction::CloseToChat);
                }

                if !editor.active_file_path.is_empty()
                    && icon_btn(
                        ui,
                        Lucide::EXTERNAL_LINK,
                        theme.text_silver,
                        theme.chip_bg,
                        "Mở file này bằng Cursor / VS Code / Zed",
                    )
                    .clicked()
                {
                    action = Some(StudioModalAction::OpenExternalEditor(Some(
                        editor.active_file_path.clone(),
                    )));
                }

                if let Some(ref hunk) = diff_hunk {
                    if icon_btn(
                        ui,
                        Lucide::QUOTE,
                        theme.accent_pop,
                        theme.accent_surface,
                        "Trích dẫn (Quote) Git Diff này vào khung Chat AI",
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

        let total_h = (ui.available_height() - 10.0).max(200.0);
        ui.horizontal(|ui| {
            // LEFT PANE: Changed Files + Project File Picker + New File Creator
            Frame::none()
                .fill(theme.panel_bg)
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::same(8.0))
                .show(ui, |ui| {
                    ui.set_width(225.0);
                    ui.set_min_height(total_h);

                    ui.vertical(|ui| {
                        // Filter search input + New File Icon Button
                        ui.horizontal(|ui| {
                            let filter_w = (ui.available_width() - 32.0).max(80.0);
                            Frame::none()
                                .fill(theme.input_bg)
                                .rounding(Rounding::same(5.0))
                                .stroke(Stroke::new(1.0, theme.border))
                                .inner_margin(Margin::symmetric(6.0, 3.0))
                                .show(ui, |ui| {
                                    ui.add_sized(
                                        vec2(filter_w, 18.0),
                                        egui::TextEdit::singleline(&mut editor.file_filter_query)
                                            .hint_text("Filter files...")
                                            .frame(false)
                                            .text_color(theme.text_paper),
                                    );
                                });

                            if icon_btn(
                                ui,
                                Lucide::FILE_PLUS,
                                theme.accent_pop,
                                theme.chip_bg,
                                "Tạo file mới trong Project",
                            )
                            .clicked()
                            {
                                editor.show_new_file_input = !editor.show_new_file_input;
                            }
                        });

                        if editor.show_new_file_input {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                let w = (ui.available_width() - 32.0).max(80.0);
                                Frame::none()
                                    .fill(theme.input_bg)
                                    .rounding(Rounding::same(4.0))
                                    .stroke(Stroke::new(1.0, theme.accent_pop))
                                    .inner_margin(Margin::symmetric(6.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.add_sized(
                                            vec2(w, 18.0),
                                            egui::TextEdit::singleline(
                                                &mut editor.new_file_path_input,
                                            )
                                            .hint_text("src/new_file.rs")
                                            .frame(false)
                                            .text_color(theme.text_paper),
                                        );
                                    });
                                if icon_btn(
                                    ui,
                                    Lucide::CHECK,
                                    theme.on_accent,
                                    theme.accent_pop,
                                    "Tạo & mở file mới",
                                )
                                .clicked()
                                    && !editor.new_file_path_input.trim().is_empty()
                                {
                                    let p = editor.new_file_path_input.trim().to_string();
                                    editor.new_file_path_input.clear();
                                    editor.show_new_file_input = false;
                                    action = Some(StudioModalAction::CreateNewFile(p));
                                }
                            });
                        }

                        ui.add_space(6.0);

                        ScrollArea::vertical()
                            .id_salt("editor_left_files_scroll")
                            .max_height(total_h - 45.0)
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
                                        egui::RichText::new("Working tree clean")
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
                                            theme.surface_2
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
                                                        "A" | "?" => theme.success,
                                                        "D" => theme.danger,
                                                        _ => theme.warning,
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
                .fill(theme.panel_bg)
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, theme.border))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ui.set_width(right_w - 20.0);
                    ui.set_min_height(total_h);

                    ui.vertical(|ui| {
                        // Editor Toolbar: File Name + Icon-First Action Buttons
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
                                    egui::RichText::new("●")
                                        .color(theme.accent_pop)
                                        .strong()
                                        .size(12.0),
                                )
                                .on_hover_text("Có thay đổi chưa lưu (Bấm ⌘S hoặc nút Save)");
                            }

                            if let Some(ref hunk) = diff_hunk {
                                ui.label(
                                    egui::RichText::new(format!("+{}", hunk.added_count))
                                        .color(theme.success)
                                        .monospace()
                                        .strong()
                                        .size(11.0),
                                );
                                ui.label(
                                    egui::RichText::new(format!("-{}", hunk.removed_count))
                                        .color(theme.danger)
                                        .monospace()
                                        .strong()
                                        .size(11.0),
                                );
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Save Icon Button
                                if !editor.active_file_path.is_empty() {
                                    if icon_btn(
                                        ui,
                                        Lucide::SAVE,
                                        theme.on_accent,
                                        if editor.file_editor_dirty {
                                            theme.accent_pop
                                        } else {
                                            theme.success
                                        },
                                        "Lưu file xuống ổ cứng (⌘S / Ctrl+S)",
                                    )
                                    .clicked()
                                    {
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

                                    // Stage / Unstage / Discard Icon Buttons
                                    if let Some(fc) = file_changes
                                        .iter()
                                        .find(|c| c.path == editor.active_file_path)
                                    {
                                        if fc.is_staged {
                                            if icon_btn(
                                                ui,
                                                Lucide::MINUS,
                                                theme.warning,
                                                theme.chip_bg,
                                                "Unstage file này",
                                            )
                                            .clicked()
                                            {
                                                action =
                                                    Some(StudioModalAction::UnstageFileFromEditor(
                                                        fc.path.clone(),
                                                    ));
                                            }
                                        } else {
                                            if icon_btn(
                                                ui,
                                                Lucide::ROTATE_CCW,
                                                theme.danger,
                                                theme.chip_bg,
                                                "Discard thay đổi của file này về HEAD",
                                            )
                                            .clicked()
                                            {
                                                action =
                                                    Some(StudioModalAction::DiscardFileFromEditor(
                                                        fc.path.clone(),
                                                    ));
                                            }
                                            if icon_btn(
                                                ui,
                                                Lucide::PLUS,
                                                theme.success,
                                                theme.chip_bg,
                                                "Stage file này",
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

                                ui.add_space(6.0);

                                // Icon Mode Switcher: [Edit (Live Code Editor)] vs [Layers (Diff View)]
                                if icon_btn(
                                    ui,
                                    Lucide::EDIT,
                                    if editor.diff_edit_mode {
                                        theme.on_accent
                                    } else {
                                        theme.text_paper
                                    },
                                    if editor.diff_edit_mode {
                                        theme.accent_pop
                                    } else {
                                        theme.chip_bg
                                    },
                                    "Chế độ: Live Code Editor (Sửa code trực tiếp)",
                                )
                                .clicked()
                                {
                                    editor.diff_edit_mode = true;
                                }

                                if icon_btn(
                                    ui,
                                    Lucide::LAYERS,
                                    if !editor.diff_edit_mode {
                                        theme.on_accent
                                    } else {
                                        theme.text_paper
                                    },
                                    if !editor.diff_edit_mode {
                                        theme.accent_pop
                                    } else {
                                        theme.chip_bg
                                    },
                                    "Chế độ: Unified Diff View (Xem chi tiết dòng thêm/xóa)",
                                )
                                .clicked()
                                {
                                    editor.diff_edit_mode = false;
                                }
                            });
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Main Content Body (Code Background adapts to Light & Dark Mode)
                        let body_h = (total_h - 55.0).max(150.0);
                        Frame::none()
                            .fill(theme.code_bg)
                            .rounding(Rounding::same(6.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                if editor.diff_edit_mode {
                                    // LIVE CODE EDITOR MODE
                                    if editor.active_file_path.is_empty() {
                                        ui.label(
                                            egui::RichText::new(
                                                "Chọn một file từ cột bên trái hoặc bấm nút tạo file mới (+) để bắt đầu chỉnh sửa.",
                                            )
                                            .color(theme.text_silver),
                                        );
                                    } else {
                                        ScrollArea::vertical()
                                            .id_salt("live_code_editor_scroll")
                                            .auto_shrink([false, false])
                                            .max_height(body_h)
                                            .show(ui, |ui| {
                                                let line_count = editor
                                                    .file_editor_buffer
                                                    .lines()
                                                    .count()
                                                    .max(1);
                                                ui.horizontal_top(|ui| {
                                                    let gutter: String = (1..=line_count)
                                                        .map(|n| format!("{:>4}", n))
                                                        .collect::<Vec<_>>()
                                                        .join("\n");
                                                    ui.label(
                                                        egui::RichText::new(gutter)
                                                            .color(theme.text_silver)
                                                            .monospace()
                                                            .size(11.5),
                                                    );
                                                    ui.add_space(8.0);

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
                                                        '+' => {
                                                            (theme.diff_add_bg, theme.diff_add_fg)
                                                        }
                                                        '-' => {
                                                            (theme.diff_del_bg, theme.diff_del_fg)
                                                        }
                                                        _ => {
                                                            (Color32::TRANSPARENT, theme.text_paper)
                                                        }
                                                    };
                                                    Frame::none().fill(bg).show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            let old_ln = line
                                                                .old_line
                                                                .map(|n| format!("{:>4}", n))
                                                                .unwrap_or_else(|| {
                                                                    "    ".to_string()
                                                                });
                                                            let new_ln = line
                                                                .new_line
                                                                .map(|n| format!("{:>4}", n))
                                                                .unwrap_or_else(|| {
                                                                    "    ".to_string()
                                                                });
                                                            ui.label(
                                                                egui::RichText::new(format!(
                                                                    "{} {} {}",
                                                                    old_ln, new_ln, line.tag
                                                                ))
                                                                .color(theme.text_silver)
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
                                                "Chọn một file bên trái để xem Diff hoặc bấm biểu tượng cây bút để sửa code trực tiếp.",
                                            )
                                            .color(theme.text_silver),
                                        );
                                    }
                                }
                            });
                    });
                });
        });
    });
    action
}

/// Phase 3: Multi-Project Workspace Manager (Icon-First + Light/Dark Ready)
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
                if icon_btn(
                    ui,
                    Lucide::X,
                    theme.text_paper,
                    theme.chip_bg,
                    "Quay lại Chat",
                )
                .clicked()
                {
                    action = Some(StudioModalAction::CloseToChat);
                }
            });
        });

        ui.add_space(8.0);
        Frame::none()
            .fill(theme.panel_bg)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(Lucide::FOLDER_PLUS)
                            .color(theme.accent_pop)
                            .size(13.0),
                    );
                    let w = (ui.available_width() - 42.0).max(140.0);
                    Frame::none()
                        .fill(theme.input_bg)
                        .rounding(Rounding::same(5.0))
                        .stroke(Stroke::new(1.0, theme.border))
                        .inner_margin(Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.add_sized(
                                vec2(w, 20.0),
                                egui::TextEdit::singleline(&mut editor.new_project_path)
                                    .frame(false)
                                    .text_color(theme.text_paper)
                                    .hint_text("/Volumes/Data/ten-du-an"),
                            );
                        });
                    if icon_btn(
                        ui,
                        Lucide::ARROW_RIGHT,
                        theme.on_accent,
                        theme.accent_pop,
                        "Mở hoặc thêm Project này vào Workspace",
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
                            theme.surface_2
                        } else {
                            theme.panel_bg
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
                                    if !is_active {
                                        if icon_btn(
                                            ui,
                                            Lucide::FOLDER_OPEN,
                                            theme.on_accent,
                                            theme.accent_pop,
                                            "Chuyển sang Project này",
                                        )
                                        .clicked()
                                        {
                                            action = Some(StudioModalAction::SwitchProject(
                                                proj.path.clone(),
                                            ));
                                        }
                                    } else {
                                        ui.label(
                                            egui::RichText::new(Lucide::CHECK)
                                                .color(theme.success)
                                                .strong()
                                                .size(12.0),
                                        )
                                        .on_hover_text("Project đang mở");
                                    }

                                    if icon_btn(
                                        ui,
                                        Lucide::PIN,
                                        if proj.pinned {
                                            theme.accent_pop
                                        } else {
                                            theme.text_silver
                                        },
                                        theme.chip_bg,
                                        if proj.pinned {
                                            "Bỏ ghim Project"
                                        } else {
                                            "Ghim Project lên đầu"
                                        },
                                    )
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

/// Phase 7 & 3.1: Agent & Skill Studio + GitHub Repo Skill Importer + Trash Undo (Icon-First)
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
                    "{} AGENT & SKILL STUDIO + GITHUB IMPORTER",
                    Lucide::BOT
                ))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
            );
            if let Some(ref notice) = editor.status_notice {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(notice)
                        .color(theme.success)
                        .size(11.0),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_btn(
                    ui,
                    Lucide::X,
                    theme.text_paper,
                    theme.chip_bg,
                    "Quay lại Chat",
                )
                .clicked()
                {
                    action = Some(StudioModalAction::CloseToChat);
                }
            });
        });

        ui.add_space(6.0);

        // 1. Core Swarm Agents Gallery with 1-Click Chat 1-1 Icon
        Frame::none()
            .fill(theme.panel_bg)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for &ag in ALL_AGENTS {
                        Frame::none()
                            .fill(theme.surface_1)
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0, theme.border))
                            .inner_margin(Margin::symmetric(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    AvatarManager::render_animated_avatar(
                                        ui,
                                        ag,
                                        AgentAvatarState::Answering,
                                        26.0,
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
                                            .size(10.8),
                                        );
                                        ui.label(
                                            egui::RichText::new(ag.title())
                                                .color(theme.text_silver)
                                                .size(9.5),
                                        );
                                    });
                                    if icon_btn(
                                        ui,
                                        Lucide::MESSAGE_SQUARE,
                                        Color32::WHITE,
                                        theme.success,
                                        &format!("Mở Chat 1-1 riêng với {}", ag.name()),
                                    )
                                    .clicked()
                                    {
                                        action = Some(StudioModalAction::OpenDedicatedAgentChat(
                                            ag.name().to_string(),
                                        ));
                                    }
                                });
                            });
                    }
                });
            });

        ui.add_space(8.0);

        // 2. Studio Action Tabs: [Wand Skill] | [Bot Subagent] | [CloudDownload GitHub Import]
        Frame::none()
            .fill(theme.panel_bg)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (t_idx, ic, lbl) in [
                        (0, Lucide::WAND, "Skill (SKILL.md)"),
                        (1, Lucide::BOT, "Subagent (.md)"),
                        (2, Lucide::CLOUD_DOWNLOAD, "GitHub Skill Importer"),
                    ] {
                        let is_sel = editor.studio_tab == t_idx;
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!("{} {}", ic, lbl))
                                        .color(if is_sel {
                                            theme.on_accent
                                        } else {
                                            theme.text_paper
                                        })
                                        .strong()
                                        .size(10.8),
                                )
                                .fill(if is_sel {
                                    theme.accent_pop
                                } else {
                                    theme.chip_bg
                                })
                                .rounding(Rounding::same(5.0)),
                            )
                            .clicked()
                        {
                            editor.studio_tab = t_idx;
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Scope Toggle: Project vs Global
                        for sc in [SkillScope::Project, SkillScope::Global] {
                            let is_sc = editor.selected_scope == sc;
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(sc.label())
                                            .color(if is_sc {
                                                theme.on_accent
                                            } else {
                                                theme.text_silver
                                            })
                                            .size(10.0),
                                    )
                                    .fill(if is_sc {
                                        theme.info
                                    } else {
                                        theme.chip_bg
                                    })
                                    .rounding(Rounding::same(4.0)),
                                )
                                .on_hover_text("Chọn phạm vi lưu: Project hoặc Global")
                                .clicked()
                            {
                                editor.selected_scope = sc;
                            }
                        }
                    });
                });

                ui.add_space(6.0);

                match editor.studio_tab {
                    0 => {
                        // CREATE / EDIT SKILL
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.add_sized(
                                vec2(170.0, 22.0),
                                egui::TextEdit::singleline(&mut editor.skill_name),
                            );
                            ui.label("Desc:");
                            ui.add_sized(
                                vec2(260.0, 22.0),
                                egui::TextEdit::singleline(&mut editor.skill_desc),
                            );
                            if icon_btn(
                                ui,
                                Lucide::SAVE,
                                theme.on_accent,
                                theme.accent_pop,
                                "Lưu Skill (SKILL.md)",
                            )
                            .clicked()
                            {
                                action = Some(StudioModalAction::SaveSkill {
                                    name: editor.skill_name.clone(),
                                    desc: editor.skill_desc.clone(),
                                    body: editor.skill_body.clone(),
                                    scope: editor.selected_scope,
                                });
                            }
                        });
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::multiline(&mut editor.skill_body)
                                .desired_rows(3)
                                .desired_width(ui.available_width())
                                .hint_text("Nội dung hướng dẫn SKILL.md..."),
                        );
                    }
                    1 => {
                        // CREATE / EDIT SUBAGENT
                        ui.horizontal(|ui| {
                            ui.label("Agent:");
                            ui.add_sized(
                                vec2(170.0, 22.0),
                                egui::TextEdit::singleline(&mut editor.agent_name),
                            );
                            ui.label("Role:");
                            ui.add_sized(
                                vec2(260.0, 22.0),
                                egui::TextEdit::singleline(&mut editor.agent_role),
                            );
                            if icon_btn(
                                ui,
                                Lucide::SAVE,
                                theme.on_accent,
                                theme.accent_pop,
                                "Lưu Subagent (.claude/agents/*.md)",
                            )
                            .clicked()
                            {
                                action = Some(StudioModalAction::SaveAgent {
                                    name: editor.agent_name.clone(),
                                    role: editor.agent_role.clone(),
                                    prompt: editor.agent_prompt.clone(),
                                    scope: editor.selected_scope,
                                });
                            }
                        });
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::multiline(&mut editor.agent_prompt)
                                .desired_rows(3)
                                .desired_width(ui.available_width())
                                .hint_text("System prompt & quy tắc của Subagent..."),
                        );
                    }
                    _ => {
                        // GITHUB REPOSITORY SKILL IMPORTER
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(Lucide::GLOBE)
                                    .color(theme.info)
                                    .size(12.0),
                            );
                            ui.add_sized(
                                vec2(260.0, 22.0),
                                egui::TextEdit::singleline(&mut editor.github_repo_url)
                                    .hint_text("owner/repo hoặc https://github.com/..."),
                            );
                            ui.add_sized(
                                vec2(90.0, 22.0),
                                egui::TextEdit::singleline(&mut editor.github_branch)
                                    .hint_text("branch (opt)"),
                            );
                            if icon_btn(
                                ui,
                                Lucide::SEARCH,
                                theme.on_accent,
                                theme.accent_pop,
                                "Quét các SKILL.md trong GitHub Repository này",
                            )
                            .clicked()
                            {
                                action = Some(StudioModalAction::ScanGitHubRepo {
                                    url: editor.github_repo_url.clone(),
                                    branch: editor.github_branch.clone(),
                                });
                            }

                            // Conflict strategy cycle button
                            let next_strat = match editor.import_conflict_strategy {
                                ConflictStrategy::Overwrite => ConflictStrategy::Rename,
                                ConflictStrategy::Rename => ConflictStrategy::Skip,
                                ConflictStrategy::Skip => ConflictStrategy::Overwrite,
                            };
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(
                                            editor.import_conflict_strategy.label(),
                                        )
                                        .color(theme.text_silver)
                                        .size(10.0),
                                    )
                                    .fill(theme.chip_bg),
                                )
                                .on_hover_text("Click để đổi chiến lược xử lý khi trùng tên Skill")
                                .clicked()
                            {
                                editor.import_conflict_strategy = next_strat;
                            }
                        });

                        if let Some((ref prog_msg, pct)) = editor.import_progress {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(prog_msg)
                                        .color(theme.accent_pop)
                                        .size(10.5),
                                );
                                ui.label(
                                    egui::RichText::new(format!("{:.0}%", pct * 100.0))
                                        .color(theme.text_silver)
                                        .size(10.0),
                                );
                            });
                        }

                        if let Some(ref mut scan) = editor.scan_result {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Tìm thấy {} skills trong {} (@{}):",
                                        scan.candidates.len(),
                                        scan.source.repo_slug(),
                                        &scan.commit_sha[..7.min(scan.commit_sha.len())]
                                    ))
                                    .color(theme.success)
                                    .strong()
                                    .size(10.8),
                                );
                                if icon_btn(
                                    ui,
                                    Lucide::CLOUD_DOWNLOAD,
                                    theme.on_accent,
                                    theme.success,
                                    "Cài đặt các Skills đã chọn",
                                )
                                .clicked()
                                {
                                    action = Some(StudioModalAction::InstallScannedSkills);
                                }
                            });
                            ScrollArea::vertical()
                                .id_salt("github_scan_candidates_scroll")
                                .max_height(90.0)
                                .show(ui, |ui| {
                                    for cand in &mut scan.candidates {
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut cand.selected, "");
                                            ui.label(
                                                egui::RichText::new(format!("${}", cand.name))
                                                    .color(theme.text_paper)
                                                    .strong()
                                                    .size(10.5),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "({} files) — {}",
                                                    cand.file_count, cand.description
                                                ))
                                                .color(theme.text_silver)
                                                .size(10.0),
                                            );
                                        });
                                    }
                                });
                        }
                    }
                }
            });

        ui.add_space(8.0);

        // 3. Discovered Skills, Custom Subagents & Trash Undo List (Icon-First)
        ScrollArea::vertical()
            .id_salt("studio_discovered_list_scroll")
            .show(ui, |ui| {
                // Trash Restore Banner if any item was recently deleted
                if !editor.trashed_items.is_empty() {
                    Frame::none()
                        .fill(theme.surface_1)
                        .rounding(Rounding::same(6.0))
                        .stroke(Stroke::new(1.0, theme.warning))
                        .inner_margin(Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} Trash ({}):",
                                        Lucide::ARCHIVE,
                                        editor.trashed_items.len()
                                    ))
                                    .color(theme.warning)
                                    .strong()
                                    .size(10.5),
                                );
                                for (t_idx, item) in editor.trashed_items.iter().enumerate() {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new(format!(
                                                    "{} Khôi phục {} '{}'",
                                                    Lucide::ROTATE_CCW,
                                                    item.kind_label,
                                                    item.name
                                                ))
                                                .color(theme.text_paper)
                                                .size(10.0),
                                            )
                                            .fill(theme.chip_bg),
                                        )
                                        .clicked()
                                    {
                                        action = Some(StudioModalAction::RestoreTrashed(t_idx));
                                    }
                                }
                            });
                        });
                    ui.add_space(6.0);
                }

                ui.label(
                    egui::RichText::new(format!(
                        "SKILLS ({})",
                        skills.len()
                    ))
                    .color(theme.accent_pop)
                    .strong()
                    .size(11.0),
                );
                for sk in skills {
                    ui.horizontal(|ui| {
                        let mut en = sk.enabled;
                        if ui
                            .checkbox(&mut en, "")
                            .on_hover_text("Bật / Tắt Skill này")
                            .changed()
                        {
                            action = Some(StudioModalAction::ToggleSkillEnabled {
                                path: sk.path.clone(),
                                enabled: en,
                            });
                        }
                        ui.label(
                            egui::RichText::new(format!("{} ${}", Lucide::WAND, sk.name))
                                .color(if sk.enabled {
                                    theme.text_paper
                                } else {
                                    theme.text_silver
                                })
                                .strong()
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("[{}]", sk.scope.label()))
                                .color(theme.info)
                                .size(9.5),
                        );
                        ui.label(
                            egui::RichText::new(&sk.description)
                                .color(theme.text_silver)
                                .size(10.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::TRASH,
                                theme.danger,
                                Color32::TRANSPARENT,
                                "Chuyển Skill này vào Thùng rác (Có thể Undo)",
                            )
                            .clicked()
                            {
                                action = Some(StudioModalAction::DeleteSkill(sk.path.clone()));
                            }
                            if let Some(ref src) = sk.source {
                                if icon_btn(
                                    ui,
                                    Lucide::REFRESH,
                                    theme.info,
                                    theme.chip_bg,
                                    &format!("Cập nhật Skill từ GitHub ({})", src.repo),
                                )
                                .clicked()
                                {
                                    action = Some(StudioModalAction::UpdateSkillFromSource {
                                        path: sk.path.clone(),
                                        source: src.clone(),
                                    });
                                }
                            }
                            if icon_btn(
                                ui,
                                Lucide::EDIT,
                                theme.text_silver,
                                theme.chip_bg,
                                "Chỉnh sửa nội dung Skill",
                            )
                            .clicked()
                            {
                                editor.studio_tab = 0;
                                editor.selected_scope = sk.scope;
                                editor.skill_name = sk.name.clone();
                                editor.skill_desc = sk.description.clone();
                                editor.skill_body = sk.body.clone();
                            }
                            if icon_btn(
                                ui,
                                Lucide::MESSAGE_SQUARE,
                                theme.on_accent,
                                theme.accent_pop,
                                "Đính kèm Skill này vào phiên Chat",
                            )
                            .clicked()
                            {
                                action = Some(StudioModalAction::InsertSkillIntoChat(
                                    sk.name.clone(),
                                ));
                            }
                        });
                    });
                }

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("CUSTOM SUBAGENTS ({})", agents.len()))
                        .color(theme.info)
                        .strong()
                        .size(11.0),
                );
                for ag in agents {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{} @{}", Lucide::BOT, ag.name))
                                .color(theme.text_paper)
                                .strong()
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("[{}]", ag.scope.label()))
                                .color(theme.info)
                                .size(9.5),
                        );
                        ui.label(
                            egui::RichText::new(&ag.role)
                                .color(theme.text_silver)
                                .size(10.5),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_btn(
                                ui,
                                Lucide::TRASH,
                                theme.danger,
                                Color32::TRANSPARENT,
                                "Xóa Subagent này (Có thể Undo)",
                            )
                            .clicked()
                            {
                                action = Some(StudioModalAction::DeleteAgent(ag.path.clone()));
                            }
                            if icon_btn(
                                ui,
                                Lucide::EDIT,
                                theme.text_silver,
                                theme.chip_bg,
                                "Chỉnh sửa System Prompt của Subagent",
                            )
                            .clicked()
                            {
                                editor.studio_tab = 1;
                                editor.selected_scope = ag.scope;
                                editor.agent_name = ag.name.clone();
                                editor.agent_role = ag.role.clone();
                                editor.agent_prompt = ag.body.clone();
                            }
                            if icon_btn(
                                ui,
                                Lucide::MESSAGE_SQUARE,
                                Color32::WHITE,
                                theme.info,
                                &format!("Mở Chat 1-1 riêng với @{}", ag.name),
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

/// Phase 12 & Phase 2: Settings, Light/Dark Mode Switcher & Elegans Diagnostics
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
                    "{} VICTINI SETTINGS, LIGHT/DARK MODE & DIAGNOSTICS",
                    Lucide::SETTINGS
                ))
                .color(theme.accent_pop)
                .strong()
                .size(13.0),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_btn(
                    ui,
                    Lucide::X,
                    theme.text_paper,
                    theme.chip_bg,
                    "Quay lại Chat",
                )
                .clicked()
                {
                    action = Some(StudioModalAction::CloseToChat);
                }
            });
        });

        ui.add_space(10.0);
        Frame::none()
            .fill(theme.panel_bg)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("1. Giao diện T3 Code OLED Dark & Paper Light Mode (⌘L)")
                        .color(theme.text_paper)
                        .strong(),
                );
                ui.checkbox(
                    &mut layout.auto_canh_gio,
                    "Tự động chuyển Light Mode (07:00–18:00) / Dark Mode (18:00–07:00) theo giờ hệ thống",
                );
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    for t in CanhTheme::all_themes() {
                        let is_cur = t.id == theme.id;
                        let ic = if t.is_dark { Lucide::MOON } else { Lucide::SUN };
                        let btn = egui::Button::new(
                            egui::RichText::new(format!("{} {}", ic, t.name))
                                .color(if is_cur {
                                    theme.on_accent
                                } else {
                                    theme.text_paper
                                })
                                .strong()
                                .size(11.0),
                        )
                        .fill(if is_cur {
                            theme.accent_pop
                        } else {
                            theme.chip_bg
                        })
                        .rounding(Rounding::same(6.0));

                        if ui.add(btn).clicked() {
                            layout.auto_canh_gio = false;
                            action = Some(StudioModalAction::SelectTheme(t));
                        }
                    }
                });
            });

        ui.add_space(10.0);
        Frame::none()
            .fill(theme.panel_bg)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("2. Memory Reaper & Background Idle Scheduler")
                        .color(theme.text_paper)
                        .strong(),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Thời gian chờ trước khi đưa IDE về trạng thái Ngủ:");
                    let mut secs = reaper.idle_threshold_secs as u32;
                    if ui
                        .add(egui::Slider::new(&mut secs, 10..=180).suffix("s"))
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
            .fill(theme.panel_bg)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, theme.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(
                            "3. Chẩn đoán bảo mật Elegans (Env Secret Guard & AST Blast Radius)",
                        )
                        .color(theme.text_paper)
                        .strong(),
                    );
                    if icon_btn(
                        ui,
                        Lucide::SHIELD,
                        theme.on_accent,
                        theme.accent_pop,
                        "Chạy quét bảo mật & phân tích phụ thuộc ngay",
                    )
                    .clicked()
                    {
                        action = Some(StudioModalAction::RunSecurityDiagnostics);
                    }
                });
            });
    });
    action
}
