//! Memory operation implementations for CHIP-8.
//!
//! This module contains implementations for all memory-related instructions,
//! including index register operations, timer management, font handling,
//! and bulk memory operations. These instructions provide the core memory
//! management capabilities of the CHIP-8 virtual machine.

use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// **ANNN - LD I, addr**: Set index register I to address NNN.
    ///
    /// This instruction loads a 12-bit address into the index register I.
    /// The index register is commonly used to point to sprite data, font characters,
    /// or other memory locations for subsequent operations.
    ///
    /// # Arguments
    ///
    /// * `nnn` - 12-bit address to load into I (0x000-0xFFF)
    ///
    /// # Errors
    ///
    /// This instruction should not fail under normal circumstances.
    ///
    /// # Side Effects
    ///
    /// Sets the index register I to the specified address.
    pub(super) fn set_i_to_nnn(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        self.i = nnn;
        Ok(())
    }

    /// **FX07 - LD Vx, DT**: Load delay timer value into register Vx.
    ///
    /// This instruction reads the current value of the delay timer and stores
    /// it in register Vx. This is useful for timing operations and delays.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) to store the timer value
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Loads the current delay timer value into register Vx.
    pub(super) fn set_vx_to_delay_timer(&mut self, x: usize) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = self.dt;
        Ok(())
    }

    /// **FX15 - LD DT, Vx**: Set delay timer to the value in register Vx.
    ///
    /// This instruction sets the delay timer to the value stored in register Vx.
    /// The delay timer decrements at 60Hz until it reaches zero.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) containing the timer value
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets the delay timer to the value in register Vx.
    pub(super) fn set_delay_timer_to_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        self.dt = vx;
        Ok(())
    }

    /// **FX18 - LD ST, Vx**: Set sound timer to the value in register Vx.
    ///
    /// This instruction sets the sound timer to the value stored in register Vx.
    /// The sound timer decrements at 60Hz, and while it's non-zero, the system
    /// should produce a beep sound.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) containing the timer value
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets the sound timer to the value in register Vx.
    pub(super) fn set_sound_timer_to_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        self.st = vx;
        Ok(())
    }

    /// **FX1E - ADD I, Vx**: Add register Vx to index register I.
    ///
    /// This instruction adds the value in register Vx to the index register I.
    /// The addition wraps around on overflow. This is commonly used to advance
    /// the index register when processing arrays or sprite data.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) containing the value to add to I
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Adds the value in register Vx to the index register I (with wrapping).
    pub(super) fn add_vx_to_i(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        self.i = self.i.wrapping_add(vx as u16);
        Ok(())
    }

    /// **FX29 - LD F, Vx**: Set I to the location of the font sprite for digit Vx.
    ///
    /// This instruction sets the index register I to the memory address of the
    /// font sprite for the hexadecimal digit stored in register Vx. Each font
    /// character is 5 bytes tall and 4 pixels wide.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) containing the digit (0-F)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets the index register I to point to the font data for the specified digit.
    ///
    /// # Note
    ///
    /// Only the lower 4 bits of Vx are used (values 0-F). Higher values will
    /// wrap around modulo 16.
    pub(super) fn set_i_to_font_location(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        // Each font character is 5 bytes, font starts at FONT_START_ADDRESS
        self.i = crate::memory::FONT_START_ADDRESS as u16 + (vx as u16 * 5);
        Ok(())
    }

    /// **FX33 - LD B, Vx**: Store BCD representation of Vx in memory.
    ///
    /// This instruction takes the decimal value in register Vx and stores its
    /// Binary-Coded Decimal (BCD) representation in memory at locations I, I+1, and I+2:
    /// - I: hundreds digit
    /// - I+1: tens digit
    /// - I+2: ones digit
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) containing the value to convert
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    /// Returns `Chip8Error::IndexError` if the memory locations I, I+1, I+2 are invalid.
    ///
    /// # Side Effects
    ///
    /// Modifies 3 bytes of memory starting at address I with the BCD representation.
    ///
    /// # Examples
    ///
    /// If Vx contains 234:
    /// - Memory\[I\] = 2 (hundreds)
    /// - Memory\[I+1\] = 3 (tens)
    /// - Memory\[I+2\] = 4 (ones)
    pub(super) fn store_bcd_of_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        let slice: [u8; 3] = [vx / 100, (vx % 100) / 10, vx % 10];
        self.memory.write_at(&slice, self.i as usize)?;
        Ok(())
    }

    /// **FX55 - LD \[I\], Vx**: Store registers V0 through Vx in memory starting at location I.
    ///
    /// This instruction copies the values from registers V0 through Vx (inclusive)
    /// into memory starting at the address stored in the index register I.
    /// After the operation, I is left unchanged.
    ///
    /// # Arguments
    ///
    /// * `x` - Highest register index to store (0-15). Stores V0 through Vx inclusive.
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    /// Returns `Chip8Error::IndexError` if the memory range starting at I is invalid.
    ///
    /// # Side Effects
    ///
    /// Copies (x+1) register values into consecutive memory locations starting at I.
    ///
    /// # Examples
    ///
    /// If x=3, this instruction stores V0, V1, V2, and V3 into memory locations
    /// I, I+1, I+2, and I+3 respectively.
    pub(super) fn store_registers_to_memory(&mut self, x: usize) -> Result<(), Chip8Error> {
        let buf = self
            .registers
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if i <= x { Some(*v) } else { None })
            .collect::<Vec<u8>>();

        self.memory.write_at(&buf, self.i as usize)?;
        Ok(())
    }

    /// **FX65 - LD Vx, \[I\]**: Load registers V0 through Vx from memory starting at location I.
    ///
    /// This instruction copies values from memory starting at the address stored
    /// in the index register I into registers V0 through Vx (inclusive).
    /// After the operation, I is left unchanged.
    ///
    /// # Arguments
    ///
    /// * `x` - Highest register index to load (0-15). Loads V0 through Vx inclusive.
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    /// Returns `Chip8Error::IndexError` if the memory range starting at I is invalid.
    ///
    /// # Side Effects
    ///
    /// Loads (x+1) values from consecutive memory locations starting at I into registers.
    ///
    /// # Examples
    ///
    /// If x=3, this instruction loads memory locations I, I+1, I+2, and I+3
    /// into registers V0, V1, V2, and V3 respectively.
    pub(super) fn load_registers_from_memory(&mut self, x: usize) -> Result<(), Chip8Error> {
        let memory = self
            .memory
            .get(self.i as usize..=self.i as usize + x)
            .ok_or(Chip8Error::IndexError(self.i))?;

        for (i, register) in self.registers.iter_mut().enumerate() {
            if i > x {
                break;
            }
            *register = memory[i];
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{tests::run_instruction, *};

    #[test]
    fn test_op_annn_ld_i() {
        let mut chip8 = Chip8::new().unwrap();
        run_instruction(&mut chip8, 0xA123).unwrap();
        assert_eq!(chip8.i, 0x123);
    }

    #[test]
    fn test_op_fx07_ld_vx_dt() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.dt = 42;
        run_instruction(&mut chip8, 0xF107).unwrap();
        assert_eq!(chip8.registers[1], 42);
    }

    #[test]
    fn test_op_fx15_ld_dt_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[2] = 100;
        run_instruction(&mut chip8, 0xF215).unwrap();
        assert_eq!(chip8.dt, 100);
    }

    #[test]
    fn test_op_fx18_ld_st_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[3] = 200;
        run_instruction(&mut chip8, 0xF318).unwrap();
        assert_eq!(chip8.st, 200);
    }

    #[test]
    fn test_op_fx1e_add_i_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x100;
        chip8.registers[4] = 0x50;
        run_instruction(&mut chip8, 0xF41E).unwrap();
        assert_eq!(chip8.i, 0x150);
    }

    #[test]
    fn test_op_fx1e_add_i_vx_overflow() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0xFFF0;
        chip8.registers[4] = 0x20;
        run_instruction(&mut chip8, 0xF41E).unwrap();
        assert_eq!(chip8.i, 0x10); // Should wrap around
    }

    #[test]
    fn test_op_fx29_ld_f_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0xA; // Digit A
        run_instruction(&mut chip8, 0xF129).unwrap();
        // Font for digit A should be at FONT_START_ADDRESS + (0xA * 5)
        let expected_address = crate::memory::FONT_START_ADDRESS as u16 + (0xA * 5);
        assert_eq!(chip8.i, expected_address);
    }

    #[test]
    fn test_op_fx29_ld_f_vx_all_digits() {
        let mut chip8 = Chip8::new().unwrap();
        for digit in 0..=0xF {
            chip8.registers[1] = digit;
            run_instruction(&mut chip8, 0xF129).unwrap();
            let expected_address = crate::memory::FONT_START_ADDRESS as u16 + (digit as u16 * 5);
            assert_eq!(chip8.i, expected_address);
            chip8.reset().unwrap();
        }
    }

    #[test]
    fn test_op_fx33_ld_b_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[0] = 123;
        chip8.i = 0x300;
        run_instruction(&mut chip8, 0xF033).unwrap();
        assert_eq!(chip8.memory.read_byte(0x300), Some(1));
        assert_eq!(chip8.memory.read_byte(0x301), Some(2));
        assert_eq!(chip8.memory.read_byte(0x302), Some(3));
    }

    #[test]
    fn test_op_fx33_ld_b_vx_single_digit() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[0] = 7;
        chip8.i = 0x300;
        run_instruction(&mut chip8, 0xF033).unwrap();
        assert_eq!(chip8.memory.read_byte(0x300), Some(0));
        assert_eq!(chip8.memory.read_byte(0x301), Some(0));
        assert_eq!(chip8.memory.read_byte(0x302), Some(7));
    }

    #[test]
    fn test_op_fx33_ld_b_vx_max_value() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[0] = 255;
        chip8.i = 0x300;
        run_instruction(&mut chip8, 0xF033).unwrap();
        assert_eq!(chip8.memory.read_byte(0x300), Some(2));
        assert_eq!(chip8.memory.read_byte(0x301), Some(5));
        assert_eq!(chip8.memory.read_byte(0x302), Some(5));
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
            assert_eq!(chip8.memory.read_byte(0x300 + i), Some(i as u8));
        }
    }

    #[test]
    fn test_op_fx55_ld_i_vx_partial() {
        let mut chip8 = Chip8::new().unwrap();
        for i in 0..16 {
            chip8.registers[i] = i as u8 + 10;
        }
        chip8.i = 0x300;
        run_instruction(&mut chip8, 0xF255).unwrap(); // Only store V0-V2

        // Check stored registers
        for i in 0..=2 {
            assert_eq!(chip8.memory.read_byte(0x300 + i), Some(i as u8 + 10));
        }

        // Check that other memory locations weren't modified
        assert_eq!(chip8.memory.read_byte(0x303), Some(0));
    }

    #[test]
    fn test_op_fx65_ld_vx_i() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        let value = (0..=5).collect::<Vec<u8>>();
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");

        run_instruction(&mut chip8, 0xF565).unwrap();
        for i in 0..=5 {
            assert_eq!(chip8.registers[i], i as u8);
        }
    }

    #[test]
    fn test_timer_operations() {
        let mut chip8 = Chip8::new().unwrap();

        // Set delay timer
        chip8.registers[1] = 60;
        run_instruction(&mut chip8, 0xF115).unwrap();
        assert_eq!(chip8.dt, 60);

        // Read delay timer
        chip8.pc = 0x200; // Reset PC
        run_instruction(&mut chip8, 0xF207).unwrap();
        assert_eq!(chip8.registers[2], 60);

        // Set sound timer
        chip8.pc = 0x200; // Reset PC
        chip8.registers[3] = 30;
        run_instruction(&mut chip8, 0xF318).unwrap();
        assert_eq!(chip8.st, 30);
    }

    #[test]
    fn test_memory_boundary_conditions() {
        let mut chip8 = Chip8::new().unwrap();

        // Test near memory boundary
        chip8.i = 4093; // Near end of memory
        chip8.registers[0] = 123;
        run_instruction(&mut chip8, 0xF033).unwrap();
        assert_eq!(chip8.memory.read_byte(4093), Some(1));
        assert_eq!(chip8.memory.read_byte(4094), Some(2));
        assert_eq!(chip8.memory.read_byte(4095), Some(3));
    }

    #[test]
    fn test_index_register_overflow_boundary() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0xFFFF;
        chip8.registers[1] = 1;
        run_instruction(&mut chip8, 0xF11E).unwrap();
        assert_eq!(chip8.i, 0); // Should wrap to 0
    }
}
