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
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let chip8 = Chip8::new().unwrap();

        // Verify initial state
        assert_eq!(chip8.pc, 0x200);
        assert_eq!(chip8.sp, 0);
        assert_eq!(chip8.i, 0);
        assert_eq!(chip8.dt, 0);
        assert_eq!(chip8.st, 0);

        // Verify font was loaded
        let font_in_memory = &chip8.memory[FONT_START_ADDRESS..FONT_START_ADDRESS + FONT_SET.len()];
        assert_eq!(font_in_memory, FONT_SET);
    }

    #[test]
    fn test_reset() {
        let mut chip8 = Chip8::new().unwrap();
        // Set some state to non-default values
        chip8.memory[0x300] = 0xFF;
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

        // Verify memory is cleared except for the font
        let font_end = FONT_START_ADDRESS + FONT_SET.len();
        let font_in_memory = &chip8.memory[FONT_START_ADDRESS..font_end];
        assert_eq!(font_in_memory, FONT_SET);
        // Check a byte before the font
        assert_eq!(chip8.memory[FONT_START_ADDRESS - 1], 0);
        // Check a byte after the font
        assert_eq!(chip8.memory[font_end], 0);
        // Check the location that was previously dirty
        assert_eq!(chip8.memory[0x300], 0);
    }

    #[test]
    fn test_default() {
        let chip8 = Chip8::new().unwrap();
        assert_eq!(chip8.pc, 0x200);
        assert_eq!(chip8.memory[FONT_START_ADDRESS], FONT_SET[0]);
    }

    #[test]
    fn test_load_font_out_of_bounds() {
        let mut chip8 = Chip8::new().unwrap();
        // Test loading font at an address that would overflow memory
        let bad_addr = chip8.memory.len() - FONT_SET.len() + 1;
        assert!(
            chip8.load_font_at(bad_addr, &FONT_SET).is_err(),
            "Should fail when font would exceed memory bounds"
        );
    }

    #[test]
    fn test_reset_preserves_font() {
        let mut chip8 = Chip8::new().unwrap();
        // Modify some memory after font
        chip8.memory[FONT_START_ADDRESS + FONT_SET.len()] = 0xFF;

        chip8.reset().unwrap();

        // Verify font is still loaded
        let font_in_memory = &chip8.memory[FONT_START_ADDRESS..FONT_START_ADDRESS + FONT_SET.len()];
        assert_eq!(font_in_memory, FONT_SET);
        // Verify other memory was reset
        assert_eq!(chip8.memory[FONT_START_ADDRESS + FONT_SET.len()], 0);
    }

    #[test]
    fn test_load_rom() {
        let mut chip8 = Chip8::new().unwrap();
        let rom_data = vec![0x1, 0x2, 0x3, 0x4];
        chip8.load_rom(&rom_data).unwrap();

        let memory_slice = &chip8.memory[ROM_START_ADDRESS..ROM_START_ADDRESS + rom_data.len()];
        assert_eq!(memory_slice, rom_data.as_slice());
    }

    #[test]
    fn test_load_rom_out_of_bounds() {
        let mut chip8 = Chip8::new().unwrap();
        let rom_size = chip8.memory.len() - ROM_START_ADDRESS + 1;
        let rom_data = vec![0u8; rom_size];

        assert!(matches!(
            chip8.load_rom(&rom_data),
            Err(Chip8Error::LoadRomError)
        ));
    }

    #[test]
    fn test_instructions_decoding() {
        let instruction = 0xABCD;
        let decoded = Instruction::new(instruction);
        assert_eq!(decoded.instruction(), 0xA);
        assert_eq!(decoded.x(), 0xB);
        assert_eq!(decoded.y(), 0xC);
        assert_eq!(decoded.n(), 0xD);
        assert_eq!(decoded.nn(), 0xCD);
        assert_eq!(decoded.nnn(), 0xBCD);
    }

    #[test]
    fn test_fetch_success() {
        let mut chip8 = Chip8::new().unwrap();
        // Load an instruction 0x1234 at the start of ROM space
        chip8.memory[ROM_START_ADDRESS] = 0x12;
        chip8.memory[ROM_START_ADDRESS + 1] = 0x34;

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
        chip8.pc = (chip8.memory.len() - 1) as u16;
        let initial_pc = chip8.pc;

        let result = chip8.fetch();
        assert!(matches!(result, Err(Chip8Error::PCError(_))));

        // PC should not advance on failure
        assert_eq!(chip8.pc, initial_pc);
    }

    // Helper to run a single instruction
    fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
        let pc = chip8.pc as usize;
        chip8.memory[pc] = (instruction >> 8) as u8;
        chip8.memory[pc + 1] = (instruction & 0xFF) as u8;
        chip8.run()
    }

    #[test]
    fn test_op_1nnn_jp() {
        let mut chip8 = Chip8::new().unwrap();
        run_instruction(&mut chip8, 0x1ABC).unwrap();
        assert_eq!(chip8.pc, 0x0ABC);
    }

    #[test]
    fn test_op_2nnn_call_and_00ee_ret() {
        let mut chip8 = Chip8::new().unwrap();
        let initial_pc = chip8.pc;

        // CALL 0x300
        run_instruction(&mut chip8, 0x2300).unwrap();
        assert_eq!(chip8.pc, 0x300, "PC should jump to subroutine address");
        assert_eq!(chip8.sp, 1, "Stack pointer should increment");
        assert_eq!(
            chip8.stack[0],
            initial_pc + 2,
            "Return address should be on stack"
        );

        // Let's test the run command for RET
        let mut chip8 = Chip8::new().unwrap();
        chip8.pc = 0x300;
        chip8.sp = 1;
        chip8.stack[0] = 0x250;

        run_instruction(&mut chip8, 0x00EE).unwrap();
        assert_eq!(chip8.pc, 0x250);
        assert_eq!(chip8.sp, 0);
    }

    #[test]
    fn test_op_3xkk_se_vx_byte_skip() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[3] = 0x42;
        let initial_pc = chip8.pc;
        run_instruction(&mut chip8, 0x3342).unwrap();
        assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
    }

    #[test]
    fn test_op_3xkk_se_vx_byte_no_skip() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[3] = 0x42;
        let initial_pc = chip8.pc;
        run_instruction(&mut chip8, 0x3343).unwrap();
        assert_eq!(chip8.pc, initial_pc + 2, "PC should not skip");
    }

    #[test]
    fn test_op_6xkk_ld_vx_byte() {
        let mut chip8 = Chip8::new().unwrap();
        run_instruction(&mut chip8, 0x65AB).unwrap();
        assert_eq!(chip8.registers[5], 0xAB);
    }

    #[test]
    fn test_op_7xkk_add_vx_byte() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[5] = 10;
        run_instruction(&mut chip8, 0x7505).unwrap();
        assert_eq!(chip8.registers[5], 15);
    }

    #[test]
    fn test_op_8xy4_add_vx_vy_no_carry() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 10;
        chip8.registers[2] = 20;
        run_instruction(&mut chip8, 0x8124).unwrap();
        assert_eq!(chip8.registers[1], 30);
        assert_eq!(chip8.registers[0xF], 0, "VF should be 0 for no carry");
    }

    #[test]
    fn test_op_8xy4_add_vx_vy_with_carry() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0xFF;
        chip8.registers[2] = 0x01;
        run_instruction(&mut chip8, 0x8124).unwrap();
        assert_eq!(chip8.registers[1], 0);
        assert_eq!(chip8.registers[0xF], 1, "VF should be 1 for carry");
    }

    #[test]
    fn test_op_fx33_ld_b_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[0] = 123;
        chip8.i = 0x300;
        run_instruction(&mut chip8, 0xF033).unwrap();
        assert_eq!(chip8.memory[0x300], 1);
        assert_eq!(chip8.memory[0x301], 2);
        assert_eq!(chip8.memory[0x302], 3);
    }

    #[test]
    fn test_op_fx55_ld_i_vx() {
        let mut chip8 = Chip8::new().unwrap();
        for i in 0..=5 {
            chip8.registers[i] = i as u8;
        }
        chip8.i = 0x300;
        run_instruction(&mut chip8, 0xF555).unwrap();
        for i in 0..=5 {
            assert_eq!(chip8.memory[0x300 + i], i as u8);
        }
    }

    #[test]
    fn test_op_fx65_ld_vx_i() {
        // This test requires implementing the Fx65 opcode first.
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        for i in 0..=5 {
            chip8.memory[0x300 + i] = i as u8;
        }

        run_instruction(&mut chip8, 0xF565).unwrap();
        for i in 0..=5 {
            assert_eq!(chip8.registers[i], i as u8);
        }
    }

    #[test]
    fn test_invalid_opcode() {
        let mut chip8 = Chip8::new().unwrap();
        // 0x0FFF is not a valid opcode
        let result = run_instruction(&mut chip8, 0x0FFF);
        assert!(matches!(result, Err(Chip8Error::InvalidOpCode(_))));
    }

    #[test]
    fn test_op_00e0_cls() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.framebuffer.iter_mut().for_each(|p| *p = 1);
        chip8.display_updated = false;
        run_instruction(&mut chip8, 0x00E0).unwrap();
        assert!(chip8.framebuffer.iter().all(|&p| p == 0));
        assert!(chip8.is_display_updated());
    }

    #[test]
    fn test_op_dxyn_drw() {
        let mut chip8 = Chip8::new().unwrap();
        // Load a simple 8x1 sprite (a horizontal line) into memory at 0x300
        chip8.i = 0x300;
        chip8.memory[0x300] = 0xFF;
        // Set Vx and Vy to draw at (10, 5)
        chip8.registers[1] = 10;
        chip8.registers[2] = 5;

        // Draw a sprite of height 1 from register V1, V2
        run_instruction(&mut chip8, 0xD121).unwrap();

        // Check that the pixels are set correctly
        for i in 0..8 {
            assert_eq!(chip8.framebuffer[5 * 64 + (10 + i)], 1);
        }
        // Check that VF is 0 (no collision)
        assert_eq!(chip8.registers[0xF], 0);
        assert!(chip8.is_display_updated());
    }

    #[test]
    fn test_op_dxyn_drw_collision() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        chip8.memory[0x300] = 0b11000000; // Sprite to draw
        chip8.registers[1] = 10;
        chip8.registers[2] = 5;

        // Pre-set a pixel that will collide
        chip8.framebuffer[5 * 64 + 10] = 1;

        run_instruction(&mut chip8, 0xD121).unwrap();

        // The first pixel was on, it should be turned off
        assert_eq!(chip8.framebuffer[5 * 64 + 10], 0);
        // The second pixel was off, it should be turned on
        assert_eq!(chip8.framebuffer[5 * 64 + 11], 1);
        // Check that VF is 1 (collision)
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_op_fx0a_ld_vx_k_wait() {
        let mut chip8 = Chip8::new().unwrap();
        let initial_pc = chip8.pc;
        // Run without a key press
        run_instruction(&mut chip8, 0xF30A).unwrap();
        // PC should be rewound, effectively pausing execution
        assert_eq!(chip8.pc, initial_pc);
    }

    #[test]
    fn test_op_fx0a_ld_vx_k_press() {
        let mut chip8 = Chip8::new().unwrap();
        let initial_pc = chip8.pc;
        // Simulate key press for key 0xA
        chip8.key_press(0xA);
        run_instruction(&mut chip8, 0xF30A).unwrap();
        // Register V3 should contain 0xA
        assert_eq!(chip8.registers[3], 0xA);
        // PC should advance normally
        assert_eq!(chip8.pc, initial_pc + 2);
    }
}
