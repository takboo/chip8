//! Flow control instruction implementations for CHIP-8.
//!
//! This module contains implementations for instructions that control program flow,
//! including jumps, subroutine calls, and conditional skip operations. These instructions
//! are fundamental to program execution and control structure in CHIP-8 programs.

use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// **00E0 - CLS**: Clear the display screen.
    ///
    /// This instruction clears the entire 64x32 pixel display by setting all pixels to 0.
    /// It also sets the display_updated flag to indicate that the screen needs to be redrawn.
    ///
    /// # Errors
    ///
    /// This instruction should not fail under normal circumstances.
    ///
    /// # Side Effects
    ///
    /// - Clears all pixels in the framebuffer
    /// - Sets the display_updated flag to true
    pub(super) fn clear_screen(&mut self) -> Result<(), Chip8Error> {
        self.framebuffer.iter_mut().for_each(|p| *p = 0);
        self.display_updated = true;

        Ok(())
    }

    /// **00EE - RET**: Return from a subroutine.
    ///
    /// This instruction returns from a subroutine by popping the return address
    /// from the stack and setting the program counter to that address.
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::SPOverflow` if the stack is empty (stack underflow).
    /// Returns `Chip8Error::SPError` if the stack pointer is invalid.
    ///
    /// # Side Effects
    ///
    /// - Decrements the stack pointer
    /// - Sets the program counter to the address popped from the stack
    pub(super) fn return_from_subroutine(&mut self) -> Result<(), Chip8Error> {
        self.pop_stack()?;

        Ok(())
    }

    /// **1NNN - JP addr**: Jump to address NNN.
    ///
    /// This instruction sets the program counter to the address NNN, causing
    /// the program to continue execution from that address.
    ///
    /// # Arguments
    ///
    /// * `nnn` - 12-bit address to jump to (0x000-0xFFF)
    ///
    /// # Errors
    ///
    /// This instruction should not fail, but the target address should be valid
    /// for program execution to continue properly.
    ///
    /// # Side Effects
    ///
    /// Sets the program counter to the specified address.
    pub(super) fn jump_to_address(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        self.pc = nnn;

        Ok(())
    }

    /// **2NNN - CALL addr**: Call subroutine at address NNN.
    ///
    /// This instruction pushes the current program counter onto the stack and then
    /// sets the program counter to NNN, effectively calling a subroutine.
    ///
    /// # Arguments
    ///
    /// * `nnn` - 12-bit address of the subroutine to call (0x000-0xFFF)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::SPOverflow` if the stack is full (stack overflow).
    /// Returns `Chip8Error::SPError` if the stack pointer is invalid.
    ///
    /// # Side Effects
    ///
    /// - Pushes the current program counter onto the stack
    /// - Increments the stack pointer
    /// - Sets the program counter to the specified address
    pub(super) fn call_subroutine(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        self.push_stack()?;
        self.pc = nnn;

        Ok(())
    }

    /// **3XNN - SE Vx, byte**: Skip next instruction if Vx equals NN.
    ///
    /// This instruction compares the value in register Vx with the immediate value NN.
    /// If they are equal, the program counter is incremented by 2 (skipping the next instruction).
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15)
    /// * `nn` - 8-bit immediate value to compare against
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// May increment the program counter by 2 if the condition is true.
    pub(super) fn skip_if_vx_equals_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        if vx == nn {
            self.pc = self.pc.wrapping_add(2);
        }

        Ok(())
    }

    /// **4XNN - SNE Vx, byte**: Skip next instruction if Vx does not equal NN.
    ///
    /// This instruction compares the value in register Vx with the immediate value NN.
    /// If they are not equal, the program counter is incremented by 2 (skipping the next instruction).
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15)
    /// * `nn` - 8-bit immediate value to compare against
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// May increment the program counter by 2 if the condition is true.
    pub(super) fn skip_if_vx_not_equals_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        if vx != nn {
            self.pc = self.pc.wrapping_add(2);
        }

        Ok(())
    }

    /// **5XY0 - SE Vx, Vy**: Skip next instruction if Vx equals Vy.
    ///
    /// This instruction compares the values in registers Vx and Vy.
    /// If they are equal, the program counter is incremented by 2 (skipping the next instruction).
    ///
    /// # Arguments
    ///
    /// * `x` - First register index (0-15)
    /// * `y` - Second register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// May increment the program counter by 2 if the condition is true.
    pub(super) fn skip_if_vx_equals_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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

        Ok(())
    }

    /// **9XY0 - SNE Vx, Vy**: Skip next instruction if Vx does not equal Vy.
    ///
    /// This instruction compares the values in registers Vx and Vy.
    /// If they are not equal, the program counter is incremented by 2 (skipping the next instruction).
    ///
    /// # Arguments
    ///
    /// * `x` - First register index (0-15)
    /// * `y` - Second register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// May increment the program counter by 2 if the condition is true.
    pub(super) fn skip_if_vx_not_equals_vy(
        &mut self,
        x: usize,
        y: usize,
    ) -> Result<(), Chip8Error> {
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

        Ok(())
    }

    /// **BNNN - JP V0, addr**: Jump to address NNN plus V0.
    ///
    /// This instruction adds the value in register V0 to the address NNN and
    /// sets the program counter to the result. This is useful for implementing
    /// jump tables and computed jumps.
    ///
    /// # Arguments
    ///
    /// * `nnn` - 12-bit base address (0x000-0xFFF)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if V0 cannot be accessed (unlikely).
    ///
    /// # Side Effects
    ///
    /// Sets the program counter to NNN + V0 (with wrapping if necessary).
    ///
    /// # Examples
    ///
    /// If V0 contains 0x02 and NNN is 0x300, the program will jump to address 0x302.
    pub(super) fn jump_to_v0_plus_nnn(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        let &v0 = self
            .registers
            .first()
            .ok_or(Chip8Error::InvalidRegister(0x0))?;
        self.pc = nnn.wrapping_add(v0 as u16);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{tests::run_instruction, *};

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
    fn test_op_4xkk_sne_vx_byte_skip() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[3] = 0x42;
        let initial_pc = chip8.pc;
        run_instruction(&mut chip8, 0x4343).unwrap(); // Different value, use 4xxx for SNE
        assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
    }

    #[test]
    fn test_op_5xy0_se_vx_vy_skip() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0x42;
        chip8.registers[2] = 0x42;
        let initial_pc = chip8.pc;
        run_instruction(&mut chip8, 0x5120).unwrap();
        assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
    }

    #[test]
    fn test_op_9xy0_sne_vx_vy_skip() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0x42;
        chip8.registers[2] = 0x43;
        let initial_pc = chip8.pc;
        run_instruction(&mut chip8, 0x9120).unwrap();
        assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
    }

    #[test]
    fn test_op_bnnn_jp_v0() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[0] = 0x05;
        run_instruction(&mut chip8, 0xB200).unwrap();
        assert_eq!(chip8.pc, 0x205, "PC should be V0 + nnn");
    }

    #[test]
    fn test_nested_subroutine_calls() {
        let mut chip8 = Chip8::new().unwrap();
        let initial_pc = chip8.pc;

        // First call
        run_instruction(&mut chip8, 0x2300).unwrap();
        assert_eq!(chip8.pc, 0x300);
        assert_eq!(chip8.sp, 1);
        assert_eq!(chip8.stack[0], initial_pc + 2);

        // Second nested call
        run_instruction(&mut chip8, 0x2400).unwrap();
        assert_eq!(chip8.pc, 0x400);
        assert_eq!(chip8.sp, 2);
        assert_eq!(chip8.stack[1], 0x302);

        // Return from second call
        run_instruction(&mut chip8, 0x00EE).unwrap();
        assert_eq!(chip8.pc, 0x302);
        assert_eq!(chip8.sp, 1);

        // Return from first call
        run_instruction(&mut chip8, 0x00EE).unwrap();
        assert_eq!(chip8.pc, initial_pc + 2);
        assert_eq!(chip8.sp, 0);
    }
}
