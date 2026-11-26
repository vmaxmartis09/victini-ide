use crate::config::{DiscoveredAgent, DiscoveredSkill};
use crate::git::{GitFileChange, WorktreeInfo};
use crate::theme::CanhTheme;
use egui::{vec2, Color32, Frame, Rounding, ScrollArea, Stroke, Ui};

pub enum GitRailAction {
    StageAll,
    Commit,
    AddSkill,
    CreateAgent,
}

pub fn render_left_git_rail(
    ui: &mut Ui,
    theme: &CanhTheme,
    worktrees: &[WorktreeInfo],
    file_changes: &[GitFileChange],
    skills: &[DiscoveredSkill],
    agents: &[DiscoveredAgent],
) -> Option<GitRailAction> {
    let mut action = None;
    ui.vertical(|ui| {
        // Section: Worktrees & Branches (Card styling)
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("WORKTREES")
                .color(theme.text_silver)
                .size(10.5)
                .strong(),
        );
        ui.add_space(4.0);

        for wt in worktrees {
            let is_head = wt.is_head;
            let bg = if is_head { Color32::from_rgb(0x18, 0x1A, 0x22) } else { Color32::TRANSPARENT };
            let border = if is_head { Stroke::new(1.0, theme.accent_pop) } else { Stroke::NONE };

            Frame::none()
                .fill(bg)
                .rounding(Rounding::same(6.0))
                .stroke(border)
                .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let dot_col = if is_head { theme.accent_pop } else { theme.text_silver };
                        ui.label(egui::RichText::new("●").color(dot_col).size(9.0));
                        ui.label(
                            egui::RichText::new(&wt.name)
                                .color(if is_head { theme.text_paper } else { theme.text_silver })
                                .size(11.5),
                        );
                    });
                });
            ui.add_space(2.0);
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // Section: Git Changes
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("CHANGES")
                    .color(theme.text_silver)
                    .size(10.5)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("Stage All").size(10.5)).clicked() {
                    action = Some(GitRailAction::StageAll);
                }
            });
        });

        ui.add_space(4.0);
        ScrollArea::vertical()
            .id_salt("git_changes_scroll")
            .max_height(140.0)
            .show(ui, |ui| {
                if file_changes.is_empty() {
                    ui.label(
                        egui::RichText::new("Tree clean")
                            .color(theme.text_silver)
                            .italics()
                            .size(11.0),
                    );
                } else {
                    for fc in file_changes {
                        let status_color = match fc.status.as_str() {
                            "A" => Color32::from_rgb(0x4E, 0xC9, 0xB0),
                            "M" => Color32::from_rgb(0xE5, 0xC0, 0x7B),
                            "D" => Color32::from_rgb(0xE0, 0x6C, 0x75),
                            _ => theme.text_silver,
                        };
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&fc.status).color(status_color).strong().size(11.0));
                            ui.label(egui::RichText::new(&fc.path).color(theme.text_paper).size(11.0));
                        });
                    }
                }
            });

        ui.add_space(8.0);
        if ui
            .add_sized(
                vec2(ui.available_width(), 28.0),
                egui::Button::new(
                    egui::RichText::new("⚡ Quick Commit & MR")
                        .color(Color32::BLACK)
                        .strong()
                        .size(11.5),
                ).fill(theme.accent_pop).rounding(Rounding::same(6.0)),
            )
            .clicked()
        {
            action = Some(GitRailAction::Commit);
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);

        // Section: Project Skills
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("PROJECT SKILLS")
                    .color(theme.text_silver)
                    .size(10.5)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("+ Skill").size(10.0)).clicked() {
                    action = Some(GitRailAction::AddSkill);
                }
            });
        });

        ui.add_space(4.0);
        ScrollArea::vertical()
            .id_salt("project_skills_scroll")
            .max_height(100.0)
            .show(ui, |ui| {
                if skills.is_empty() {
                    ui.label(egui::RichText::new("No custom skills").color(theme.text_silver).size(10.5));
                } else {
                    for sk in skills {
                        ui.label(egui::RichText::new(format!("🔧 {}", sk.name)).color(theme.text_paper).size(11.0));
                    }
                }
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("SUBAGENTS")
                    .color(theme.text_silver)
                    .size(10.5)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("+ Agent").size(10.0)).clicked() {
                    action = Some(GitRailAction::CreateAgent);
                }
            });
        });

        ui.add_space(4.0);
        ScrollArea::vertical()
            .id_salt("project_agents_scroll")
            .max_height(90.0)
            .show(ui, |ui| {
                if agents.is_empty() {
                    ui.label(egui::RichText::new("Elegans Swarm Active").color(theme.accent_pop).size(10.5));
                } else {
                    for ag in agents {
                        ui.label(egui::RichText::new(format!("🤖 {}", ag.name)).color(theme.text_paper).size(11.0));
                    }
                }
            });
    });
    action
}
