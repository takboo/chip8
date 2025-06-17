pub struct Chip8 {
    /// Memory of the Chip8
    memory: [u8; 4096],

    /// Registers of the Chip8
    ///
    /// 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    /// The VF register should not be used by any program, as it is used as a flag by some instructions.
    registers: [u8; 16],

    /// Program Counter of the Chip8
    pc: u16,

    /// Stack Pointer of the Chip8
    sp: u8,

    /// Index Register of the Chip8
    ///
    /// a 16-bit register called I. This register is generally used to store memory addresses, so only the lowest (rightmost) 12 bits are usually used.
    i: u16,

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

impl Chip8 {
    /// Create a new Chip8 instance
    ///
    /// Initializes the Chip8 instance with default values.
    /// Most Chip-8 programs start at location 0x200 (512), but some begin at 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660 computer.
    /// rom location 0x000 (0) to 0xFFF (4095).
    /// The first 512 bytes, from 0x000 to 0x1FF, are where the original interpreter was located, and should not be used by programs.
    pub fn new() -> Self {
        Self {
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
        }
    }

    /// Reset the Chip8 instance
    pub fn reset(&mut self) {
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
    }
}
