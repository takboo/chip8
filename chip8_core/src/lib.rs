use rand::Rng;

/// Standard CHIP-8 font set (hex digits 0-F)
/// Each digit is 5 bytes representing an 8x5 pixel sprite
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

/// Memory address where rom are loaded
const ROM_START_ADDRESS: usize = 0x200;

/// Decoded representation of a single 16-bit CHIP-8 instruction.
///
/// This struct breaks down an opcode into its constituent parts (e.g., `instr`, `x`, `y`, `n`, `nn`, `nnn`)
/// for easier processing during the execution phase.
struct Instructions {
    /// The primary 4-bit instruction identifier.
    instr: u8,
    /// A 4-bit value, often used as a register index.
    x: usize,
    /// A 4-bit value, often used as a register index.
    y: usize,
    /// A 4-bit value, often used for small constants.
    n: u8,
    /// An 8-bit value, often used for immediate values.
    nn: u8,
    /// A 12-bit value, typically representing a memory address.
    nnn: u16,
}

impl Instructions {
    /// Creates a new `Instructions` instance by decoding a 16-bit opcode.
    fn new(instruction: u16) -> Self {
        let instr = ((instruction & 0xF000) >> 12) as u8;
        let x = ((instruction & 0x0F00) >> 8) as usize;
        let y = ((instruction & 0x00F0) >> 4) as usize;
        let n = (instruction & 0x000F) as u8;
        let nn = (instruction & 0x00FF) as u8;
        let nnn = instruction & 0x0FFF;

        Self {
            instr,
            x,
            y,
            n,
            nn,
            nnn,
        }
    }

    /// Returns the primary 4-bit instruction identifier.
    pub fn instruction(&self) -> u8 {
        self.instr
    }

    /// Returns the `x` component of the instruction (a 4-bit register index).
    pub fn x(&self) -> usize {
        self.x
    }
    /// Returns the `y` component of the instruction (a 4-bit register index).
    pub fn y(&self) -> usize {
        self.y
    }

    /// Returns the `n` component of the instruction (a 4-bit value).
    pub fn n(&self) -> u8 {
        self.n
    }

    /// Returns the `nn` component of the instruction (an 8-bit value).
    pub fn nn(&self) -> u8 {
        self.nn
    }

    /// Returns the `nnn` component of the instruction (a 12-bit address).
    pub fn nnn(&self) -> u16 {
        self.nnn
    }
}

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
}

/// Defines the possible errors that can occur during CHIP-8 emulation.
#[derive(Debug, thiserror::Error)]
pub enum Chip8Error {
    /// Occurs when the font set cannot be loaded into memory because it would exceed the memory bounds.
    #[error("Font-set is out of bounds")]
    LoadFontSetError,
    /// Occurs when a ROM cannot be loaded because it is too large to fit in the available memory space.
    #[error("ROM is out of bounds")]
    LoadRomError,
    /// Occurs when the program counter (`pc`) points to an invalid memory location for an instruction fetch.
    #[error("PC points to an invalid memory: {0}")]
    PCError(u16),
    #[error("Invalid opcode")]
    InvalidOpCode,
    #[error("SP {0} is out of bounds")]
    SPError(u8),
    #[error("SP {0} is overflow")]
    SPOverflow(u8),
    #[error("Index register points to an invalid memory: {0}")]
    IndexError(u16),
    #[error("Invalid register: V{0}")]
    InvalidRegister(usize),
    #[error("Invalid key: {0}")]
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

    pub fn run(&mut self) -> Result<(), Chip8Error> {
        let instruction = self.fetch()?;
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );
        let nn = instruction.nn();
        let nnn = instruction.nnn();

