use anyhow::Result;
use std::process::Command;

pub struct AdbController;

impl AdbController {
    pub fn is_device_connected() -> bool {
        if let Ok(output) = Command::new("adb").arg("devices").output() {
            let s = String::from_utf8_lossy(&output.stdout);
            s.lines().skip(1).any(|l| l.contains("device") && !l.contains("offline"))
        } else {
            false
        }
    }

    pub fn capture_screenshot() -> Result<Vec<u8>> {
        let output = Command::new("adb")
            .args(["exec-out", "screencap", "-p"])
            .output()?;
        Ok(output.stdout)
    }

    pub fn dispatch_tap(x: u32, y: u32) -> Result<()> {
        Command::new("adb")
            .args(["shell", "input", "tap", &x.to_string(), &y.to_string()])
            .spawn()?;
        Ok(())
    }

    pub fn dispatch_swipe(x1: u32, y1: u32, x2: u32, y2: u32, duration_ms: u32) -> Result<()> {
        Command::new("adb")
            .args([
                "shell", "input", "swipe",
                &x1.to_string(), &y1.to_string(),
                &x2.to_string(), &y2.to_string(),
                &duration_ms.to_string(),
            ])
            .spawn()?;
        Ok(())
    }
}
