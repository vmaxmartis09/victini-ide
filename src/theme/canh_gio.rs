use egui::Color32;

/// T3 Code (pingdotgg/t3code) Exact Color System + Shared Semantic Tokens:
/// Canvas: #0A0A0A (Pure dark canvas)
/// Sidebar: #000000 (Pure black OLED)
/// Surface 1 / Cards: #111111
/// Surface 2 / Elevated: #161616
/// Surface 3 / Active: #1D1D22
/// Border: #191919 / Border Strong: #28282E
/// Text: #F5F5F5 | Text Muted: #818181
/// Accent Pop: #E25B2D (T3 Ember Orange) or #346BF1 (T3 Blue)

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanhId {
    T3Ember, // T3 Code Default Dark (Pitch black #000000/#0A0A0A + Ember orange #E25B2D)
    T3Blue,  // T3 Code Electric Blue #346BF1
    T3Lime,  // T3 Code Neon Lime
}

#[derive(Clone, Debug)]
pub struct CanhTheme {
    pub id: CanhId,
    pub name: &'static str,
    pub bg_ink: Color32,         // Sidebar #000000
    pub bg_field: Color32,       // Canvas #0A0A0A
    pub panel_bg: Color32,       // Surface 1 #111111
    pub panel_hover: Color32,    // Surface 2 #161616
    pub surface_1: Color32,      // #0E0E10
    pub surface_2: Color32,      // #141418
    pub surface_3: Color32,      // #1C1C22
    pub border: Color32,         // #191919
    pub border_strong: Color32,  // #2A2A32
    pub text_paper: Color32,     // #F5F5F5
    pub text_silver: Color32,    // #818181
    pub accent_pop: Color32,     // Signature Ember #E25B2D
    pub accent_surface: Color32, // #22140F
    pub tag_red: Color32,        // #E04848
    pub success: Color32,        // #4EC9B0
    pub warning: Color32,        // #F7B955
    pub danger: Color32,         // #E05252
    pub info: Color32,           // #51A2FF
}

impl CanhTheme {
    pub fn t3_ember() -> Self {
        Self {
            id: CanhId::T3Ember,
            name: "T3 Code Ember (Default)",
            bg_ink: Color32::from_rgb(0x00, 0x00, 0x00),
            bg_field: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            panel_bg: Color32::from_rgb(0x11, 0x11, 0x11),
            panel_hover: Color32::from_rgb(0x16, 0x16, 0x16),
            surface_1: Color32::from_rgb(0x0E, 0x0E, 0x10),
            surface_2: Color32::from_rgb(0x14, 0x14, 0x18),
            surface_3: Color32::from_rgb(0x1C, 0x1C, 0x22),
            border: Color32::from_rgb(0x19, 0x19, 0x19),
            border_strong: Color32::from_rgb(0x2A, 0x2A, 0x32),
            text_paper: Color32::from_rgb(0xF5, 0xF5, 0xF5),
            text_silver: Color32::from_rgb(0x81, 0x81, 0x81),
            accent_pop: Color32::from_rgb(0xE2, 0x5B, 0x2D),
            accent_surface: Color32::from_rgb(0x22, 0x14, 0x0F),
            tag_red: Color32::from_rgb(0xE0, 0x48, 0x48),
            success: Color32::from_rgb(0x4E, 0xC9, 0xB0),
            warning: Color32::from_rgb(0xF7, 0xB9, 0x55),
            danger: Color32::from_rgb(0xE0, 0x52, 0x52),
            info: Color32::from_rgb(0x51, 0xA2, 0xFF),
        }
    }

    pub fn t3_blue() -> Self {
        Self {
            id: CanhId::T3Blue,
            name: "T3 Code Classic (Blue)",
            bg_ink: Color32::from_rgb(0x00, 0x00, 0x00),
            bg_field: Color32::from_rgb(0x0A, 0x0A, 0x0A),
            panel_bg: Color32::from_rgb(0x11, 0x11, 0x11),
            panel_hover: Color32::from_rgb(0x16, 0x16, 0x16),
            surface_1: Color32::from_rgb(0x0E, 0x0E, 0x10),
            surface_2: Color32::from_rgb(0x14, 0x14, 0x18),
            surface_3: Color32::from_rgb(0x1C, 0x1C, 0x22),
            border: Color32::from_rgb(0x19, 0x19, 0x19),
            border_strong: Color32::from_rgb(0x2A, 0x2A, 0x32),
            text_paper: Color32::from_rgb(0xF5, 0xF5, 0xF5),
            text_silver: Color32::from_rgb(0x81, 0x81, 0x81),
            accent_pop: Color32::from_rgb(0x34, 0x6B, 0xF1),
            accent_surface: Color32::from_rgb(0x10, 0x18, 0x2C),
            tag_red: Color32::from_rgb(0xE0, 0x48, 0x48),
            success: Color32::from_rgb(0x4E, 0xC9, 0xB0),
            warning: Color32::from_rgb(0xF7, 0xB9, 0x55),
            danger: Color32::from_rgb(0xE0, 0x52, 0x52),
            info: Color32::from_rgb(0x51, 0xA2, 0xFF),
        }
    }

    pub fn all_themes() -> Vec<Self> {
        vec![Self::t3_ember(), Self::t3_blue()]
    }

    pub fn dan_theme() -> Self {
        Self::t3_ember()
    }

    pub fn for_hour(_hour: u32) -> Self {
        Self::t3_ember()
    }
}
