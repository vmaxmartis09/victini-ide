use crate::theme::Lucide;
use crate::ai::{AgentAvatarState, AvatarManager, MemoryReaper, QuotedContext, ALL_AGENTS};
use crate::config::{DiscoveredAgent, DiscoveredSkill, ProjectManager};
use crate::git::DiffHunk;
use crate::theme::{CanhTheme, SharedTicker};
use crate::ui::layout_state::{CenterViewMode, LayoutState};
use egui::{vec2, Align, Color32, Frame, Layout, Margin, Rounding, ScrollArea, Stroke, Ui};
use std::path::PathBuf;

pub enum StudioModalAction {
    CloseToChat,
    SwitchProject(PathBuf),
    AddNewProject(String),
    TogglePinProject(PathBuf),
    QuoteDiffToChat(QuotedContext),
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
}

impl Default for StudioEditorState {
    fn default() -> Self {
        Self {
            new_project_path: "/Volumes/Data/".to_string(),
            skill_name: "vibe-ui-audit".to_string(),
            skill_desc: "Tự động kiểm tra giao diện & chống AI slop".to_string(),
            skill_body: "Kiểm tra độ tương phản, font Be Vietnam Pro và responsive 3 breakpoints.".to_string(),
            agent_name: "fe_custom_specialist".to_string(),
            agent_role: "Chuyên gia tối ưu Rust WGPU & GitLab CI".to_string(),
            agent_prompt: "Thực thi code sạch, gọn theo nguyên tắc Ponytail Zero-Bloat.".to_string(),
            is_editing_agent: false,
        }
    }
}

/// Phase 4: Inline Git Diff Viewer Modal
pub fn render_git_diff_viewer(
    ui: &mut Ui,
    theme: &CanhTheme,
    diff_hunk: &Option<DiffHunk>,
) -> Option<StudioModalAction> {
    let mut action = None;
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("{} INLINE GIT DIFF VIEWER", Lucide::SEARCH))
                    .color(theme.accent_pop)
                    .strong()
                    .size(13.0),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("✕ Quay lại Chat").clicked() {
                    action = Some(StudioModalAction::CloseToChat);
                }
                if let Some(ref hunk) = diff_hunk {
                    if ui
                        .button(
                            egui::RichText::new("❝ Quote Diff vào Chat AI")
                                .color(theme.accent_pop)
                                .strong(),
                        )
                        .clicked()
                    {
                        let snippet: String = hunk
                            .lines
                            .iter()
                            .filter(|l| l.tag != ' ')
                            .take(25)
                            .map(|l| format!("{} {}", l.tag, l.content))
                            .collect::<Vec<_>>()
                            .join("\n");
                        action = Some(StudioModalAction::QuoteDiffToChat(QuotedContext {
                            source_msg_id: hunk.file_path.clone(),
                            author_label: "Git Diff".to_string(),
                            snippet: if snippet.is_empty() {
                                format!("File {}", hunk.file_path)
                            } else {
                                snippet
                            },
                            is_quote_block: true,
                        }));
                    }
                }
            });
        });

        ui.add_space(6.0);
        if let Some(ref hunk) = diff_hunk {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&hunk.file_path)
                        .color(theme.text_paper)
                        .strong()
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(format!("+{}", hunk.added_count))
                        .color(Color32::from_rgb(0x98, 0xC3, 0x79))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("-{}", hunk.removed_count))
                        .color(Color32::from_rgb(0xE0, 0x6C, 0x75))
                        .strong(),
                );
            });
            ui.add_space(6.0);

            Frame::none()
                .fill(Color32::from_rgb(0x09, 0x0B, 0x10))
                .rounding(Rounding::same(8.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(0x1E, 0x24, 0x32)))
                .inner_margin(Margin::same(10.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .id_salt("inline_diff_lines_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for line in &hunk.lines {
                                let (bg, fg) = match line.tag {
                                    '+' => (
                                        Color32::from_rgba_unmultiplied(0x28, 0x48, 0x2C, 95),
                                        Color32::from_rgb(0xA8, 0xE3, 0x89),
                                    ),
                                    '-' => (
                                        Color32::from_rgba_unmultiplied(0x52, 0x22, 0x26, 95),
                                        Color32::from_rgb(0xF0, 0x8C, 0x95),
                                    ),
                                    _ => (Color32::TRANSPARENT, theme.text_silver),
                                };
                                Frame::none().fill(bg).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let ln = line
                                            .new_line
                                            .or(line.old_line)
                                            .map(|n| format!("{:>4}", n))
                                            .unwrap_or_else(|| "    ".to_string());
                                        ui.label(
                                            egui::RichText::new(format!("{} {}", ln, line.tag))
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
                });
        } else {
            ui.label("Chọn một file trong danh sách GIT CHANGES ở Sidebar để xem Diff chi tiết.");
        }
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
                egui::RichText::new(format!("{} MULTI-PROJECT & WORKSPACE MANAGER", Lucide::FOLDER))
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
            .fill(Color32::from_rgb(0x11, 0x14, 0x1D))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("＋ Mở hoặc thêm đường dẫn Project:")
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
                                egui::RichText::new("Mở Project ⚡")
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
                            Color32::from_rgb(0x16, 0x1B, 0x26)
                        } else {
                            Color32::from_rgb(0x0E, 0x10, 0x16)
                        })
                        .rounding(Rounding::same(8.0))
                        .stroke(Stroke::new(
                            1.0,
                            if is_active {
                                theme.accent_pop
                            } else {
                                Color32::from_rgb(0x1E, 0x22, 0x30)
                            },
                        ))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} {}",
                                        if proj.pinned { Lucide::PIN } else { Lucide::FOLDER },
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
                                    egui::RichText::new(format!("( {})", proj.last_branch))
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
                                        action =
                                            Some(StudioModalAction::SwitchProject(proj.path.clone()));
                                    } else if is_active {
                                        ui.label(
                                            egui::RichText::new("● Đang mở")
                                                .color(theme.accent_pop)
                                                .strong()
                                                .size(10.5),
                                        );
                                    }

                                    if ui
                                        .small_button(if proj.pinned { "Bỏ ghim" } else { "Ghim" })
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

