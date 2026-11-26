use egui::{FontData, FontDefinitions, FontFamily};

pub fn configure_be_vietnam_pro_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Include Be Vietnam Pro Regular & SemiBold TTF bytes
    fonts.font_data.insert(
        "BeVietnamPro-Regular".to_owned(),
        FontData::from_static(include_bytes!("../../assets/BeVietnamPro-Regular.ttf")),
    );
    fonts.font_data.insert(
        "BeVietnamPro-SemiBold".to_owned(),
        FontData::from_static(include_bytes!("../../assets/BeVietnamPro-SemiBold.ttf")),
    );

    // Put Be Vietnam Pro first in proportional fonts
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "BeVietnamPro-Regular".to_owned());

    fonts
        .families
        .entry(FontFamily::Name("SemiBold".into()))
        .or_default()
        .insert(0, "BeVietnamPro-SemiBold".to_owned());

    ctx.set_fonts(fonts);
}
