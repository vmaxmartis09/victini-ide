use egui::Color32;

/// T3 Code (pingdotgg/t3code) Exact Color System:
/// Canvas: #0A0A0A (Pure dark canvas)
/// Sidebar: #000000 (Pure black OLED)
/// Surface / Cards: #111111 (Subtle dark cards)
/// Surface Hover: #141414
/// Border: #191919 (Ultra-refined hairline borders)
/// Text: #F5F5F5 (Crisp typography)
/// Text Muted: #818181 (Clean secondary metadata)
/// Accent Pop: #E25B2D (Signature T3 Ember Orange) or #346BF1 (T3 Blue)

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
    pub bg_ink: Color32,       // Sidebar #000000
    pub bg_field: Color32,     // Canvas #0A0A0A
    pub panel_bg: Color32,     // Surface #111111
    pub panel_hover: Color32,  // #141414
    pub border: Color32,       // #191919
    pub text_paper: Color32,   // #F5F5F5
    pub text_silver: Color32,  // #818181
    pub accent_pop: Color32,   // Signature Ember #E25B2D
    pub accent_surface: Color32, // #20130E
    pub tag_red: Color32,      // #FB414A (Red pill for repo/worktree)
}

impl CanhTheme {
    pub fn t3_ember() -> Self {
        Self {
            id: CanhId::T3Ember,
            name: "T3 Code Ember (Default)",
            bg_ink: Color32::from_rgb(0x00, 0x00, 0x00),       // #000000 Pure Black Sidebar
            bg_field: Color32::from_rgb(0x0A, 0x0A, 0x0A),     // #0A0A0A Canvas
            panel_bg: Color32::from_rgb(0x11, 0x11, 0x11),     // #111111 Surface
            panel_hover: Color32::from_rgb(0x16, 0x16, 0x16),  // #161616 Hover
            border: Color32::from_rgb(0x19, 0x19, 0x19),       // #191919 Refined Hairline
            text_paper: Color32::from_rgb(0xF5, 0xF5, 0xF5),   // #F5F5F5
            text_silver: Color32::from_rgb(0x81, 0x81, 0x81),  // #818181
            accent_pop: Color32::from_rgb(0xE2, 0x5B, 0x2D),   // #E25B2D Ember Orange
            accent_surface: Color32::from_rgb(0x22, 0x14, 0x0F),
            tag_red: Color32::from_rgb(0xE0, 0x48, 0x48),      // Red PN tag
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
            border: Color32::from_rgb(0x19, 0x19, 0x19),
            text_paper: Color32::from_rgb(0xF5, 0xF5, 0xF5),
            text_silver: Color32::from_rgb(0x81, 0x81, 0x81),
            accent_pop: Color32::from_rgb(0x34, 0x6B, 0xF1),   // #346BF1
            accent_surface: Color32::from_rgb(0x10, 0x18, 0x2C),
            tag_red: Color32::from_rgb(0xE0, 0x48, 0x48),
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
