//! # CHIP-8 Core Library
//!
//! This library provides a pure CHIP-8 virtual machine implementation without any external dependencies
//! for timing, graphics, or audio. It focuses solely on CPU instruction execution and state management.
//!
//! ## Key Features
//!
//! - Complete CHIP-8 instruction set implementation
//! - 4KB memory with font set pre-loaded
//! - 16 general-purpose registers (V0-VF)
//! - 64x32 monochrome display with XOR sprite drawing
//! - 16-key hexadecimal keypad support
//! - Stack-based subroutine calls (16 levels deep)
//! - Delay and sound timers
//!
//! ## Timer Management
//!
//! CHIP-8 has two 8-bit countdown timers that operate at 60Hz:
//!
//! - **Delay Timer (DT)**: Used for timing delays and synchronization
//! - **Sound Timer (ST)**: Controls beep sound duration
//!
//! **Important**: This library does NOT handle timing automatically. You must call
//! [`Chip8::tick_timers()`] at exactly 60Hz for proper timer behavior.
//!
//! ## Usage Example
//!
//! ```rust
//! use chip8_core::Chip8;
//! use std::time::{Duration, Instant};
//!
//! // Initialize the CHIP-8 system
//! let mut chip8 = Chip8::new().unwrap();
//!
//! // Load a ROM program
//! let rom_data = vec![0xA2, 0x2A, 0x60, 0x0C, 0x61, 0x08]; // Example ROM
//! chip8.load_rom(&rom_data).unwrap();
//!
//! // Main emulation loop
//! let mut last_timer_update = Instant::now();
//! let timer_interval = Duration::from_nanos(16_666_667); // 60Hz
//!
//! loop {
//!     // Execute one instruction
//!     if let Err(e) = chip8.run() {
//!         eprintln!("Execution error: {}", e);
//!         break;
//!     }
//!
//!     // Update timers at 60Hz
//!     if last_timer_update.elapsed() >= timer_interval {
//!         chip8.tick_timers();
//!         last_timer_update = Instant::now();
//!     }
//!
//!     // Handle display updates
//!     if chip8.is_display_updated() {
//!         // Render the framebuffer to screen
//!         let pixels = chip8.framebuffer();
//!         // ... render pixels to display ...
//!         chip8.clear_display_updated_flag();
//!     }
//!
//!     // Handle audio
//!     if chip8.should_beep() {
//!         // Play beep sound
//!     } else {
//!         // Stop beep sound
//!     }
//!
//!     // Handle input
//!     // chip8.key_press(key_index);   // When key is pressed
//!     // chip8.key_release(key_index); // When key is released
//! }
//! ```
mod consts;
mod executor;
mod instruction;
mod memory;

use consts::*;
use instruction::Instruction;

use crate::memory::{Memory, MemoryError};

/// Represents the CHIP-8 virtual machine.
///
/// This struct holds the entire state of a CHIP-8 system, including memory, registers,
/// timers, and I/O devices like the screen buffer and keyboard state.
pub struct Chip8 {
    /// Memory of the Chip8
    memory: Memory,

    /// Registers of the Chip8
    ///
    /// 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    /// The VF register should not be used by any program, as it is used as a flag by some instructions.
    registers: [u8; 16],

    /// Index Register of the Chip8
    ///
    /// a 16-bit register called I. This register is generally used to store memory addresses, so only the lowest (rightmost) 12 bits are usually used.
    i: u16,

    /// Program Counter of the Chip8
    pc: u16,

    /// Stack Pointer of the Chip8
    sp: u8,

    /// Stack of the Chip8
    stack: [u16; 16],

    /// Delay Timer of the Chip8
    dt: u8,

    /// Sound Timer of the Chip8
    st: u8,

    /// Frame Buffer of the Chip8
    framebuffer: [u8; 64 * 32],

    /// Keyboard State of the Chip8
    keyboard: [u8; 16],

    /// Flag to indicate that the display has been updated
    display_updated: bool,
}

