use egui::Color32;

// Warm, Eye-Friendly Color System for Victini IDE:
// - Warm Dark ("Tối Ấm — Warm Espresso"): Soft warm charcoal & espresso tones (#181614 / #1F1D1A / #282521),
//   warm cream typography (#F2ECE1), muted warm stone (#A69E93), warm terracotta accent (#D97757).
// - Warm Light ("Sáng Ấm — Warm Paper"): Soft warm paper & ivory tones (#F6F3EC / #EFECE4 / #FAF8F5),
//   deep espresso ink text (#26231F), warm umber muted text (#6E675F), warm terracotta accent (#C86446).
// - Warm Dusk ("Chiều Tà — Soft Slate"): Balanced warm slate dark mode (#1B1E23 / #22262C / #2A2F37).

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanhId {
    WarmDark,  // Default Warm Espresso Dark
    WarmLight, // Warm Paper Light
    WarmDusk,  // Soft Warm Slate Dusk
}

#[derive(Clone, Debug)]
pub struct CanhTheme {
    pub id: CanhId,
    pub name: &'static str,
    pub is_dark: bool,
    pub bg_ink: Color32,         // Sidebar background
    pub bg_field: Color32,       // Main central canvas
    pub panel_bg: Color32,       // Primary panel surface
    pub panel_hover: Color32,    // Hovered surface
    pub surface_1: Color32,      // Column / section container
    pub surface_2: Color32,      // Card / elevated control
    pub surface_3: Color32,      // Active / selected card or tab
    pub input_bg: Color32,       // Text input field background
    pub code_bg: Color32,        // Code block / terminal log background
    pub chip_bg: Color32,        // Tag / Chip badge background
    pub border: Color32,         // Subtle divider border
    pub border_strong: Color32,  // Card & input border
    pub text_paper: Color32,     // Primary readable text
    pub text_silver: Color32,    // Secondary metadata text
    pub accent_pop: Color32,     // Warm terracotta / amber primary accent
    pub accent_surface: Color32, // Subtle tinted accent background
    pub on_accent: Color32,      // Text/icon color over accent
    pub tag_red: Color32,        // Project badge accent
    pub tag_red_bg: Color32,     // Project badge subtle background
    pub success: Color32,        // Green status
    pub warning: Color32,        // Amber warning
    pub danger: Color32,         // Coral red danger
    pub info: Color32,           // Calm blue info
    pub diff_add_bg: Color32,    // Diff addition line background
    pub diff_add_fg: Color32,    // Diff addition text
    pub diff_del_bg: Color32,    // Diff deletion line background
    pub diff_del_fg: Color32,    // Diff deletion text
}

impl CanhTheme {
    /// Text color to place on top of `accent_pop` buttons for high contrast.
    pub fn text_on_accent(&self) -> Color32 {
        self.on_accent
    }

