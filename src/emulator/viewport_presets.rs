#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportPreset {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
    pub is_mobile: bool,
}

pub const ELEGANS_VIEWPORT_PRESETS: &[ViewportPreset] = &[
    ViewportPreset {
        name: "iPhone 15 Pro (390×844)",
        width: 390,
        height: 844,
        device_scale_factor: 3.0,
        is_mobile: true,
    },
    ViewportPreset {
        name: "Android Compact (360×800)",
        width: 360,
        height: 800,
        device_scale_factor: 2.0,
        is_mobile: true,
    },
    ViewportPreset {
        name: "iPad Air (820×1180)",
        width: 820,
        height: 1180,
        device_scale_factor: 2.0,
        is_mobile: true,
    },
    ViewportPreset {
        name: "Laptop 1280 (1280×800)",
        width: 1280,
        height: 800,
        device_scale_factor: 1.0,
        is_mobile: false,
    },
    ViewportPreset {
        name: "Stress Test 320 (320×640)",
        width: 320,
        height: 640,
        device_scale_factor: 1.0,
        is_mobile: true,
    },
];
