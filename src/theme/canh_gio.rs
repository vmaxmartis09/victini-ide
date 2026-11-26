use egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanhId {
    Ty,
    Suu,
    Dan,
    Mao,
    Thin,
    Ti,
    Ngo,
    Mui,
    Than,
    Dau,
    Tuat,
    Hoi,
}

#[derive(Clone, Debug)]
pub struct CanhTheme {
    pub id: CanhId,
    pub name: &'static str,
    pub zodiac: &'static str,
    pub time_range: &'static str,
    pub bg_ink: Color32,
    pub bg_field: Color32,
    pub panel_bg: Color32,
    pub text_paper: Color32,
    pub text_silver: Color32,
    pub accent_pop: Color32,
}

impl CanhTheme {
    pub fn dan_theme() -> Self {
        Self {
            id: CanhId::Dan,
            name: "Giờ Dần (Qoder/T3 Slate)",
            zodiac: "Hổ",
            time_range: "03h – 05h",
            bg_ink: Color32::from_rgb(0x06, 0x07, 0x09),
            bg_field: Color32::from_rgb(0x0B, 0x0C, 0x10),
            panel_bg: Color32::from_rgb(0x11, 0x13, 0x1A),
            text_paper: Color32::from_rgb(0xEE, 0xF1, 0xF5),
            text_silver: Color32::from_rgb(0x8D, 0x94, 0x9E),
            accent_pop: Color32::from_rgb(0xC8, 0xFE, 0x04),
        }
    }