    /// Warm Espresso Dark Mode (Default — cozy, zero pitch-black harshness)
    pub fn warm_dark() -> Self {
        Self {
            id: CanhId::WarmDark,
            name: "Tối Ấm (Warm Espresso)",
            is_dark: true,
            bg_ink: Color32::from_rgb(0x18, 0x16, 0x14),         // #181614 Warm Espresso Sidebar
            bg_field: Color32::from_rgb(0x1F, 0x1D, 0x1A),       // #1F1D1A Warm Charcoal Canvas
            panel_bg: Color32::from_rgb(0x25, 0x22, 0x1E),       // #25221E Panel Surface
            panel_hover: Color32::from_rgb(0x32, 0x2E, 0x29),    // #322E29 Hover
            surface_1: Color32::from_rgb(0x25, 0x22, 0x1F),      // #25221F Column Container
            surface_2: Color32::from_rgb(0x2E, 0x2A, 0x26),      // #2E2A26 Card Surface
            surface_3: Color32::from_rgb(0x38, 0x33, 0x2E),      // #38332E Active Surface
            input_bg: Color32::from_rgb(0x1A, 0x18, 0x16),       // #1A1816 Input Field
            code_bg: Color32::from_rgb(0x16, 0x15, 0x13),        // #161513 Code / Log Box
            chip_bg: Color32::from_rgb(0x2A, 0x26, 0x22),        // #2A2622 Chip badge
            border: Color32::from_rgb(0x36, 0x32, 0x2C),         // #36322C Soft Warm Border
            border_strong: Color32::from_rgb(0x48, 0x42, 0x3B),  // #48423B Card Border
            text_paper: Color32::from_rgb(0xF2, 0xEC, 0xE1),     // #F2ECE1 Warm Cream Text
            text_silver: Color32::from_rgb(0xA8, 0x9F, 0x94),    // #A89F94 Warm Stone Muted
            accent_pop: Color32::from_rgb(0xD9, 0x77, 0x57),     // #D97757 Warm Terracotta
            accent_surface: Color32::from_rgb(0x3D, 0x28, 0x20), // #3D2820 Warm Accent Tint
            on_accent: Color32::from_rgb(0xFF, 0xFB, 0xF7),
            tag_red: Color32::from_rgb(0xE0, 0x6C, 0x5A),
            tag_red_bg: Color32::from_rgba_premultiplied(0xE0, 0x6C, 0x5A, 0x2A),
            success: Color32::from_rgb(0x58, 0xB8, 0x86),
            warning: Color32::from_rgb(0xE5, 0xA9, 0x42),
            danger: Color32::from_rgb(0xE0, 0x62, 0x56),
            info: Color32::from_rgb(0x68, 0x9D, 0xE8),
            diff_add_bg: Color32::from_rgba_premultiplied(0x2E, 0x7D, 0x32, 0x33),
            diff_add_fg: Color32::from_rgb(0x81, 0xC7, 0x84),
            diff_del_bg: Color32::from_rgba_premultiplied(0xC6, 0x28, 0x28, 0x33),
            diff_del_fg: Color32::from_rgb(0xE5, 0x73, 0x73),
        }
    }

    /// Warm Paper Light Mode (Cozy ivory & sand tones, zero harsh #FFFFFF glare)
    pub fn warm_light() -> Self {
        Self {
            id: CanhId::WarmLight,
            name: "Sáng Ấm (Warm Paper)",
            is_dark: false,
            bg_ink: Color32::from_rgb(0xEB, 0xE5, 0xDA),         // #EBE5DA Warm Oat Sidebar
            bg_field: Color32::from_rgb(0xF5, 0xF1, 0xE8),       // #F5F1E8 Warm Paper Canvas
            panel_bg: Color32::from_rgb(0xEE, 0xE9, 0xDF),       // #EEE9DF Panel Surface
            panel_hover: Color32::from_rgb(0xE4, 0xDE, 0xD2),    // #E4DED2 Hover
            surface_1: Color32::from_rgb(0xEC, 0xE7, 0xDC),      // #ECE7DC Column Container
            surface_2: Color32::from_rgb(0xFA, 0xF7, 0xF2),      // #FAF7F2 Ivory Card Surface
            surface_3: Color32::from_rgb(0xE2, 0xDC, 0xCF),      // #E2DCCF Active Surface
            input_bg: Color32::from_rgb(0xFC, 0xFA, 0xF6),       // #FCFAF6 Crisp Ivory Input
            code_bg: Color32::from_rgb(0xEA, 0xE5, 0xDB),        // #EAE5DB Warm Code Box
            chip_bg: Color32::from_rgb(0xE3, 0xDD, 0xD0),        // #E3DDD0 Chip badge
            border: Color32::from_rgb(0xD8, 0xD1, 0xC4),         // #D8D1C4 Soft Sand Border
            border_strong: Color32::from_rgb(0xC2, 0xB9, 0xA9),  // #C2B9A9 Defined Card Border
            text_paper: Color32::from_rgb(0x26, 0x22, 0x1E),     // #26221E Deep Espresso Ink
            text_silver: Color32::from_rgb(0x6E, 0x66, 0x5D),    // #6E665D Warm Umber Grey
            accent_pop: Color32::from_rgb(0xC6, 0x61, 0x42),     // #C66142 Terracotta Accent
            accent_surface: Color32::from_rgb(0xF3, 0xDF, 0xD7), // #F3DFD7 Soft Peach Tint
            on_accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            tag_red: Color32::from_rgb(0xC4, 0x46, 0x3A),
            tag_red_bg: Color32::from_rgba_premultiplied(0xC4, 0x46, 0x3A, 0x20),
            success: Color32::from_rgb(0x2D, 0x8A, 0x5B),
            warning: Color32::from_rgb(0xC2, 0x7D, 0x16),
            danger: Color32::from_rgb(0xC8, 0x42, 0x39),
            info: Color32::from_rgb(0x32, 0x6D, 0xC2),
            diff_add_bg: Color32::from_rgba_premultiplied(0x38, 0x8E, 0x3C, 0x28),
            diff_add_fg: Color32::from_rgb(0x2E, 0x7D, 0x32),
            diff_del_bg: Color32::from_rgba_premultiplied(0xD3, 0x2F, 0x2F, 0x28),
            diff_del_fg: Color32::from_rgb(0xC6, 0x28, 0x28),
        }
    }

