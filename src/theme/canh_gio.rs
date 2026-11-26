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

    /// Warm Obsidian Dark Mode (Default — Agy / Claude / Linear inspired 3-tier warm obsidian)
    pub fn warm_dark() -> Self {
        Self {
            id: CanhId::WarmDark,
            name: "Tối Ấm (Warm Obsidian)",
            is_dark: true,
            bg_ink: Color32::from_rgb(0x14, 0x14, 0x16),         // #141416 Layer 0: Deep Warm Obsidian Sidebar
            bg_field: Color32::from_rgb(0x1B, 0x1B, 0x1E),       // #1B1B1E Layer 1: Main Central Canvas
            panel_bg: Color32::from_rgb(0x23, 0x23, 0x27),       // #232327 Layer 2: Elevated Panel / Card
            panel_hover: Color32::from_rgb(0x2B, 0x2B, 0x30),    // #2B2B30 Hover Surface
            surface_1: Color32::from_rgb(0x22, 0x22, 0x26),      // #222226 Container Surface
            surface_2: Color32::from_rgb(0x28, 0x28, 0x2D),      // #28282D Card / Message Capsule
            surface_3: Color32::from_rgb(0x31, 0x31, 0x37),      // #313137 Active / Selected Tab
            input_bg: Color32::from_rgb(0x16, 0x16, 0x19),       // #161619 Inset Input Field
            code_bg: Color32::from_rgb(0x13, 0x13, 0x15),        // #131315 Code Block / Pill
            chip_bg: Color32::from_rgb(0x28, 0x28, 0x2D),        // #28282D Subtle Monochrome Chip
            border: Color32::from_rgb(0x2E, 0x2E, 0x34),         // #2E2E34 1px Hairline Border
            border_strong: Color32::from_rgb(0x3E, 0x3E, 0x46),  // #3E3E46 Active Card Border
            text_paper: Color32::from_rgb(0xEC, 0xEA, 0xE4),     // #ECEAE4 Soft Ivory Primary Text
            text_silver: Color32::from_rgb(0x94, 0x92, 0x8C),    // #94928C Warm Stone Secondary Text
            accent_pop: Color32::from_rgb(0xD9, 0x77, 0x57),     // #D97757 Warm Terracotta Primary Accent
            accent_surface: Color32::from_rgba_unmultiplied(0xD9, 0x77, 0x57, 0x22),
            on_accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            tag_red: Color32::from_rgb(0xD9, 0x77, 0x57),
            tag_red_bg: Color32::from_rgba_unmultiplied(0xD9, 0x77, 0x57, 0x1E),
            success: Color32::from_rgb(0x56, 0xB6, 0x86),
            warning: Color32::from_rgb(0xDF, 0xA2, 0x48),
            danger: Color32::from_rgb(0xDE, 0x60, 0x56),
            info: Color32::from_rgb(0x6B, 0x9E, 0xDF),
            diff_add_bg: Color32::from_rgba_premultiplied(0x2E, 0x7D, 0x32, 0x2E),
            diff_add_fg: Color32::from_rgb(0x81, 0xC7, 0x84),
            diff_del_bg: Color32::from_rgba_premultiplied(0xC6, 0x28, 0x28, 0x2E),
            diff_del_fg: Color32::from_rgb(0xE5, 0x73, 0x73),
        }
    }

    /// Warm Alabaster Light Mode (Agy / Claude Warm Paper & Porcelain, zero stark glare)
    pub fn warm_light() -> Self {
        Self {
            id: CanhId::WarmLight,
            name: "Sáng Ấm (Warm Alabaster)",
            is_dark: false,
            bg_ink: Color32::from_rgb(0xEF, 0xEC, 0xE6),         // #EFECE6 Layer 0: Warm Cream Sidebar
            bg_field: Color32::from_rgb(0xF8, 0xF6, 0xF1),       // #F8F6F1 Layer 1: Warm Alabaster Canvas
            panel_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),       // #FFFFFF Layer 2: Crisp Card Surface
            panel_hover: Color32::from_rgb(0xEA, 0xE6, 0xDE),    // #EAE6DE Hover Surface
            surface_1: Color32::from_rgb(0xF2, 0xEF, 0xE9),      // #F2EFE9 Container Surface
            surface_2: Color32::from_rgb(0xFF, 0xFF, 0xFF),      // #FFFFFF Elevated Card / Bubble
            surface_3: Color32::from_rgb(0xE5, 0xE0, 0xD6),      // #E5E0D6 Active Tab
            input_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),       // #FFFFFF Clean Input Field
            code_bg: Color32::from_rgb(0xEE, 0xEB, 0xE3),        // #EEEBE3 Warm Code Pill / Block
            chip_bg: Color32::from_rgb(0xE9, 0xE5, 0xDC),        // #E9E5DC Subtle Chip
            border: Color32::from_rgb(0xE2, 0xDD, 0xD5),         // #E2DDD5 Hairline Border
            border_strong: Color32::from_rgb(0xCC, 0xC5, 0xB9),  // #CCC5B9 Defined Border
            text_paper: Color32::from_rgb(0x1F, 0x1E, 0x1B),     // #1F1E1B Deep Charcoal Ink
            text_silver: Color32::from_rgb(0x6E, 0x6A, 0x63),    // #6E6A63 Warm Umber Muted
            accent_pop: Color32::from_rgb(0xC6, 0x61, 0x42),     // #C66142 Terracotta Accent
            accent_surface: Color32::from_rgba_unmultiplied(0xC6, 0x61, 0x42, 0x1A),
            on_accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            tag_red: Color32::from_rgb(0xC6, 0x61, 0x42),
            tag_red_bg: Color32::from_rgba_unmultiplied(0xC6, 0x61, 0x42, 0x18),
            success: Color32::from_rgb(0x2D, 0x8A, 0x5B),
            warning: Color32::from_rgb(0xC2, 0x7D, 0x16),
            danger: Color32::from_rgb(0xC8, 0x42, 0x39),
            info: Color32::from_rgb(0x32, 0x6D, 0xC2),
            diff_add_bg: Color32::from_rgba_premultiplied(0x38, 0x8E, 0x3C, 0x24),
            diff_add_fg: Color32::from_rgb(0x2E, 0x7D, 0x32),
            diff_del_bg: Color32::from_rgba_premultiplied(0xD3, 0x2F, 0x2F, 0x24),
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
