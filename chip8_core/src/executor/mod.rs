//! Instruction execution module for the CHIP-8 emulator.
//!
//! This module provides the core instruction execution logic, organized by instruction types
//! for better maintainability and performance. Instructions are first categorized by their
//! functional type (flow control, arithmetic, memory operations, etc.) and then dispatched
//! to specialized handler methods.

use crate::instruction::{Instruction, InstructionType};
use crate::{Chip8, Chip8Error};

pub mod arithmetic;
pub mod display;
pub mod flow_control;
pub mod input;
pub mod memory;

impl Chip8 {
    /// Executes a single CHIP-8 instruction.
    ///
    /// This method uses a two-stage dispatch mechanism:
    /// 1. First, it determines the instruction type using the `InstructionType` enum
    /// 2. Then, it dispatches to the appropriate handler method based on the type
    ///
    /// This approach provides better code organization and can improve performance
    /// by reducing the number of pattern matches needed.
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded instruction to execute
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred during execution
    pub(super) fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        match instruction.instruction_type() {
            InstructionType::FlowControl => self.execute_flow_control(instruction),
            InstructionType::ConditionalSkip => self.execute_conditional_skip(instruction),
            InstructionType::RegisterOp => self.execute_register_operation(instruction),
            InstructionType::MemoryOp => self.execute_memory_operation(instruction),
            InstructionType::Display => self.execute_display_operation(instruction),
            InstructionType::InputOutput => self.execute_input_output(instruction),
            InstructionType::Timer => self.execute_timer_operation(instruction),
            InstructionType::Random => self.execute_random_operation(instruction),
        }
    }

    /// Executes flow control instructions that change program execution flow.
    ///
    /// Handles instructions like:
    /// - 0x00EE: Return from subroutine
    /// - 0x1NNN: Jump to address
    /// - 0x2NNN: Call subroutine
    /// - 0xBNNN: Jump to V0 + NNN
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded flow control instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., stack overflow, invalid address)
    fn execute_flow_control(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );
        let nnn = instruction.nnn();

        match (instr, x, y, n) {
            (0, 0, 0xE, 0xE) => self.return_from_subroutine(),
            (1, _, _, _) => self.jump_to_address(nnn),
            (2, _, _, _) => self.call_subroutine(nnn),
            (0xB, _, _, _) => self.jump_to_v0_plus_nnn(nnn),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid flow control opcode: {}",
                instruction
            ))),
        }
    }

    /// Executes conditional skip instructions that may skip the next instruction.
    ///
    /// Handles instructions like:
    /// - 0x3XNN: Skip if Vx == NN
    /// - 0x4XNN: Skip if Vx != NN
    /// - 0x5XY0: Skip if Vx == Vy
    /// - 0x9XY0: Skip if Vx != Vy
    /// - 0xEX9E: Skip if key pressed
    /// - 0xEXA1: Skip if key not pressed
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded conditional skip instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., invalid register or key)
    fn execute_conditional_skip(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );
        let nn = instruction.nn();

        match (instr, x, y, n) {
            (3, _, _, _) => self.skip_if_vx_equals_nn(x, nn),
            (4, _, _, _) => self.skip_if_vx_not_equals_nn(x, nn),
            (5, _, _, 0) => self.skip_if_vx_equals_vy(x, y),
            (9, _, _, 0) => self.skip_if_vx_not_equals_vy(x, y),
            (0xE, _, 0x9, 0xE) => self.skip_if_key_pressed(x),
            (0xE, _, 0xA, 0x1) => self.skip_if_key_not_pressed(x),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid conditional skip opcode: {}",
                instruction
            ))),
        }
    }

    /// Executes register operation instructions that directly manipulate register values.
    ///
    /// Handles instructions like:
    /// - 0x6XNN: Set Vx = NN
    /// - 0x7XNN: Add NN to Vx
    /// - 0x8XY_: Various arithmetic and logical operations between registers
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded register operation instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., invalid register)
    fn execute_register_operation(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );
        let nn = instruction.nn();

        match (instr, x, y, n) {
            (6, _, _, _) => self.set_vx_to_nn(x, nn),
            (7, _, _, _) => self.add_nn_to_vx(x, nn),
            (8, _, _, 0) => self.set_vx_to_vy(x, y),
            (8, _, _, 1) => self.or_vx_vy(x, y),
            (8, _, _, 2) => self.and_vx_vy(x, y),
            (8, _, _, 3) => self.xor_vx_vy(x, y),
            (8, _, _, 4) => self.add_vx_vy(x, y),
            (8, _, _, 5) => self.sub_vx_vy(x, y),
            (8, _, _, 6) => self.shift_vx_right(x),
            (8, _, _, 7) => self.sub_vy_vx(x, y),
            (8, _, _, 0xE) => self.shift_vx_left(x),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid register operation opcode: {}",
                instruction
            ))),
        }
    }

    /// Executes memory operation instructions that involve memory access.
    ///
    /// Handles instructions like:
    /// - 0xANNN: Set I = NNN
    /// - 0xFX1E: Add Vx to I
    /// - 0xFX29: Set I to font location for digit Vx
    /// - 0xFX33: Store BCD representation of Vx
    /// - 0xFX55: Store registers V0-Vx to memory
    /// - 0xFX65: Load registers V0-Vx from memory
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded memory operation instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., memory access out of bounds)
    fn execute_memory_operation(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );
        let nnn = instruction.nnn();

        match (instr, x, y, n) {
            (0xA, _, _, _) => self.set_i_to_nnn(nnn),
            (0xF, _, 0x1, 0xE) => self.add_vx_to_i(x),
            (0xF, _, 0x2, 0x9) => self.set_i_to_font_location(x),
            (0xF, _, 0x3, 0x3) => self.store_bcd_of_vx(x),
            (0xF, _, 0x5, 0x5) => self.store_registers_to_memory(x),
            (0xF, _, 0x6, 0x5) => self.load_registers_from_memory(x),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid memory operation opcode: {}",
                instruction
            ))),
        }
    }

    /// Executes display operation instructions for graphics rendering.
    ///
    /// Handles instructions like:
    /// - 0x00E0: Clear screen
    /// - 0xDXYN: Draw sprite at (Vx, Vy) with height N
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded display operation instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., framebuffer overflow)
    fn execute_display_operation(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );

        match (instr, x, y, n) {
            (0, 0, 0xE, 0) => self.clear_screen(),
            (0xD, _, _, _) => self.draw_sprite(x, y, n),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid display operation opcode: {}",
                instruction
            ))),
        }
    }

    /// Executes input/output instructions for keyboard and user interaction.
    ///
    /// Handles instructions like:
    /// - 0xFX0A: Wait for key press and store key value in Vx
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded input/output instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., invalid key or register)
    fn execute_input_output(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );

        match (instr, x, y, n) {
            (0xF, _, 0x0, 0xA) => self.wait_for_key_press(x),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid input/output opcode: {}",
                instruction
            ))),
        }
    }

    /// Executes timer operation instructions for delay and sound timers.
    ///
    /// Handles instructions like:
    /// - 0xFX07: Set Vx to delay timer value
    /// - 0xFX15: Set delay timer to Vx
    /// - 0xFX18: Set sound timer to Vx
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded timer operation instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., invalid register)
    fn execute_timer_operation(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );

        match (instr, x, y, n) {
            (0xF, _, 0x0, 0x7) => self.set_vx_to_delay_timer(x),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer_to_vx(x),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer_to_vx(x),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid timer operation opcode: {}",
                instruction
            ))),
        }
    }

    /// Executes random number generation instructions.
    ///
    /// Handles instructions like:
    /// - 0xCXNN: Set Vx to (random number) AND NN
    ///
    /// # Arguments
    ///
    /// * `instruction` - The decoded random operation instruction
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the instruction was executed successfully
    /// * `Err(Chip8Error)` - If an error occurred (e.g., invalid register)
    fn execute_random_operation(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );
        let nn = instruction.nn();

        match (instr, x, y, n) {
            (0xC, _, _, _) => self.set_vx_to_random_and_nn(x, nn),
            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid random operation opcode: {}",
                instruction
            ))),
        }
    }
}