/// Defines the possible errors that can occur during CHIP-8 emulation.
#[derive(Debug, thiserror::Error)]
pub enum Chip8Error {
    /// Failed to load the font set into memory. This is an internal error that should not occur in normal operation.
    #[error("Font-set is out of bounds")]
    MemoryError(#[from] MemoryError),
    /// The program counter points to an invalid memory address, preventing instruction fetching.
    #[error("PC points to an invalid memory: {0}")]
    PCError(u16),
    /// An unknown or unimplemented opcode was encountered.
    #[error("Invalid opcode")]
    InvalidOpCode(String),
    /// The stack pointer is out of its valid bounds (0-15).
    #[error("SP {0} is out of bounds")]
    SPError(u8),
    /// A stack push or pop operation failed due to overflow or underflow.
    #[error("SP {0} is overflow or underflow")]
    SPOverflow(u8),
    /// Occurs when an operation attempts to access a pixel outside the framebuffer's boundaries.
    #[error("Frame buffer is out of bounds: {0}")]
    FrameBufferOverflow(usize),
    /// The index register (I) points to an invalid memory address.
    #[error("Index register points to an invalid memory: {0}")]
    IndexError(u16),
    /// An instruction referenced an invalid general-purpose register (valid range: V0-VF).
    #[error("Invalid register: V{0}")]
    InvalidRegister(usize),
    /// An instruction referenced an invalid keyboard key (valid range: 0-15).
    #[error("Invalid keyboard key index: {0}")]
    InvalidKey(u8),
}

impl Chip8 {
    /// Creates and initializes a new CHIP-8 virtual machine.
    ///
    /// This function sets up the initial state of the emulator:
    /// - It clears memory, registers, and the stack.
    /// - It sets the program counter (`pc`) to `0x200`, the standard starting address for CHIP-8 programs.
    /// - It loads the built-in font set into memory starting at `0x50`.
    ///
    /// # Returns
    ///
    /// * `Ok(Chip8)` with a new, ready-to-use `Chip8` instance.
    /// * `Err(Chip8Error::LoadFontSetError)` if the font set cannot be loaded, which is an unlikely internal error.
    pub fn new() -> Result<Self, Chip8Error> {
        Ok(Self {
            memory: Memory::try_new()?,
            registers: [0; 16],
            pc: 0x200,
            sp: 0,
            i: 0,
            stack: [0; 16],
            dt: 0,
            st: 0,
            framebuffer: [0; 64 * 32],
            keyboard: [0; 16],
            display_updated: false,
        })
    }

    /// Resets the CHIP-8 virtual machine to its initial state.
    ///
    /// This is equivalent to turning the machine off and on again. It clears all registers,
    /// memory (except for the font set), the stack, and I/O devices. The program counter
    /// is reset to `0x200`. The font set is reloaded into its standard memory location.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful reset.
    /// * `Err(Chip8Error::LoadFontSetError)` if reloading the font fails, which is an unlikely internal error.
    pub fn reset(&mut self) -> Result<(), Chip8Error> {
        self.memory = Memory::try_new()?;
        self.registers = [0; 16];
        self.pc = 0x200;
        self.sp = 0;
        self.i = 0;
        self.stack = [0; 16];
        self.dt = 0;
        self.st = 0;
        self.framebuffer = [0; 64 * 32];
        self.keyboard = [0; 16];
        self.display_updated = false;

        Ok(())
    }

    /// Loads a CHIP-8 program (ROM) into memory.
    ///
    /// The provided ROM data is copied into the CHIP-8 memory, starting at the
    /// standard program address `0x200`.
    ///
    /// # Arguments
    ///
    /// * `rom`: A byte slice representing the program's binary data.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the ROM was successfully loaded.
    /// * `Err(Chip8Error::LoadRomError)` if the ROM is too large to fit in the memory
    ///   from the starting address `0x200` to the end of memory.
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), Chip8Error> {
        self.memory.write_at(rom, ROM_START_ADDRESS)?;
        Ok(())
    }

