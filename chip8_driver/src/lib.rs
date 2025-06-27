use std::time::{Duration, Instant};

use chip8_core::Chip8;

const TIMER_SPEED_HZ: u64 = 60;

#[derive(thiserror::Error, Debug)]
pub enum DriverError {
    #[error(transparent)]
    CoreError(#[from] chip8_core::Chip8Error),
}

pub struct Driver {
    core: Chip8,

    cpu_speed_hz: u64,
    cpu_cycle_duration: Duration,
    last_cpu_tick: Instant,

    timer_cycle_duration: Duration,
    last_timer_tick: Instant,
}

impl Driver {
    pub fn new(cpu_speed_hz: u64) -> Result<Self, DriverError> {
        let mut driver = Self {
            core: Chip8::new()?,
            cpu_speed_hz,
            cpu_cycle_duration: Duration::from_secs(0),
            last_cpu_tick: Instant::now(),
            timer_cycle_duration: Duration::from_secs_f64(1.0 / TIMER_SPEED_HZ as f64),
            last_timer_tick: Instant::now(),
        };
        driver.set_cpu_speed(driver.cpu_speed_hz);
        Ok(driver)
    }

    pub fn reset(&mut self) -> Result<(), DriverError> {
        self.core.reset()?;
        Ok(())
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

    pub fn tick(&mut self) -> Result<(), DriverError> {
        let now = Instant::now();
        let cpu_duration = now.duration_since(self.last_cpu_tick);
        let timer_duration = now.duration_since(self.last_timer_tick);

        // --- CPU Tick ---
        // Check if enough time has passed since the last CPU tick
        if cpu_duration >= self.cpu_cycle_duration {
            let cycles = cpu_duration.as_nanos() / self.cpu_cycle_duration.as_nanos();
            for _ in 0..cycles.max(1) {
                self.core.run()?;
            }
            self.last_cpu_tick = now;
        }

        // --- Timer Tick ---
        // Check if enough time has passed since the last timer tick
        if timer_duration >= self.timer_cycle_duration {
            let cycles = timer_duration.as_nanos() / self.timer_cycle_duration.as_nanos();
            for _ in 0..cycles.max(1) {
                self.core.tick_timers(); // Update timers
            }
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
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), DriverError> {
        self.core.load_rom(rom)?;
        Ok(())
    }
}

pub fn pixels_width() -> usize {
    chip8_core::framebuffer_width()
}

pub fn pixels_height() -> usize {
    chip8_core::framebuffer_height()
}