    /// Warm Dusk Mode (Soft slate & amber)
    pub fn warm_dusk() -> Self {
        Self {
            id: CanhId::WarmDusk,
            name: "Chiều Tà (Warm Dusk)",
            is_dark: true,
            bg_ink: Color32::from_rgb(0x18, 0x1B, 0x20),
            bg_field: Color32::from_rgb(0x1E, 0x22, 0x28),
            panel_bg: Color32::from_rgb(0x25, 0x29, 0x30),
            panel_hover: Color32::from_rgb(0x30, 0x35, 0x3E),
            surface_1: Color32::from_rgb(0x24, 0x28, 0x30),
            surface_2: Color32::from_rgb(0x2C, 0x31, 0x3A),
            surface_3: Color32::from_rgb(0x35, 0x3B, 0x46),
            input_bg: Color32::from_rgb(0x1A, 0x1D, 0x22),
            code_bg: Color32::from_rgb(0x16, 0x19, 0x1D),
            chip_bg: Color32::from_rgb(0x28, 0x2E, 0x38),
            border: Color32::from_rgb(0x34, 0x3A, 0x44),
            border_strong: Color32::from_rgb(0x45, 0x4D, 0x59),
            text_paper: Color32::from_rgb(0xEE, 0xE9, 0xE0),
            text_silver: Color32::from_rgb(0x9E, 0xA4, 0xAE),
            accent_pop: Color32::from_rgb(0xDF, 0x84, 0x53),
            accent_surface: Color32::from_rgb(0x3B, 0x2B, 0x22),
            on_accent: Color32::from_rgb(0xFF, 0xFB, 0xF7),
            tag_red: Color32::from_rgb(0xE0, 0x65, 0x5A),
            tag_red_bg: Color32::from_rgba_premultiplied(0xE0, 0x65, 0x5A, 0x2A),
            success: Color32::from_rgb(0x56, 0xB8, 0x8A),
            warning: Color32::from_rgb(0xE5, 0xAA, 0x44),
            danger: Color32::from_rgb(0xE0, 0x62, 0x58),
            info: Color32::from_rgb(0x66, 0x9E, 0xE8),
            diff_add_bg: Color32::from_rgba_premultiplied(0x2E, 0x7D, 0x32, 0x33),
            diff_add_fg: Color32::from_rgb(0x81, 0xC7, 0x84),
            diff_del_bg: Color32::from_rgba_premultiplied(0xC6, 0x28, 0x28, 0x33),
            diff_del_fg: Color32::from_rgb(0xE5, 0x73, 0x73),
        }
    }

    /// Toggle seamlessly between Dark and Light mode
    pub fn toggle_light_dark(&self) -> Self {
        if self.is_dark {
            Self::warm_light()
        } else {
            Self::warm_dark()
        }
    }

    pub fn all_themes() -> Vec<Self> {
        vec![Self::warm_dark(), Self::warm_light(), Self::warm_dusk()]
    }

    pub fn dan_theme() -> Self {
        Self::warm_dark()
    }

    pub fn t3_ember() -> Self {
        Self::warm_dark()
    }

    pub fn t3_blue() -> Self {
        Self::warm_dark()
    }

    pub fn t3_paper_light() -> Self {
        Self::warm_light()
    }

    pub fn t3_daylight_blue() -> Self {
        Self::warm_light()
    }

    pub fn for_hour(hour: u32) -> Self {
        if (7..18).contains(&hour) {
            Self::warm_light()
        } else {
            Self::warm_dark()
        }
    }
}