    pub fn all_themes() -> Vec<Self> {
        vec![
            Self {
                id: CanhId::Ty,
                name: "Giờ Tý",
                zodiac: "Chuột",
                time_range: "23h – 01h",
                bg_ink: Color32::from_rgb(0x05, 0x09, 0x11),
                bg_field: Color32::from_rgb(0x09, 0x11, 0x1E),
                panel_bg: Color32::from_rgb(0x10, 0x19, 0x29),
                text_paper: Color32::from_rgb(0xE8, 0xF5, 0xFA),
                text_silver: Color32::from_rgb(0x7A, 0x98, 0xB0),
                accent_pop: Color32::from_rgb(0xFF, 0xC8, 0x57),
            },
            Self {
                id: CanhId::Suu,
                name: "Giờ Sửu",
                zodiac: "Trâu",
                time_range: "01h – 03h",
                bg_ink: Color32::from_rgb(0x07, 0x08, 0x0C),
                bg_field: Color32::from_rgb(0x0D, 0x0F, 0x15),
                panel_bg: Color32::from_rgb(0x13, 0x16, 0x1F),
                text_paper: Color32::from_rgb(0xEC, 0xF0, 0xF6),
                text_silver: Color32::from_rgb(0x88, 0x92, 0xA0),
                accent_pop: Color32::from_rgb(0x61, 0xAF, 0xEF),
            },
            Self::dan_theme(),
            Self {
                id: CanhId::Mao,
                name: "Giờ Mão",
                zodiac: "Mèo",
                time_range: "05h – 07h",
                bg_ink: Color32::from_rgb(0x07, 0x0B, 0x0E),
                bg_field: Color32::from_rgb(0x0B, 0x12, 0x16),
                panel_bg: Color32::from_rgb(0x11, 0x1A, 0x20),
                text_paper: Color32::from_rgb(0xEE, 0xF6, 0xF5),
                text_silver: Color32::from_rgb(0x84, 0x9C, 0x9A),
                accent_pop: Color32::from_rgb(0x4E, 0xC9, 0xB0),
            },
            Self {
                id: CanhId::Thin,
                name: "Giờ Thìn",
                zodiac: "Rồng",
                time_range: "07h – 09h",
                bg_ink: Color32::from_rgb(0x09, 0x09, 0x0F),
                bg_field: Color32::from_rgb(0x0E, 0x10, 0x18),
                panel_bg: Color32::from_rgb(0x15, 0x18, 0x24),
                text_paper: Color32::from_rgb(0xF5, 0xF5, 0xFA),
                text_silver: Color32::from_rgb(0x92, 0x98, 0xAC),
                accent_pop: Color32::from_rgb(0xF7, 0xB9, 0x55),
            },
            Self {
                id: CanhId::Ti,
                name: "Giờ Tỵ",
                zodiac: "Rắn",
                time_range: "09h – 11h",
                bg_ink: Color32::from_rgb(0x08, 0x0A, 0x0D),
                bg_field: Color32::from_rgb(0x0D, 0x11, 0x16),
                panel_bg: Color32::from_rgb(0x14, 0x19, 0x21),
                text_paper: Color32::from_rgb(0xF0, 0xF4, 0xF8),
                text_silver: Color32::from_rgb(0x8A, 0x96, 0xA4),
                accent_pop: Color32::from_rgb(0x98, 0xC3, 0x79),
            },
            Self {
                id: CanhId::Ngo,
                name: "Giờ Ngọ",
                zodiac: "Ngựa",
                time_range: "11h – 13h",
                bg_ink: Color32::from_rgb(0x0D, 0x07, 0x08),
                bg_field: Color32::from_rgb(0x15, 0x0B, 0x0D),
                panel_bg: Color32::from_rgb(0x1F, 0x11, 0x14),
                text_paper: Color32::from_rgb(0xF8, 0xF4, 0xEE),
                text_silver: Color32::from_rgb(0xA6, 0x8C, 0x85),
                accent_pop: Color32::from_rgb(0xFF, 0x7B, 0x72),
            },
            Self {
                id: CanhId::Mui,
                name: "Giờ Mùi",
                zodiac: "Dê",
                time_range: "13h – 15h",
                bg_ink: Color32::from_rgb(0x09, 0x09, 0x0C),
                bg_field: Color32::from_rgb(0x10, 0x10, 0x15),
                panel_bg: Color32::from_rgb(0x17, 0x17, 0x1F),
                text_paper: Color32::from_rgb(0xF2, 0xF1, 0xF6),
                text_silver: Color32::from_rgb(0x95, 0x92, 0xA2),
                accent_pop: Color32::from_rgb(0xE5, 0xC0, 0x7B),
            },
            Self {
                id: CanhId::Than,
                name: "Giờ Thân",
                zodiac: "Khỉ",
                time_range: "15h – 17h",
                bg_ink: Color32::from_rgb(0x07, 0x09, 0x0E),
                bg_field: Color32::from_rgb(0x0C, 0x0F, 0x17),
                panel_bg: Color32::from_rgb(0x13, 0x17, 0x22),
                text_paper: Color32::from_rgb(0xEE, 0xF2, 0xF7),
                text_silver: Color32::from_rgb(0x8B, 0x95, 0xA5),
                accent_pop: Color32::from_rgb(0x56, 0xB6, 0xC2),
            },
            Self {
                id: CanhId::Dau,
                name: "Giờ Dậu",
                zodiac: "Gà",
                time_range: "17h – 19h",
                bg_ink: Color32::from_rgb(0x09, 0x08, 0x0D),
                bg_field: Color32::from_rgb(0x11, 0x0E, 0x18),
                panel_bg: Color32::from_rgb(0x19, 0x14, 0x23),
                text_paper: Color32::from_rgb(0xF4, 0xF0, 0xF8),
                text_silver: Color32::from_rgb(0x9A, 0x8F, 0xA8),
                accent_pop: Color32::from_rgb(0xC6, 0x78, 0xDD),
            },
            Self {
                id: CanhId::Tuat,
                name: "Giờ Tuất",
                zodiac: "Chó",
                time_range: "19h – 21h",
                bg_ink: Color32::from_rgb(0x06, 0x08, 0x0B),
                bg_field: Color32::from_rgb(0x0B, 0x0E, 0x13),
                panel_bg: Color32::from_rgb(0x12, 0x16, 0x1E),
                text_paper: Color32::from_rgb(0xED, 0xF2, 0xF6),
                text_silver: Color32::from_rgb(0x87, 0x93, 0xA0),
                accent_pop: Color32::from_rgb(0xC8, 0xFE, 0x04),
            },
            Self {
                id: CanhId::Hoi,
                name: "Giờ Hợi",
                zodiac: "Lợn",
                time_range: "21h – 23h",
                bg_ink: Color32::from_rgb(0x06, 0x09, 0x10),
                bg_field: Color32::from_rgb(0x0B, 0x11, 0x1C),
                panel_bg: Color32::from_rgb(0x12, 0x1A, 0x28),
                text_paper: Color32::from_rgb(0xF2, 0xF4, 0xF8),
                text_silver: Color32::from_rgb(0x7E, 0x8C, 0xA0),
                accent_pop: Color32::from_rgb(0x52, 0xC4, 0xA0),
            },
        ]
    }

    pub fn for_hour(hour: u32) -> Self {
        let index = (((hour + 1) % 24) / 2) as usize;
        let all = Self::all_themes();
        all.into_iter().nth(index).unwrap_or_else(Self::dan_theme)
    }
}
