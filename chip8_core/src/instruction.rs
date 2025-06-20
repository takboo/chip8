/// Categories of CHIP-8 instructions based on their functionality.
///
/// This enum provides a high-level classification of instruction types,
/// which can be useful for analysis, debugging, and optimization purposes.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InstructionType {
    /// Flow control instructions that change program execution flow.
    /// Includes: 0x00EE (ret), 0x1NNN (jmp), 0x2NNN (call), 0xBNNN (jmp+v0)
    FlowControl,

    /// Conditional skip instructions that may skip the next instruction.
    /// Includes: 0x3XNN, 0x4XNN, 0x5XY0, 0x9XY0, 0xEX9E, 0xEXA1
    ConditionalSkip,

    /// Register operation instructions that directly manipulate register values.
    /// Includes: 0x6XNN (ld), 0x7XNN (add), 0x8XY_ (arithmetic operations)
    RegisterOp,

    /// Memory operation instructions that involve memory access.
    /// Includes: 0xANNN, 0xFX1E, 0xFX29, 0xFX33, 0xFX55, 0xFX65
    MemoryOp,

    /// Display operation instructions for graphics rendering.
    /// Includes: 0x00E0 (cls), 0xDXYN (draw)
    Display,

    /// Input/output instructions for keyboard and user interaction.
    /// Includes: 0xFX0A (wait key)
    InputOutput,

    /// Timer operation instructions for delay and sound timers.
    /// Includes: 0xFX07, 0xFX15, 0xFX18
    Timer,

    /// Random number generation instructions.
    /// Includes: 0xCXNN
    Random,
}

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

    /// Returns the instruction type classification for this instruction.
    ///
    /// This method analyzes the opcode pattern and returns the appropriate
    /// `InstructionType` enum variant based on the instruction's functionality.
    ///
    /// # Returns
    ///
    /// An `InstructionType` enum indicating the category of this instruction.
    pub fn instruction_type(&self) -> InstructionType {
        match (self.instr, self.x, self.y, self.n) {
            // Flow control instructions
            (0, 0, 0xE, 0xE) => InstructionType::FlowControl, // Return from subroutine
            (1, _, _, _) => InstructionType::FlowControl,     // Jump to address
            (2, _, _, _) => InstructionType::FlowControl,     // Call subroutine
            (0xB, _, _, _) => InstructionType::FlowControl,   // Jump to V0 + NNN

            // Conditional skip instructions
            (3, _, _, _) => InstructionType::ConditionalSkip, // Skip if Vx == NN
            (4, _, _, _) => InstructionType::ConditionalSkip, // Skip if Vx != NN
            (5, _, _, 0) => InstructionType::ConditionalSkip, // Skip if Vx == Vy
            (9, _, _, 0) => InstructionType::ConditionalSkip, // Skip if Vx != Vy
            (0xE, _, 0x9, 0xE) => InstructionType::ConditionalSkip, // Skip if key pressed
            (0xE, _, 0xA, 0x1) => InstructionType::ConditionalSkip, // Skip if key not pressed

            // Register operation instructions
            (6, _, _, _) => InstructionType::RegisterOp, // Set Vx = NN
            (7, _, _, _) => InstructionType::RegisterOp, // Add NN to Vx
            (8, _, _, _) => InstructionType::RegisterOp, // All arithmetic operations

            // Memory operation instructions
            (0xA, _, _, _) => InstructionType::MemoryOp, // Set I = NNN
            (0xF, _, 0x1, 0xE) => InstructionType::MemoryOp, // Add Vx to I
            (0xF, _, 0x2, 0x9) => InstructionType::MemoryOp, // Set I to font location
            (0xF, _, 0x3, 0x3) => InstructionType::MemoryOp, // Store BCD of Vx
            (0xF, _, 0x5, 0x5) => InstructionType::MemoryOp, // Store registers to memory
            (0xF, _, 0x6, 0x5) => InstructionType::MemoryOp, // Load registers from memory

            // Display instructions
            (0, 0, 0xE, 0) => InstructionType::Display, // Clear screen
            (0xD, _, _, _) => InstructionType::Display, // Draw sprite

            // Input/output instructions
            (0xF, _, 0x0, 0xA) => InstructionType::InputOutput, // Wait for key press

            // Timer instructions
            (0xF, _, 0x0, 0x7) => InstructionType::Timer, // Set Vx to delay timer
            (0xF, _, 0x1, 0x5) => InstructionType::Timer, // Set delay timer to Vx
            (0xF, _, 0x1, 0x8) => InstructionType::Timer, // Set sound timer to Vx

            // Random number generation
            (0xC, _, _, _) => InstructionType::Random, // Set Vx to random & NN

            // Default case - this should not happen for valid instructions
            _ => InstructionType::FlowControl, // Default fallback
        }
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
