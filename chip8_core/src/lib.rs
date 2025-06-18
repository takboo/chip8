/// Standard CHIP-8 font set (hex digits 0-F)
/// Each digit is 5 bytes representing a 8x5 pixel sprite
const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// Memory address where font sprites are loaded
const FONT_START_ADDRESS: usize = 0x50;

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
}

#[derive(Debug, thiserror::Error)]
pub enum Chip8Error {
    #[error("Font-set is out of bounds")]
    LoadFontSetError,
}

impl Chip8 {
    /// Create a new Chip8 instance
    ///
    /// Initializes the Chip8 instance with default values.
    /// Most Chip-8 programs start at location 0x200 (512), but some begin at 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660 computer.
    /// rom location 0x000 (0) to 0xFFF (4095).
    /// The first 512 bytes, from 0x000 to 0x1FF, are where the original interpreter was located, and should not be used by programs.
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
        };
        chip8.load_font_at(FONT_START_ADDRESS, &FONT_SET)?;
        Ok(chip8)
    }

    /// Reset the Chip8 instance
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

        self.load_font_at(FONT_START_ADDRESS, &FONT_SET)?;
        Ok(())
    }

    fn load_font_at(&mut self, start_address: usize, font: &[u8]) -> Result<(), Chip8Error> {
        if let Some(memory) = self
            .memory
            .get_mut(start_address..start_address + font.len())
        {
            memory.clone_from_slice(font);
            Ok(())
        } else {
            Err(Chip8Error::LoadFontSetError)
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
}
