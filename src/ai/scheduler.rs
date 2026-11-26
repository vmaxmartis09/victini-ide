use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExecutionMode {
    Auto,       // AI executes end-to-end autonomously
    Bypass,     // Skip safety prompts & confirm gates, max speed
    Gatekeeper, // Strict manual 1-click approval for every change
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActiveModal {
    VibeStream,   // Center focus: chat, diffs, agent timeline
    Verification, // Focus: live Emulator/Browser verification
}

pub struct MemoryReaper {
    pub last_interaction: Instant,
    pub is_sleeping: bool,
    pub idle_threshold_secs: u64,
}

impl MemoryReaper {
    pub fn new(idle_secs: u64) -> Self {
        Self {
            last_interaction: Instant::now(),
            is_sleeping: false,
            idle_threshold_secs: idle_secs,
        }
    }

    pub fn mark_activity(&mut self) {
        self.last_interaction = Instant::now();
        self.is_sleeping = false;
    }

    pub fn check_idle_and_reap(&mut self) -> bool {
        if !self.is_sleeping && self.last_interaction.elapsed().as_secs() >= self.idle_threshold_secs {
            self.is_sleeping = true;
            true // Trigger memory cleanup and lower frame rate
        } else {
            false
        }
    }
}
