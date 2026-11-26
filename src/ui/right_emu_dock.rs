use crate::emulator::{EmulatorState, ELEGANS_VIEWPORT_PRESETS};
use crate::theme::CanhTheme;
use egui::{vec2, Align2, Color32, FontId, Frame, Margin, Rounding, Stroke, Ui};

pub fn render_right_emu_dock(
    ui: &mut Ui,
    theme: &CanhTheme,
    emu_state: &mut EmulatorState,
    mut on_trigger_tap: impl FnMut(u32, u32),
    mut on_switch_preset: impl FnMut(usize),
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("REMOTE EMU & VERIFY")
                    .color(theme.text_silver)
                    .strong()
                    .size(10.5),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let badge = if emu_state.is_adb_mode { "ADB" } else { "CDP 140+" };
                ui.label(
                    egui::RichText::new(badge)
                        .color(theme.accent_pop)
                        .size(10.0),
                );
            });
        });

        ui.add_space(6.0);

        // Viewport Selector Chips
        ui.horizontal(|ui| {
            for (idx, preset) in ELEGANS_VIEWPORT_PRESETS.iter().enumerate() {
                let is_sel = emu_state.active_preset_idx == idx;
                let btn = egui::Button::new(
                    egui::RichText::new(preset.name.split('(').next().unwrap_or(preset.name))
                        .size(10.0)
                        .color(if is_sel { Color32::BLACK } else { theme.text_paper }),
                )
                .fill(if is_sel { theme.accent_pop } else { Color32::from_rgb(0x16, 0x18, 0x22) })
                .rounding(Rounding::same(4.0));

                if ui.add(btn).clicked() {
                    emu_state.active_preset_idx = idx;
                    on_switch_preset(idx);
                }
            }
        });

        ui.add_space(8.0);

        // Emulator Frame Canvas (Rounded Phone View)
        let current_preset = ELEGANS_VIEWPORT_PRESETS[emu_state.active_preset_idx];
        let aspect = current_preset.width as f32 / current_preset.height as f32;
        let emu_w = ui.available_width().min(260.0);
        let emu_h = emu_w / aspect;

        Frame::none()
            .fill(Color32::from_rgb(0x07, 0x08, 0x0C))
            .rounding(Rounding::same(24.0))
            .stroke(Stroke::new(1.5, Color32::from_rgb(0x2A, 0x30, 0x40)))
            .inner_margin(Margin::same(8.0))
            .show(ui, |ui| {
                let (rect, resp) = ui.allocate_exact_size(vec2(emu_w, emu_h.min(440.0)), egui::Sense::click_and_drag());

                if resp.clicked() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        let rel_x = (pos.x - rect.min.x) / rect.width();
                        let rel_y = (pos.y - rect.min.y) / rect.height();
                        let tap_x = (rel_x * current_preset.width as f32) as u32;
                        let tap_y = (rel_y * current_preset.height as f32) as u32;
                        on_trigger_tap(tap_x, tap_y);
                    }
                }

                let painter = ui.painter_at(rect);
                // Simulated mobile screen preview
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{}\n{}×{}\nTouch Forwarding Active", current_preset.name, current_preset.width, current_preset.height),
                    FontId::proportional(11.5),
                    theme.text_silver,
                );
            });

        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!("Trạng thái: {}", emu_state.status_text))
                .color(theme.text_silver)
                .size(10.5),
        );
    });
}
