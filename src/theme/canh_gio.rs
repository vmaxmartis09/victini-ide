use egui::Color32;

// ============================================================================
// ELEGANS · VICIEO SIGNATURE UI CRAFT TOKENS (VICIEO-UI-01 / 01B / 01C)
// - Framed-Sheet Architecture: Shell Ground (hsl(38 36% 96%) / hsl(36 14% 7%))
//   + Working Sheet inset by 8px, 1px border, 16px concentric radius.
// - Strict 4px Spacing Grid: 4, 8, 12, 16, 20, 24px (Zero .5 or 7/9/11 steps).
// - 7-Step Typography Scale (>= 11px minimum, zero 9px/10px).
// - Concentric Border Radius: R_inner = R_outer - Padding (24 -> 16 -> 12 -> 8 -> 4).
// - Primary CTA in Ink color; Brand Accent strictly < 3% screen area.
// ============================================================================

pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_5: f32 = 20.0;
pub const SPACE_6: f32 = 24.0;

pub const ROUND_SHEET: f32 = 24.0;
pub const ROUND_PANEL: f32 = 16.0;
pub const ROUND_CARD: f32 = 12.0;
pub const ROUND_CTL: f32 = 8.0;
pub const ROUND_XS: f32 = 4.0;

pub const FONT_CAPTION: f32 = 11.0;
pub const FONT_FOOTNOTE: f32 = 12.0;
pub const FONT_BODY: f32 = 13.0;
pub const FONT_CALLOUT: f32 = 14.0;
pub const FONT_HEADLINE: f32 = 15.0;
pub const FONT_TITLE: f32 = 17.0;
pub const FONT_LARGETITLE: f32 = 22.0;

pub const CTL_SM: f32 = 24.0;      // h-6
pub const CTL_DEFAULT: f32 = 28.0; // h-7 (Elegans Default Control Height)
pub const CTL_LG: f32 = 36.0;      // h-9

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanhId {
    WarmDark,  // Deep Ink Framed-Sheet Dark (hsl(36 14% 7%))
    WarmLight, // Warm Stone Framed-Sheet Light (hsl(38 36% 96%))
    WarmDusk,  // Soft Slate Framed-Sheet Dusk
}

#[derive(Clone, Debug)]
pub struct CanhTheme {
    pub id: CanhId,
    pub name: &'static str,
    pub is_dark: bool,
    pub bg_ink: Color32,         // Outer Shell / Sidebar ground
    pub bg_field: Color32,       // Framed-Sheet main working surface
    pub panel_bg: Color32,       // Card surface (Flat at rest)
    pub panel_hover: Color32,    // Hovered row/control surface
    pub surface_1: Color32,      // Subtle recessed/elevated container
    pub surface_2: Color32,      // Active control / User message capsule
    pub surface_3: Color32,      // Floating Popover / Flyout surface
    pub input_bg: Color32,       // Text input background
    pub code_bg: Color32,        // Code block / inline pill background
    pub chip_bg: Color32,        // Monochrome control chip
    pub border: Color32,         // 1px hairline border
    pub border_strong: Color32,  // Active / focused 1px border
    pub text_paper: Color32,     // Primary Ink text & Primary Ink CTA fill
    pub text_silver: Color32,    // Secondary muted text
    pub accent_pop: Color32,     // Brand accent (< 3% area: active indicator, focus ring, running dot)
    pub accent_surface: Color32, // Subtle 10% tint of brand accent
    pub on_accent: Color32,      // Text over brand accent
    pub tag_red: Color32,        // Semantic badge foreground
    pub tag_red_bg: Color32,     // Semantic badge background
    pub success: Color32,        // Semantic pass/staged green
    pub warning: Color32,        // Semantic warning amber
    pub danger: Color32,         // Semantic danger red
    pub info: Color32,           // Semantic info blue
    pub diff_add_bg: Color32,
    pub diff_add_fg: Color32,
    pub diff_del_bg: Color32,
    pub diff_del_fg: Color32,
}

impl CanhTheme {
    pub fn text_on_accent(&self) -> Color32 {
        self.on_accent
    }

    /// Primary Ink CTA text color (contrasts against `self.text_paper` button fill per Elegans rule)
    pub fn text_on_ink(&self) -> Color32 {
        self.bg_field
    }

