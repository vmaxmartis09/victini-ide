use egui::{FontData, FontDefinitions, FontFamily};

pub fn configure_be_vietnam_pro_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // 1. Include Be Vietnam Pro Regular & SemiBold TTF bytes
    fonts.font_data.insert(
        "BeVietnamPro-Regular".to_owned(),
        FontData::from_static(include_bytes!("../../assets/BeVietnamPro-Regular.ttf")),
    );
    fonts.font_data.insert(
        "BeVietnamPro-SemiBold".to_owned(),
        FontData::from_static(include_bytes!("../../assets/BeVietnamPro-SemiBold.ttf")),
    );
    // 2. Include Lucide Icons TTF
    fonts.font_data.insert(
        "Lucide".to_owned(),
        FontData::from_static(include_bytes!("../../assets/lucide.ttf")),
    );

    // 3. Put Be Vietnam Pro first in proportional fonts, Lucide second for icon glyph fallbacks
    if let Some(prop) = fonts.families.get_mut(&FontFamily::Proportional) {
        prop.insert(0, "Lucide".to_owned());
        prop.insert(0, "BeVietnamPro-Regular".to_owned());
    } else {
        fonts
            .families
            .insert(FontFamily::Proportional, vec!["BeVietnamPro-Regular".to_owned(), "Lucide".to_owned()]);
    }

    if let Some(mono) = fonts.families.get_mut(&FontFamily::Monospace) {
        mono.insert(0, "Lucide".to_owned());
        mono.insert(0, "BeVietnamPro-Regular".to_owned());
    }

    fonts
        .families
        .entry(FontFamily::Name("SemiBold".into()))
        .or_default()
        .insert(0, "BeVietnamPro-SemiBold".to_owned());

    ctx.set_fonts(fonts);
}
