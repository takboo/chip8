//! Input/output operation implementations for CHIP-8.
//!
//! This module contains implementations for keyboard input handling instructions.
//! The CHIP-8 system has a 16-key hexadecimal keypad (0-F) that programs can
//! interact with through these instructions.

use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// **EX9E - SKP Vx**: Skip next instruction if key with value of Vx is pressed.
    ///
    /// This instruction checks if the key corresponding to the value in register Vx
    /// is currently being pressed. If the key is pressed, the program counter is
    /// incremented by 2 (skipping the next instruction).
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) containing the key code to check
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    /// Returns `Chip8Error::InvalidKey` if the key value in Vx is not a valid key (0-15).
    ///
    /// # Side Effects
    ///
    /// May increment the program counter by 2 if the specified key is pressed.
    pub fn skip_if_key_pressed(&mut self, x: usize) -> Result<(), Chip8Error> {
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

        Ok(())
    }

    /// **EXA1 - SKNP Vx**: Skip next instruction if key with value of Vx is not pressed.
    ///
    /// This instruction checks if the key corresponding to the value in register Vx
    /// is currently not being pressed. If the key is not pressed, the program counter
    /// is incremented by 2 (skipping the next instruction).
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) containing the key code to check
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    /// Returns `Chip8Error::InvalidKey` if the key value in Vx is not a valid key (0-15).
    ///
    /// # Side Effects
    ///
    /// May increment the program counter by 2 if the specified key is not pressed.
    pub fn skip_if_key_not_pressed(&mut self, x: usize) -> Result<(), Chip8Error> {
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

        Ok(())
    }

    /// **FX0A - LD Vx, K**: Wait for a key press and store the key value in Vx.
    ///
    /// This instruction pauses program execution until a key is pressed. Once a key
    /// is pressed, its hexadecimal value (0-F) is stored in register Vx and execution
    /// continues. If no key is pressed, the instruction repeats by decrementing the
    /// program counter.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15) where the pressed key value will be stored
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// - Stores the pressed key value (0-15) in register Vx when a key is pressed
    /// - Decrements the program counter by 2 if no key is pressed (causing the instruction to repeat)
    ///
    /// # Behavior
    ///
    /// This instruction implements a blocking wait - the program will not continue
    /// until a key is actually pressed. The first key found to be pressed will be
    /// used if multiple keys are pressed simultaneously.
    pub fn wait_for_key_press(&mut self, x: usize) -> Result<(), Chip8Error> {
        // Check all keys to find the first one that is pressed
        let mut key_pressed = false;
        for (i, &key) in self.keyboard.iter().enumerate() {
            if key != 0 {
                let vx = self
                    .registers
                    .get_mut(x)
                    .ok_or(Chip8Error::InvalidRegister(x))?;
                *vx = i as u8;
                key_pressed = true;
                break;
            }
        }

        if !key_pressed {
            // No key pressed - repeat this instruction by moving PC back
            self.pc = self.pc.wrapping_sub(2);
        }
        Ok(())
    }
}
