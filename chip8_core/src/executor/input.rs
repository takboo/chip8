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
    pub(super) fn skip_if_key_pressed(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    pub(super) fn skip_if_key_not_pressed(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    pub(super) fn wait_for_key_press(&mut self, x: usize) -> Result<(), Chip8Error> {
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

#[cfg(test)]
mod tests {
    use crate::{tests::run_instruction, *};

    #[test]
    fn test_op_ex9e_skp_key_pressed() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 5; // Key index
        chip8.key_press(5); // Press key 5
        let initial_pc = chip8.pc;

        run_instruction(&mut chip8, 0xE19E).unwrap();
        assert_eq!(
            chip8.pc,
            initial_pc + 4,
            "PC should skip next instruction when key is pressed"
        );
    }

    #[test]
    fn test_op_ex9e_skp_key_not_pressed() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 5; // Key index
        // Don't press any key
        let initial_pc = chip8.pc;

        run_instruction(&mut chip8, 0xE19E).unwrap();
        assert_eq!(
            chip8.pc,
            initial_pc + 2,
            "PC should not skip when key is not pressed"
        );
    }

    #[test]
    fn test_op_exa1_sknp_key_not_pressed() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 5; // Key index
        // Don't press any key
        let initial_pc = chip8.pc;

        run_instruction(&mut chip8, 0xE1A1).unwrap();
        assert_eq!(
            chip8.pc,
            initial_pc + 4,
            "PC should skip next instruction when key is not pressed"
        );
    }

    #[test]
    fn test_op_exa1_sknp_key_pressed() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 5; // Key index
        chip8.key_press(5); // Press key 5
        let initial_pc = chip8.pc;

        run_instruction(&mut chip8, 0xE1A1).unwrap();
        assert_eq!(
            chip8.pc,
            initial_pc + 2,
            "PC should not skip when key is pressed"
        );
    }

    #[test]
    fn test_op_fx0a_ld_vx_k_wait() {
        let mut chip8 = Chip8::new().unwrap();
        let initial_pc = chip8.pc;
        // Run without a key press
        run_instruction(&mut chip8, 0xF30A).unwrap();
        // PC should be rewound, effectively pausing execution
        assert_eq!(chip8.pc, initial_pc);
    }

    #[test]
    fn test_op_fx0a_ld_vx_k_press() {
        let mut chip8 = Chip8::new().unwrap();
        let initial_pc = chip8.pc;
        // Simulate key press for key 0xA
        chip8.key_press(0xA);
        run_instruction(&mut chip8, 0xF30A).unwrap();
        // Register V3 should contain 0xA
        assert_eq!(chip8.registers[3], 0xA);
        // PC should advance normally
        assert_eq!(chip8.pc, initial_pc + 2);
    }

    #[test]
    fn test_key_press_release_cycle() {
        let mut chip8 = Chip8::new().unwrap();

        // Initially no keys pressed
        for i in 0..16 {
            assert_eq!(chip8.keyboard[i], 0);
        }

        // Press key 5
        chip8.key_press(5);
        assert_eq!(chip8.keyboard[5], 1);

        // Release key 5
        chip8.key_release(5);
        assert_eq!(chip8.keyboard[5], 0);
    }

    #[test]
    fn test_multiple_keys_pressed() {
        let mut chip8 = Chip8::new().unwrap();

        // Press multiple keys
        chip8.key_press(0);
        chip8.key_press(5);
        chip8.key_press(15);

        assert_eq!(chip8.keyboard[0], 1);
        assert_eq!(chip8.keyboard[5], 1);
        assert_eq!(chip8.keyboard[15], 1);

        // Other keys should still be unpressed
        assert_eq!(chip8.keyboard[1], 0);
        assert_eq!(chip8.keyboard[7], 0);
    }

    #[test]
    fn test_key_input_invalid_index() {
        let mut chip8 = Chip8::new().unwrap();

        // These should not panic or cause errors
        chip8.key_press(16); // Invalid key
        chip8.key_press(255); // Invalid key
        chip8.key_release(20); // Invalid key

        // All valid keys should still be unpressed
        for i in 0..16 {
            assert_eq!(chip8.keyboard[i], 0);
        }
    }

    #[test]
    fn test_key_detection_priority() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.key_press(0);
        chip8.key_press(5);
        chip8.key_press(10);

        let initial_pc = chip8.pc;
        run_instruction(&mut chip8, 0xF10A).unwrap(); // Wait for key

        // Should detect the first pressed key (lowest index)
        assert_eq!(chip8.registers[1], 0);
        assert_eq!(chip8.pc, initial_pc + 2);
    }

    #[test]
    fn test_key_instruction_with_invalid_key_register() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 16; // Invalid key index

        // This should return an error
        let result = run_instruction(&mut chip8, 0xE19E);
        assert!(matches!(result, Err(Chip8Error::InvalidKey(16))));
    }
}
