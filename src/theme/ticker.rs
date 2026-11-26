use std::collections::HashMap;
use std::time::Instant;

pub struct SharedTicker {
    start_time: Instant,
    last_tick: Instant,
    pub dt_ms: f32,
    pub elapsed_secs: f32,
    active_animations: usize,
    floats: HashMap<&'static str, f32>,
}

impl SharedTicker {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_tick: now,
            dt_ms: 16.667,
            elapsed_secs: 0.0,
            active_animations: 0,
            floats: HashMap::new(),
        }
    }

    pub fn tick(&mut self) -> f32 {
        let now = Instant::now();
        let dt = (now.duration_since(self.last_tick).as_secs_f64() * 1000.0) as f32;
        self.last_tick = now;
        self.dt_ms = dt.clamp(1.0, 100.0);
        self.elapsed_secs = now.duration_since(self.start_time).as_secs_f32();
        self.active_animations = 0;
        self.dt_ms
    }

    /// Frame-rate independent lerp factor from Portfolio ticker.ts
    /// factor = 1 - (1 - per_frame)^(dt / 16.667)
    pub fn damp(per_frame: f32, dt_ms: f32) -> f32 {
        1.0 - (1.0 - per_frame).powf(dt_ms / 16.667)
    }

    /// Smoothly animate a named float toward `target` and automatically request repaint while moving
    pub fn animate_float(&mut self, key: &'static str, target: f32, speed: f32) -> f32 {
        let current = *self.floats.entry(key).or_insert(target);
        let diff = (target - current).abs();
        if diff > 0.005 {
            self.active_animations += 1;
            let factor = Self::damp(speed, self.dt_ms);
            let next = current + (target - current) * factor;
            self.floats.insert(key, next);
            next
        } else {
            self.floats.insert(key, target);
            target
        }
    }

    /// Continuous 0.0..=1.0 sine wave for pulsing rings and breathing animations
    pub fn pulse(&self, freq_hz: f32) -> f32 {
        let angle = self.elapsed_secs * freq_hz * std::f32::consts::TAU;
        (angle.sin() * 0.5) + 0.5
    }

    /// Continuous rotation angle in radians for spinners and orbital rings
    pub fn spinner_angle(&self, speed_rad_per_sec: f32) -> f32 {
        (self.elapsed_secs * speed_rad_per_sec) % std::f32::consts::TAU
    }

    pub fn request_continuous(&mut self) {
        self.active_animations += 1;
    }

    pub fn should_repaint(&self) -> bool {
        self.active_animations > 0
    }
}