/// Phase 7: Agent & Skill Studio (Create, Edit, Delete, Preview Animated Avatars)
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
                egui::RichText::new(format!("{} AGENT & SKILL STUDIO — NOTIONISTS AVATAR GALLERY", Lucide::BOT))
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

        // 1. Live Notionists Animated Avatar Showcase for the 6 Core Elegans Agents
        Frame::none()
            .fill(Color32::from_rgb(0x10, 0x13, 0x1B))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x1F, 0x25, 0x34)))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Core Swarm Agents (Thư viện Avatar cử động từ /Volumes/Data/core_vm):")
                        .color(theme.text_silver)
                        .strong()
                        .size(11.0),
                );
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    for &ag in ALL_AGENTS {
                        Frame::none()
                            .fill(Color32::from_rgb(0x16, 0x1A, 0x25))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(Margin::symmetric(10.0, 6.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    AvatarManager::render_animated_avatar(
                                        ui,
                                        ag,
                                        AgentAvatarState::Answering,
                                        32.0,
                                        elapsed_secs,
                                        theme.accent_pop,
                                    );
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{} {}", ag.icon(), ag.name()))
                                                .color(theme.accent_pop)
                                                .strong()
                                                .size(11.0),
                                        );
                                        ui.label(
                                            egui::RichText::new(ag.title())
                                                .color(theme.text_paper)
                                                .size(10.0),
                                        );
                                    });
                                });
                            });
                    }
                });
            });

        ui.add_space(10.0);

        // 2. Editor Form to Create / Update Skill or Custom Agent
        Frame::none()
            .fill(Color32::from_rgb(0x11, 0x14, 0x1C))
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(0x22, 0x28, 0x38)))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(!editor.is_editing_agent, format!("{} Tạo / Sửa Skill (SKILL.md)", Lucide::CODE))
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
                                egui::RichText::new(format!("{} Lưu Skill vào Project", Lucide::SAVE))
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
                                egui::RichText::new(format!("{} Lưu Subagent vào Project", Lucide::SAVE))
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
                    egui::RichText::new(format!("Danh sách Skills trong Project ({})", skills.len()))
                        .color(theme.accent_pop)
                        .strong()
                        .size(11.5),
                );
                for sk in skills {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("🔧 ${}", sk.name))
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
                            if ui.small_button("✎ Sửa").clicked() {
                                editor.is_editing_agent = false;
                                editor.skill_name = sk.name.clone();
                                editor.skill_desc = sk.description.clone();
                                editor.skill_body = sk.content.clone();
                            }
                            if ui.small_button("⚡ Dùng trong Chat").clicked() {
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
                            egui::RichText::new(format!("@{}", ag.name))
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
                            if ui.small_button("✎ Sửa").clicked() {
                                editor.is_editing_agent = true;
                                editor.agent_name = ag.name.clone();
                                editor.agent_role = ag.role.clone();
                                editor.agent_prompt = ag.content.clone();
                            }
                            if ui.small_button("⚡ Triệu hồi").clicked() {
                                action = Some(StudioModalAction::InsertSkillIntoChat(format!(
                                    "@{} ",
                                    ag.name
                                )));
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
                egui::RichText::new("⚙ VICTINI SETTINGS, MEMORY REAPER & 12 CANH GIỜ")
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
            .fill(Color32::from_rgb(0x11, 0x14, 0x1C))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("1. Memory Reaper & Background Idle Scheduler")
                        .color(theme.text_paper)
                        .strong(),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Thời gian chờ trước khi đưa IDE về trạng thái Ngủ (giảm RAM < 45MB):");
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
            .fill(Color32::from_rgb(0x11, 0x14, 0x1C))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("2. Bảng Màu 12 Canh Giờ (Elegans Portfolio Palette)")
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
                                .color(if is_cur { Color32::BLACK } else { t.text_paper })
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
