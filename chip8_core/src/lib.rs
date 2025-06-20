mod consts;
mod executor;
mod instruction;

use consts::*;
use instruction::Instruction;

/// Represents the CHIP-8 virtual machine.
///
/// This struct holds the entire state of a CHIP-8 system, including memory, registers,
/// timers, and I/O devices like the screen buffer and keyboard state.
pub struct Chip8 {
    /// Memory of the Chip8
    memory: [u8; 4096],

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
    LoadFontSetError,
    /// The provided ROM is too large to fit into the CHIP-8 memory.
    #[error("ROM is too large to fit into the CHIP-8 memory")]
    LoadRomError,
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
        let mut chip8 = Self {
            memory: [0; 4096],
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
        };
        chip8.load_font_at(FONT_START_ADDRESS, &FONT_SET)?;
        Ok(chip8)
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
        self.memory = [0; 4096];
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

        self.load_font()?;
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
        self.load_slice_at(ROM_START_ADDRESS, rom)
            .map_err(|_| Chip8Error::LoadRomError)
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
        if let Some(instruction_bytes) = self.memory.get(self.pc as usize..self.pc as usize + 2) {
            self.pc = self.pc.wrapping_add(2);
            let instruction = (instruction_bytes[0] as u16) << 8 | instruction_bytes[1] as u16;
            Ok(Instruction::new(instruction))
        } else {
            Err(Chip8Error::PCError(self.pc))
        }
    }

    /// Loads the built-in font set into memory at the default address.
    fn load_font(&mut self) -> Result<(), Chip8Error> {
        self.load_font_at(FONT_START_ADDRESS, &FONT_SET)
    }

    /// Loads the built-in font set into memory at a specific address.
    fn load_font_at(&mut self, start_address: usize, font: &[u8]) -> Result<(), Chip8Error> {
        self.load_slice_at(start_address, font)
            .map_err(|_| Chip8Error::LoadFontSetError)
    }

    /// A helper function to copy a slice of data into memory at a specific address.
    /// Returns a generic error on failure to be mapped by the caller.
    fn load_slice_at(&mut self, start_address: usize, data: &[u8]) -> Result<(), ()> {
        if let Some(memory) = self
            .memory
            .get_mut(start_address..start_address + data.len())
        {
            memory.clone_from_slice(data);
            Ok(())
        } else {
            Err(())
        }
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
mod tests;