    /// Returns a read-only slice of the framebuffer.
    ///
    /// The framebuffer represents the CHIP-8's 64x32 monochrome display.
    /// Each byte in the slice corresponds to a pixel, with `1` representing
    /// a pixel that is on and `0` for a pixel that is off. The data is
    /// stored in row-major order.
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Checks if the display has been updated since the last check.
    ///
    /// This flag is set to `true` by instructions that modify the framebuffer,
    /// such as `00E0` (clear screen) and `DXYN` (draw sprite). The UI layer
    /// should check this flag each frame to determine if it needs to redraw
    /// the screen.
    pub fn is_display_updated(&self) -> bool {
        self.display_updated
    }

    /// Clears the display updated flag.
    ///
    /// This should be called by the UI layer after it has redrawn the screen
    /// based on the `is_display_updated` flag.
    pub fn clear_display_updated_flag(&mut self) {
        self.display_updated = false;
    }

    /// Simulates a key press on the CHIP-8 keypad.
    ///
    /// # Arguments
    ///
    /// * `key_index`: The index of the key to press (0-15). Any value outside
    ///   this range will be ignored.
    pub fn key_press(&mut self, key_index: u8) {
        if let Some(key) = self.keyboard.get_mut(key_index as usize) {
            *key = 1;
        }
    }

    /// Simulates a key release on the CHIP-8 keypad.
    ///
    /// # Arguments
    ///
    /// * `key_index`: The index of the key to release (0-15). Any value outside
    ///   this range will be ignored.
    pub fn key_release(&mut self, key_index: u8) {
        if let Some(key) = self.keyboard.get_mut(key_index as usize) {
            *key = 0;
        }
    }

    /// Decrements both delay and sound timers by 1 if they are greater than 0.
    ///
    /// This function should be called at exactly 60Hz frequency to maintain proper
    /// timing behavior that CHIP-8 programs expect. The CHIP-8 specification
    /// defines that both timers decrement at this rate until they reach zero.
    ///
    /// # Timer Behavior
    ///
    /// - **Delay Timer (DT)**: Used by programs for timing delays and synchronization
    /// - **Sound Timer (ST)**: Controls the duration of the beep sound
    ///
    /// # Usage
    ///
    /// This function should typically be called in your main emulation loop at
    /// a consistent 60Hz interval (approximately every 16.67ms).
    ///
    /// # Note
    ///
    /// This function does not handle timing automatically. It is the caller's
    /// responsibility to ensure it is called at the correct frequency for
    /// accurate CHIP-8 timing behavior.
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    /// Returns true if the sound timer is greater than 0, indicating a beep should be played.
    ///
    /// The sound timer controls when the CHIP-8 system should produce its characteristic
    /// beep sound. When the timer is non-zero, a continuous tone should be played.
    /// When it reaches zero, the sound should stop.
    ///
    /// # Returns
    ///
    /// * `true` if sound should be playing (sound timer > 0)
    /// * `false` if sound should be silent (sound timer = 0)
    pub fn should_beep(&self) -> bool {
        self.st > 0
    }

    /// Returns the current value of the delay timer.
    ///
    /// The delay timer is an 8-bit countdown timer that decrements at 60Hz until
    /// it reaches zero. Programs use it for timing delays, animations, and
    /// synchronization. It can be set by the `FX15` instruction and read by
    /// the `FX07` instruction.
    ///
    /// # Returns
    ///
    /// The current delay timer value (0-255)
    pub fn delay_timer(&self) -> u8 {
        self.dt
    }

    /// Returns the current value of the sound timer.
    ///
    /// The sound timer is an 8-bit countdown timer that decrements at 60Hz until
    /// it reaches zero. While non-zero, the CHIP-8 system should produce a beep
    /// sound. It can be set by the `FX18` instruction.
    ///
    /// # Returns
    ///
    /// The current sound timer value (0-255)
    pub fn sound_timer(&self) -> u8 {
        self.st
    }

    /// Returns true if the delay timer has reached zero (finished).
    ///
    /// This is a convenience method that's equivalent to `delay_timer() == 0`.
    /// It's commonly used to check if a timed delay has completed.
    ///
    /// # Returns
    ///
    /// * `true` if the delay timer is 0 (delay finished)
    /// * `false` if the delay timer is still counting down
    pub fn delay_timer_finished(&self) -> bool {
        self.dt == 0
    }