    /// Elegans Deep Ink Framed-Sheet Dark Mode (`hsl(36 14% 7%)` shell + `#1A1917` sheet)
    pub fn warm_dark() -> Self {
        Self {
            id: CanhId::WarmDark,
            name: "Mực Sâu (Elegans Deep Ink)",
            is_dark: true,
            bg_ink: Color32::from_rgb(0x13, 0x12, 0x10),         // Outer Shell Ground hsl(36 14% 7%)
            bg_field: Color32::from_rgb(0x1A, 0x19, 0x17),       // Framed-Sheet Working Surface
            panel_bg: Color32::from_rgb(0x21, 0x1F, 0x1C),       // Flat-at-Rest Card Surface
            panel_hover: Color32::from_rgb(0x29, 0x27, 0x23),    // Hover State
            surface_1: Color32::from_rgb(0x21, 0x1F, 0x1C),      // Subtle Container
            surface_2: Color32::from_rgb(0x2A, 0x27, 0x23),      // Active Pill / User Capsule
            surface_3: Color32::from_rgb(0x32, 0x2F, 0x2A),      // Popover Surface
            input_bg: Color32::from_rgb(0x16, 0x15, 0x13),       // Inset Input
            code_bg: Color32::from_rgb(0x14, 0x13, 0x11),        // Code Block
            chip_bg: Color32::from_rgb(0x26, 0x24, 0x20),        // Monochrome Chip
            border: Color32::from_rgb(0x2C, 0x29, 0x25),         // 1px Hairline Border
            border_strong: Color32::from_rgb(0x3D, 0x39, 0x33),  // Focused / Active Border
            text_paper: Color32::from_rgb(0xED, 0xEA, 0xE2),     // Warm Ivory Ink
            text_silver: Color32::from_rgb(0x96, 0x91, 0x88),    // Warm Stone Muted
            accent_pop: Color32::from_rgb(0xD9, 0x77, 0x57),     // < 3% Brand Accent
            accent_surface: Color32::from_rgba_unmultiplied(0xD9, 0x77, 0x57, 0x1E),
            on_accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            tag_red: Color32::from_rgb(0xD9, 0x77, 0x57),
            tag_red_bg: Color32::from_rgba_unmultiplied(0xD9, 0x77, 0x57, 0x1A),
            success: Color32::from_rgb(0x52, 0xB2, 0x82),
            warning: Color32::from_rgb(0xD9, 0x9E, 0x46),
            danger: Color32::from_rgb(0xDC, 0x5E, 0x54),
            info: Color32::from_rgb(0x68, 0x9A, 0xD8),
            diff_add_bg: Color32::from_rgba_premultiplied(0x2E, 0x7D, 0x32, 0x2A),
            diff_add_fg: Color32::from_rgb(0x81, 0xC7, 0x84),
            diff_del_bg: Color32::from_rgba_premultiplied(0xC6, 0x28, 0x28, 0x2A),
            diff_del_fg: Color32::from_rgb(0xE5, 0x73, 0x73),
        }
    }

    /// Elegans Warm Stone Framed-Sheet Light Mode (`hsl(38 36% 96%)` shell + `#FCFBF8` sheet)
    pub fn warm_light() -> Self {
        Self {
            id: CanhId::WarmLight,
            name: "Giấy Ấm (Elegans Framed-Sheet)",
            is_dark: false,
            bg_ink: Color32::from_rgb(0xF3, 0xEF, 0xE6),         // Outer Warm Stone Shell hsl(38 36% 93%)
            bg_field: Color32::from_rgb(0xFC, 0xFB, 0xF8),       // Framed-Sheet Working Surface
            panel_bg: Color32::from_rgb(0xF6, 0xF3, 0xEC),       // Flat-at-Rest Card Surface
            panel_hover: Color32::from_rgb(0xEC, 0xE7, 0xDD),    // Hover State
            surface_1: Color32::from_rgb(0xF4, 0xF1, 0xEA),      // Subtle Container
            surface_2: Color32::from_rgb(0xE9, 0xE4, 0xD9),      // Active Pill / User Capsule
            surface_3: Color32::from_rgb(0xFF, 0xFF, 0xFF),      // Popover Surface
            input_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),       // Crisp Input
            code_bg: Color32::from_rgb(0xF2, 0xEE, 0xE5),        // Code Block
            chip_bg: Color32::from_rgb(0xEC, 0xE7, 0xDD),        // Monochrome Chip
            border: Color32::from_rgb(0xE3, 0xDD, 0xD2),         // 1px Hairline Border
            border_strong: Color32::from_rgb(0xC9, 0xC1, 0xB4),  // Focused / Active Border
            text_paper: Color32::from_rgb(0x1C, 0x1A, 0x17),     // Deep Charcoal Ink
            text_silver: Color32::from_rgb(0x6E, 0x69, 0x60),    // Warm Umber Muted
            accent_pop: Color32::from_rgb(0xC4, 0x5D, 0x3E),     // < 3% Brand Accent
            accent_surface: Color32::from_rgba_unmultiplied(0xC4, 0x5D, 0x3E, 0x18),
            on_accent: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            tag_red: Color32::from_rgb(0xC4, 0x5D, 0x3E),
            tag_red_bg: Color32::from_rgba_unmultiplied(0xC4, 0x5D, 0x3E, 0x16),
            success: Color32::from_rgb(0x2D, 0x86, 0x59),
            warning: Color32::from_rgb(0xB8, 0x76, 0x14),
            danger: Color32::from_rgb(0xC4, 0x40, 0x37),
            info: Color32::from_rgb(0x32, 0x6B, 0xBE),
            diff_add_bg: Color32::from_rgba_premultiplied(0x38, 0x8E, 0x3C, 0x22),
            diff_add_fg: Color32::from_rgb(0x2E, 0x7D, 0x32),
            diff_del_bg: Color32::from_rgba_premultiplied(0xD3, 0x2F, 0x2F, 0x22),
            diff_del_fg: Color32::from_rgb(0xC6, 0x28, 0x28),
        }
    }

    pub fn warm_dusk() -> Self {
        let mut t = Self::warm_dark();
        t.id = CanhId::WarmDusk;
        t.name = "Hoàng Hôn (Elegans Warm Dusk)";
        t.bg_ink = Color32::from_rgb(0x18, 0x16, 0x1A);
        t.bg_field = Color32::from_rgb(0x20, 0x1D, 0x22);
        t.panel_bg = Color32::from_rgb(0x28, 0x24, 0x2B);
        t.panel_hover = Color32::from_rgb(0x31, 0x2C, 0x35);
        t.surface_1 = Color32::from_rgb(0x27, 0x23, 0x2A);
        t.surface_2 = Color32::from_rgb(0x32, 0x2D, 0x36);
        t.surface_3 = Color32::from_rgb(0x3B, 0x35, 0x40);
        t.border = Color32::from_rgb(0x36, 0x31, 0x3B);
        t.border_strong = Color32::from_rgb(0x4A, 0x43, 0x52);
        t
    }

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
