use crate::emulator::adb_simctl::AdbController;
use anyhow::Result;
use egui::{ColorImage, Pos2};
use image::ImageReader;
use std::io::Cursor;

pub struct TouchRipple {
    pub norm_pos: Pos2,
    pub device_x: u32,
    pub device_y: u32,
    pub created_at: f32,
}

pub struct EmulatorState {
    pub active_preset_idx: usize,
    pub is_adb_mode: bool,
    pub adb_connected: bool,
    pub current_frame: Option<ColorImage>,
    pub frame_counter: u64,
    pub status_text: String,
    pub preview_url: String,
    pub last_touch: Option<TouchRipple>,
    pub auto_refresh_screencast: bool,
}

impl Default for EmulatorState {
    fn default() -> Self {
        let adb_ok = AdbController::is_device_connected();
        Self {
            active_preset_idx: 0,
            is_adb_mode: adb_ok,
            adb_connected: adb_ok,
            current_frame: None,
            frame_counter: 0,
            status_text: if adb_ok {
                "ADB Device Online".to_string()
            } else {
                "CDP / Viewport Simulator Ready".to_string()
            },
            preview_url: "http://localhost:3000".to_string(),
            last_touch: None,
            auto_refresh_screencast: false,
        }
    }
}

impl EmulatorState {
    pub fn refresh_adb_screenshot(&mut self) {
        self.adb_connected = AdbController::is_device_connected();
        self.is_adb_mode = self.adb_connected;
        if self.adb_connected {
            if let Ok(bytes) = AdbController::capture_screenshot() {
                if let Ok(img) = decode_png_to_color_image(&bytes) {
                    self.current_frame = Some(img);
                    self.frame_counter += 1;
                    self.status_text = format!("ADB Frame #{} Captured", self.frame_counter);
                    return;
                }
            }
        }

        // Fallback: Try capturing live screenshot from isolated CDP Browser on port 9222
        let cdp_bytes = std::thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .ok()?;
            let client = crate::browser_gitlab::cdp_client::CdpClient::new(9222);
            rt.block_on(client.capture_first_tab_screenshot()).ok()
        })
        .join()
        .ok()
        .flatten();

        if let Some(bytes) = cdp_bytes {
            if let Ok(img) = decode_png_to_color_image(&bytes) {
                self.current_frame = Some(img);
                self.frame_counter += 1;
                self.status_text = format!("CDP Frame #{} Captured", self.frame_counter);
                return;
            }
        }

        self.status_text = "Simulated Interactive Viewport Active".to_string();
    }

    pub fn clear_memory_cache(&mut self) {
        self.current_frame = None;
        self.auto_refresh_screencast = false;
    }
}

pub fn decode_png_to_color_image(bytes: &[u8]) -> Result<ColorImage> {
    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let dyn_img = reader.decode()?;
    let rgba = dyn_img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    Ok(ColorImage::from_rgba_unmultiplied(size, &pixels))
}