        match (instr, x, y, n) {
            (0, 0, 0xE, 0) => {
                dbg!("clear screen");
                Ok(())
            }
            (1, _, _, _) => {
                self.pc = nnn;
                dbg!("jump to nnn");
                Ok(())
            }
            (2, _, _, _) => {
                self.push_stack()?;
                self.pc = nnn;
                dbg!("call a subroutine, should push pc to stack");
                Ok(())
            }
            (0, 0, 0xE, 0xE) => {
                self.pop_stack()?;
                dbg!("return, pop pc from stack");
                Ok(())
            }
            (3, _, _, _) => {
                // read Vx register
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                if vx == nn {
                    self.pc = self.pc.wrapping_add(2);
                }
                dbg!("condition check vx equal nn");
                Ok(())
            }
            (4, _, _, _) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                if vx != nn {
                    self.pc = self.pc.wrapping_add(2);
                }
                dbg!("condition check vx not equal nn");
                Ok(())
            }
            (5, _, _, _) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                if vx == vy {
                    self.pc = self.pc.wrapping_add(2);
                }
                dbg!("condition check vy equal vx");
                Ok(())
            }
            (9, _, _, _) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                if vx != vy {
                    self.pc = self.pc.wrapping_add(2);
                }
                dbg!("condition check vy not equal vx");
                Ok(())
            }
            (6, _, _, _) => {
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx = nn;
                dbg!("set vx to nn");
                Ok(())
            }
            (7, _, _, _) => {
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx = vx.wrapping_add(nn);
                dbg!("add nn to vx");
                Ok(())
            }
            (8, _, _, 0) => {
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx = vy;
                dbg!("set vy to vx");
                Ok(())
            }
            (8, _, _, 1) => {
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx |= vy;
                dbg!("binary OR vy to vx");
                Ok(())
            }
            (8, _, _, 2) => {
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx &= vy;
                dbg!("binary AND vy to vx");
                Ok(())
            }
            (8, _, _, 3) => {
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx ^= vy;
                dbg!("binary XOR vy to vx");
                Ok(())
            }
            (8, _, _, 4) => {
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;

                let (result, is_overflow) = vx.overflowing_add(vy);
                *vx = result;
                let vf = self
                    .registers
                    .last_mut()
                    .ok_or(Chip8Error::InvalidRegister(0xf))?;
                *vf = is_overflow as u8;
                dbg!("add vy to vx with overflow flag setup");
                Ok(())
            }
            (8, _, _, 5) => {
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                let (result, borrow) = vx.overflowing_sub(vy);
                *vx = result;
                let vf = self
                    .registers
                    .last_mut()
                    .ok_or(Chip8Error::InvalidRegister(0xf))?;
                *vf = !borrow as u8;
                dbg!("sub vy from vx");
                Ok(())
            }
            (8, _, _, 7) => {
                let &vy = self
                    .registers
                    .get(y)
                    .ok_or(Chip8Error::InvalidRegister(y))?;
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                let (result, borrow) = vy.overflowing_sub(*vx);
                *vx = result;
                let vf = self
                    .registers
                    .last_mut()
                    .ok_or(Chip8Error::InvalidRegister(0xf))?;
                *vf = !borrow as u8;
                dbg!("sub vx from vy then set vx");
                Ok(())
            }
            (8, _, _, 6) => {
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                // shift vx one bit to the right, then set vf to the one bit was shifted out
                let shifted_out = *vx & 0x1;
                *vx >>= 1;
                let vf = self
                    .registers
                    .last_mut()
                    .ok_or(Chip8Error::InvalidRegister(0xf))?;
                *vf = shifted_out;
                dbg!("shift vx one bit to the right then set vf to the one bit was shifted out");
                Ok(())
            }
            (8, _, _, 0xE) => {
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                // shift vx one bit to the left, then set vf to the one bit was shifted out
                let shifted_out = (*vx >> 7) & 0x1;
                *vx <<= 1;
                let vf = self
                    .registers
                    .last_mut()
                    .ok_or(Chip8Error::InvalidRegister(0xf))?;
                *vf = shifted_out;
                dbg!("shift vx one bit to the left then set vf to the one bit was shifted out");
                Ok(())
            }
            (0xa, _, _, _) => {
                self.i = nnn;
                dbg!("set i to nnn");
                Ok(())
            }
            (0xb, _, _, _) => {
                let &v0 = self
                    .registers
                    .first()
                    .ok_or(Chip8Error::InvalidRegister(0x0))?;
                self.pc = nnn.wrapping_add(v0 as u16);
                dbg!("set pc to v0 + nnn");
                Ok(())
            }
            (0xc, _, _, _) => {
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx = rand::rng().random_range(0..=255) & nn;
                dbg!("set vx to random number and nn");
                Ok(())
            }
            (0xd, _, _, _) => {
                dbg!("draw sprite at vx, vy, n");
                Ok(())
            }
            (0xe, _, 0x9, 0xe) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                let &key = self
                    .keyboard
                    .get(vx as usize)
                    .ok_or(Chip8Error::InvalidKey(vx))?;
                if key != 0 {
                    self.pc = self.pc.wrapping_add(2);
                }
                dbg!("skip next instruction if key is pressed");
                Ok(())
            }
            (0xe, _, 0xa, 0x1) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                let &key = self
                    .keyboard
                    .get(vx as usize)
                    .ok_or(Chip8Error::InvalidKey(vx))?;
                if key == 0 {
                    self.pc = self.pc.wrapping_add(2);
                }
                dbg!("skip next instruction if key is not pressed");
                Ok(())
            }
            (0xf, _, 0x0, 0x7) => {
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx = self.dt;
                dbg!("set vx to dt");
                Ok(())
            }
            (0xf, _, 0x1, 0x5) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                self.dt = vx;
                dbg!("set dt to vx");
                Ok(())
            }
            (0xf, _, 0x1, 0x8) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                self.st = vx;
                dbg!("set st to vx");
                Ok(())
            }
            (0xf, _, 0x1, 0xe) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                self.i = self.i.wrapping_add(vx as u16);
                dbg!("add vx to i");
                Ok(())
            }
            (0xf, _, 0x0, 0xa) => {
                // wait for a keypress, store the value of the key in vx
                dbg!("wait for a keypress, store the value of the key in vx");
                Ok(())
            }
            (0xf, _, 0x2, 0x9) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                self.i = vx as u16;
                dbg!("set i to vx");
                Ok(())
            }
            (0xf, _, 0x3, 0x3) => {
                let &vx = self
                    .registers
                    .get(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                let memory = self
                    .memory
                    .get_mut(self.i as usize..self.i as usize + 3)
                    .ok_or(Chip8Error::IndexError(self.i))?;
                memory[0] = vx / 100;
                memory[1] = (vx % 100) / 10;
                memory[2] = vx % 10;
                dbg!(
                    "convert vx to three decimal digits and store in memory at the address in the index register I"
                );
                Ok(())
            }
            (0xf, _, 0x5, 0x5) => {
                let memory = self
                    .memory
                    .get_mut(self.i as usize..=self.i as usize + x)
                    .ok_or(Chip8Error::IndexError(self.i))?;
                for (i, register) in self.registers.iter().enumerate() {
                    if i > x {
                        break;
                    }
                    memory[i] = *register;
                }
                dbg!("copy registers to memory at the address in the index register I");
                Ok(())
            }
            (0xf, _, 0x6, 0x5) => {
                let memory = self
                    .memory
                    .get_mut(self.i as usize..=self.i as usize + x)
                    .ok_or(Chip8Error::IndexError(self.i))?;
                for (i, register) in self.registers.iter_mut().enumerate() {
                    if i > x {
                        break;
                    }
                    *register = memory[i];
                }
                dbg!("copy memory to registers at the address in the index register I");
                Ok(())
            }

            _ => Err(Chip8Error::InvalidOpCode),
        }
    }

    /// Fetches the next instruction from memory at the current program counter (`pc`),
    /// decodes it, and advances the `pc` by two bytes.
    ///
    /// # Returns
    ///
    /// * `Ok(Instructions)` containing the decoded instruction.
    /// * `Err(Chip8Error::PCError)` if the `pc` is at or near the end of memory,
    ///   making it impossible to fetch a full 2-byte instruction.
    fn fetch(&mut self) -> Result<Instructions, Chip8Error> {
        if let Some(instruction_bytes) = self.memory.get(self.pc as usize..self.pc as usize + 2) {
            self.pc = self.pc.wrapping_add(2);
            let instruction = (instruction_bytes[0] as u16) << 8 | instruction_bytes[1] as u16;
            Ok(Instructions::new(instruction))
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
        let decoded = Instructions::new(instruction);
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
        assert!(matches!(result, Err(Chip8Error::InvalidOpCode)));
    }
}
