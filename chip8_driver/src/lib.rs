use std::time::{Duration, Instant};

use chip8_core::Chip8;

const TIMER_SPEED_HZ: u64 = 60;

pub struct Driver {
    core: Chip8,

    cpu_speed_hz: u64,
    cpu_cycle_duration: Duration,
    last_cpu_tick: Instant,

    timer_cycle_duration: Duration,
    last_timer_tick: Instant,
}

impl Driver {
    pub fn new() -> Result<Self, chip8_core::Chip8Error> {
        let mut driver = Self {
            core: Chip8::new()?,
            cpu_speed_hz: 500, // a resonable speed, 500HZ
            cpu_cycle_duration: Duration::from_secs(0),
            last_cpu_tick: Instant::now(),
            timer_cycle_duration: Duration::from_secs_f64(1.0 / TIMER_SPEED_HZ as f64),
            last_timer_tick: Instant::now(),
        };
        driver.set_cpu_speed(driver.cpu_speed_hz);
        Ok(driver)
    }

    pub fn reset(&mut self) -> Result<(), chip8_core::Chip8Error> {
        self.core.reset()
    }

    pub fn set_cpu_speed(&mut self, hz: u64) {
        self.cpu_speed_hz = hz;
        if hz > 0 {
            self.cpu_cycle_duration = Duration::from_secs_f64(1.0 / hz as f64);
        } else {
            // If the speed is 0, set it to a very long time, effectively pausing the CPU
            self.cpu_cycle_duration = Duration::from_secs(u64::MAX);
        }
    }

    pub fn tick(&mut self) -> Result<(), chip8_core::Chip8Error> {
        let now = Instant::now();

        // --- CPU Tick ---
        // Check if enough time has passed since the last CPU tick
        if now.duration_since(self.last_cpu_tick) >= self.cpu_cycle_duration {
            self.core.run()?;
            self.last_cpu_tick = now;
        }

        // --- Timer Tick ---
        // Check if enough time has passed since the last timer tick
        if now.duration_since(self.last_timer_tick) >= self.timer_cycle_duration {
            self.core.tick_timers(); // Update timers
            self.last_timer_tick = now;
        }

        Ok(())
    }

    // Input
    pub fn key_press(&mut self, key_index: u8) {
        self.core.key_press(key_index);
    }

    pub fn key_release(&mut self, key_index: u8) {
        self.core.key_release(key_index);
    }

    // Output
    pub fn framebuffer(&self) -> &[u8] {
        self.core.framebuffer()
    }

    pub fn is_display_updated(&self) -> bool {
        self.core.is_display_updated()
    }

    pub fn clear_display_updated_flag(&mut self) {
        self.core.clear_display_updated_flag();
    }

    pub fn should_beep(&self) -> bool {
        self.core.should_beep()
    }

    // ROM Loading
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), chip8_core::Chip8Error> {
        self.core.load_rom(rom)
    }
}

pub fn pixels_width() -> usize {
    chip8_core::framebuffer_width()
}

pub fn pixels_height() -> usize {
    chip8_core::framebuffer_height()
}
