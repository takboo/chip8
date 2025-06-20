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
    pub fn set_i_to_nnn(&mut self, nnn: u16) -> Result<(), Chip8Error> {
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
    pub fn set_vx_to_delay_timer(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    pub fn set_delay_timer_to_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    pub fn set_sound_timer_to_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    pub fn add_vx_to_i(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    pub fn set_i_to_font_location(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        // Each font character is 5 bytes, font starts at FONT_START_ADDRESS
        self.i = crate::consts::FONT_START_ADDRESS as u16 + (vx as u16 * 5);
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
    /// - Memory[I] = 2 (hundreds)
    /// - Memory[I+1] = 3 (tens)
    /// - Memory[I+2] = 4 (ones)
    pub fn store_bcd_of_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        let memory = self
            .memory
            .get_mut(self.i as usize..self.i as usize + 3)
            .ok_or(Chip8Error::IndexError(self.i))?;
        memory[0] = vx / 100; // Hundreds digit
        memory[1] = (vx % 100) / 10; // Tens digit
        memory[2] = vx % 10; // Ones digit
        Ok(())
    }

    /// **FX55 - LD [I], Vx**: Store registers V0 through Vx in memory starting at location I.
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
    pub fn store_registers_to_memory(&mut self, x: usize) -> Result<(), Chip8Error> {
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
        Ok(())
    }

    /// **FX65 - LD Vx, [I]**: Load registers V0 through Vx from memory starting at location I.
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
    pub fn load_registers_from_memory(&mut self, x: usize) -> Result<(), Chip8Error> {
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
        Ok(())
    }
}
