/// Decoded representation of a single 16-bit CHIP-8 instruction.
///
/// Opcodes in CHIP-8 are 16 bits long. This struct breaks down an opcode
/// into its constituent parts for easier processing by the emulator's execution logic.
///
/// The parts are:
/// - `instr`: The most significant 4 bits, identifying the instruction type.
/// - `x`: The lower 4 bits of the high byte, typically a register index.
/// - `y`: The upper 4 bits of the low byte, typically another register index.
/// - `n`: The lowest 4 bits, a nibble.
/// - `nn`: The lowest 8 bits, a byte.
/// - `nnn`: The lowest 12 bits, an address.
#[derive(Debug, PartialEq, Eq)]
pub struct Instruction {
    /// The most significant 4 bits of the opcode, identifying the instruction group.
    /// Also known as the "opcode type".
    instr: u8,
    /// The second nibble of the opcode. Often used to identify one of the 16 registers (V0-VF), `Vx`.
    x: usize,
    /// The third nibble of the opcode. Often used to identify one of the 16 registers (V0-VF), `Vy`.
    y: usize,
    /// The fourth nibble (least significant) of the opcode.
    /// This can represent a 4-bit immediate value.
    n: u8,
    /// The lower 8 bits (least significant byte) of the opcode.
    /// This can represent an 8-bit immediate value.
    nn: u8,
    /// The lower 12 bits of the opcode.
    /// This is typically used for memory addresses.
    nnn: u16,
}

impl Instruction {
    /// Creates a new `Instruction` instance by decoding a 16-bit opcode.
    ///
    /// The opcode is masked and shifted to extract the different components.
    ///
    /// # Arguments
    ///
    /// * `opcode`: The 16-bit CHIP-8 opcode to decode.
    pub fn new(opcode: u16) -> Self {
        let instr = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        Self {
            instr,
            x,
            y,
            n,
            nn,
            nnn,
        }
    }

    /// Returns the primary 4-bit instruction identifier (`instr`).
    pub fn instruction(&self) -> u8 {
        self.instr
    }

    /// Returns the `x` component of the instruction, typically a register index.
    pub fn x(&self) -> usize {
        self.x
    }
    /// Returns the `y` component of the instruction, typically a register index.
    pub fn y(&self) -> usize {
        self.y
    }

    /// Returns the `n` component of the instruction, a 4-bit immediate value.
    pub fn n(&self) -> u8 {
        self.n
    }

    /// Returns the `nn` component of the instruction, an 8-bit immediate value.
    pub fn nn(&self) -> u8 {
        self.nn
    }

    /// Returns the `nnn` component of the instruction, a 12-bit address.
    pub fn nnn(&self) -> u16 {
        self.nnn
    }
}

impl std::fmt::Display for Instruction {
    /// Formats the instruction for display purposes.
    ///
    /// This is useful for debugging, as it provides a human-readable representation
    /// of the decoded instruction's components.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "instr: {}\tx: {}\ty: {}\tn: {}\tnn: {}\tnnn: {}",
            self.instruction(),
            self.x(),
            self.y(),
            self.n(),
            self.nn(),
            self.nnn()
        )
    }
}