    /// Executes a single CHIP-8 instruction cycle.
    ///
    /// This involves fetching the opcode from memory at the program counter,
    /// decoding it, and executing the corresponding operation. The program
    /// counter is advanced accordingly.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful execution of the instruction.
    /// * `Err(Chip8Error)` if an error occurs, such as fetching from an invalid
    ///   memory address or executing an invalid opcode.
    pub fn run(&mut self) -> Result<(), Chip8Error> {
        let instruction = self.fetch()?;
        self.execute_instruction(&instruction)
    }

    /// Fetches the next instruction from memory at the current program counter (`pc`),
    /// decodes it, and advances the `pc` by two bytes.
    ///
    /// # Returns
    ///
    /// * `Ok(Instructions)` containing the decoded instruction.
    /// * `Err(Chip8Error::PCError)` if the `pc` is at or near the end of memory,
    ///   making it impossible to fetch a full 2-byte instruction.
    fn fetch(&mut self) -> Result<Instruction, Chip8Error> {
        let instruction = self
            .memory
            .read_word(self.pc as usize)
            .ok_or(Chip8Error::PCError(self.pc))?;

        self.pc = self.pc.checked_add(2).ok_or(Chip8Error::PCError(self.pc))?;
        Ok(Instruction::new(instruction))
    }

    /// Pushes the program counter (`pc`) onto the stack.
    ///
    /// Increments the stack pointer (`sp`) after pushing.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the push was successful.
    /// * `Err(Chip8Error::SPOverflow)` if the stack pointer would overflow.
    /// * `Err(Chip8Error::SPError)` if the stack pointer is out of bounds.
    fn push_stack(&mut self) -> Result<(), Chip8Error> {
        if let Some(memory) = self.stack.get_mut(self.sp as usize) {
            *memory = self.pc;
            self.sp = self
                .sp
                .checked_add(1)
                .ok_or(Chip8Error::SPOverflow(self.sp))?;
        } else {
            return Err(Chip8Error::SPError(self.sp));
        }
        Ok(())
    }

    /// Pops a value from the stack into the program counter (`pc`).
    ///
    /// Decrements the stack pointer (`sp`) before popping.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the pop was successful.
    /// * `Err(Chip8Error::SPOverflow)` if the stack pointer would underflow.
    /// * `Err(Chip8Error::SPError)` if the stack pointer is out of bounds.
    fn pop_stack(&mut self) -> Result<(), Chip8Error> {
        self.sp = self
            .sp
            .checked_sub(1)
            .ok_or(Chip8Error::SPOverflow(self.sp))?;
        if let Some(&memory) = self.stack.get(self.sp as usize) {
            self.pc = memory;
            Ok(())
        } else {
            Err(Chip8Error::SPError(self.sp))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
        let pc = chip8.pc as usize;
        chip8
            .memory
            .write_byte(pc, (instruction >> 8) as u8)
            .expect("Failed to write instruction");
        chip8
            .memory
            .write_byte(pc + 1, (instruction & 0xFF) as u8)
            .expect("Failed to write instruction");
        chip8.run()
    }

    #[test]
    fn test_new() {
        let chip8 = Chip8::new().unwrap();

        // Verify initial state
        assert_eq!(chip8.pc, 0x200);
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.i, 0);
        assert_eq!(chip8.dt, 0);
        assert_eq!(chip8.st, 0);
    }

    #[test]
    fn test_reset() {
        let mut chip8 = Chip8::new().unwrap();
        // Set some state to non-default values
        chip8
            .memory
            .write_byte(0x300, 0xFF)
            .expect("Failed to write memory");
        chip8.registers[0] = 0xAA;
        chip8.pc = 0x300;
        chip8.sp = 5;
        chip8.i = 0x123;
        chip8.stack[0] = 0x456;
        chip8.dt = 10;
        chip8.st = 20;
        chip8.framebuffer[0] = 1;
        chip8.keyboard[0] = 1;

        chip8.reset().unwrap();

        // Verify all fields were reset
        assert_eq!(chip8.registers, [0; 16]);
        assert_eq!(chip8.pc, 0x200);
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.i, 0);
        assert_eq!(chip8.stack, [0; 16]);
        assert_eq!(chip8.dt, 0);
        assert_eq!(chip8.st, 0);
        assert_eq!(chip8.framebuffer, [0; 64 * 32]);
        assert_eq!(chip8.keyboard, [0; 16]);
    }

    #[test]
    fn test_timer_management() {
        let mut chip8 = Chip8::new().unwrap();

        // Initial state - both timers should be 0
        assert_eq!(chip8.delay_timer(), 0);
        assert_eq!(chip8.sound_timer(), 0);
        assert!(!chip8.should_beep());
        assert!(chip8.delay_timer_finished());

        // Manually set timers to test tick functionality
        chip8.dt = 10;
        chip8.st = 5;

        assert_eq!(chip8.delay_timer(), 10);
        assert_eq!(chip8.sound_timer(), 5);
        assert!(chip8.should_beep());
        assert!(!chip8.delay_timer_finished());

        // Test single tick
        chip8.tick_timers();
        assert_eq!(chip8.delay_timer(), 9);
        assert_eq!(chip8.sound_timer(), 4);
        assert!(chip8.should_beep());

        // Test multiple ticks until sound timer reaches 0
        for expected_dt in (5..9).rev() {
            chip8.tick_timers();
            assert_eq!(chip8.delay_timer(), expected_dt);
        }

        // At this point: dt = 5, st = 0
        assert_eq!(chip8.delay_timer(), 5);
        assert_eq!(chip8.sound_timer(), 0);
        assert!(!chip8.should_beep());
        assert!(!chip8.delay_timer_finished());

        // Tick until delay timer also reaches 0
        for _ in 0..5 {
            chip8.tick_timers();
        }

        assert_eq!(chip8.delay_timer(), 0);
        assert_eq!(chip8.sound_timer(), 0);
        assert!(!chip8.should_beep());
        assert!(chip8.delay_timer_finished());

        // Ticking when timers are already 0 should not cause underflow
        chip8.tick_timers();
        assert_eq!(chip8.delay_timer(), 0);
        assert_eq!(chip8.sound_timer(), 0);
    }

    #[test]
    fn test_timer_edge_cases() {
        let mut chip8 = Chip8::new().unwrap();

        // Test timer value 1 (should go to 0 after one tick)
        chip8.dt = 1;
        chip8.st = 1;

        assert!(!chip8.delay_timer_finished());
        assert!(chip8.should_beep());

        chip8.tick_timers();

        assert!(chip8.delay_timer_finished());
        assert!(!chip8.should_beep());

        // Test maximum timer value (255)
        chip8.dt = 255;
        chip8.st = 255;

        chip8.tick_timers();

        assert_eq!(chip8.delay_timer(), 254);
        assert_eq!(chip8.sound_timer(), 254);

        // Test asymmetric timer values
        chip8.dt = 100;
        chip8.st = 10;

        // Tick 10 times
        for i in 1..=10 {
            chip8.tick_timers();
            assert_eq!(chip8.delay_timer(), 100 - i);
            if i < 10 {
                assert_eq!(chip8.sound_timer(), 10 - i);
                assert!(chip8.should_beep());
            } else {
                assert_eq!(chip8.sound_timer(), 0);
                assert!(!chip8.should_beep());
            }
        }
    }

    #[test]
    fn test_timer_frequency_simulation() {
        let mut chip8 = Chip8::new().unwrap();

        // Simulate 1 second of operation at 60Hz
        chip8.dt = 60; // 1 second delay
        chip8.st = 30; // 0.5 second beep

        // Simulate 60 timer ticks (1 second at 60Hz)
        for tick in 1..=60 {
            chip8.tick_timers();

            let expected_dt = if tick <= 60 { 60 - tick } else { 0 };
            let expected_st = if tick <= 30 { 30 - tick } else { 0 };

            assert_eq!(chip8.delay_timer(), expected_dt);
            assert_eq!(chip8.sound_timer(), expected_st);

            // Sound should stop after 30 ticks (0.5 seconds)
            if tick < 30 {
                assert!(
                    chip8.should_beep(),
                    "Sound should be playing at tick {}",
                    tick
                );
            } else {
                assert!(
                    !chip8.should_beep(),
                    "Sound should be silent at tick {}",
                    tick
                );
            }

            // Delay should finish after 60 ticks (1 second)
            if tick < 60 {
                assert!(
                    !chip8.delay_timer_finished(),
                    "Delay should not be finished at tick {}",
                    tick
                );
            } else {
                assert!(
                    chip8.delay_timer_finished(),
                    "Delay should be finished at tick {}",
                    tick
                );
            }
        }
    }

    #[test]
    fn test_timer_integration_with_instructions() {
        let mut chip8 = Chip8::new().unwrap();

        // Test with FX15 instruction (set delay timer to Vx)
        chip8.registers[5] = 42;
        run_instruction(&mut chip8, 0xF515).unwrap(); // FX15: Set DT to V5
        assert_eq!(chip8.delay_timer(), 42);

        // Test with FX18 instruction (set sound timer to Vx)
        chip8.registers[3] = 25;
        run_instruction(&mut chip8, 0xF318).unwrap(); // FX18: Set ST to V3
        assert_eq!(chip8.sound_timer(), 25);
        assert!(chip8.should_beep());

        // Test with FX07 instruction (load delay timer into Vx)
        chip8.registers[7] = 0; // Clear register first
        run_instruction(&mut chip8, 0xF707).unwrap(); // FX07: Load DT into V7
        assert_eq!(chip8.registers[7], 42);

        // Simulate some timer ticks and verify behavior
        for _ in 0..10 {
            chip8.tick_timers();
        }

        assert_eq!(chip8.delay_timer(), 32);
        assert_eq!(chip8.sound_timer(), 15);
        assert!(chip8.should_beep());

        // Read the updated delay timer value
        run_instruction(&mut chip8, 0xF207).unwrap(); // FX07: Load DT into V2
        assert_eq!(chip8.registers[2], 32);
    }

    #[test]
    fn test_load_rom() {
        let mut chip8 = Chip8::new().unwrap();
        let rom_data = vec![0x1, 0x2, 0x3, 0x4];
        chip8.load_rom(&rom_data).unwrap();

        let memory_slice = chip8
            .memory
            .get(ROM_START_ADDRESS..ROM_START_ADDRESS + rom_data.len())
            .expect("Failed to read memory at ROM address");
        assert_eq!(memory_slice, rom_data.as_slice());
    }

    #[test]
    fn test_load_rom_out_of_bounds() {
        let mut chip8 = Chip8::new().unwrap();
        let rom_size = chip8.memory.size() - ROM_START_ADDRESS + 1;
        let rom_data = vec![0u8; rom_size];

        assert!(matches!(
            chip8.load_rom(&rom_data),
            Err(Chip8Error::MemoryError(_))
        ));
    }

    #[test]
    fn test_fetch_success() {
        let mut chip8 = Chip8::new().unwrap();
        // Load an instruction 0x1234 at the start of ROM space
        chip8
            .memory
            .write_byte(ROM_START_ADDRESS, 0x12)
            .expect("Failed to write byte");
        chip8
            .memory
            .write_byte(ROM_START_ADDRESS + 1, 0x34)
            .expect("Failed to write byte");

        let initial_pc = chip8.pc;
        let instructions = chip8.fetch().unwrap();

        assert_eq!(instructions.instruction(), 0x1);
        assert_eq!(instructions.x(), 0x2);
        assert_eq!(instructions.y(), 0x3);
        assert_eq!(instructions.n(), 0x4);
        assert_eq!(instructions.nn(), 0x34);
        assert_eq!(instructions.nnn(), 0x234);

        // PC should advance by 2 bytes
        assert_eq!(chip8.pc, initial_pc + 2);
    }

    #[test]
    fn test_fetch_out_of_bounds() {
        let mut chip8 = Chip8::new().unwrap();
        // Set PC to the last byte of memory, where a 2-byte instruction cannot be read
        chip8.pc = (chip8.memory.size() - 1) as u16;
        let initial_pc = chip8.pc;

        let result = chip8.fetch();
        assert!(matches!(result, Err(Chip8Error::PCError(_))));

        // PC should not advance on failure
        assert_eq!(chip8.pc, initial_pc);
    }
}
