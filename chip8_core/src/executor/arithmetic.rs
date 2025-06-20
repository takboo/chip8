//! Arithmetic and logical operation implementations for CHIP-8 instructions.
//!
//! This module contains implementations for all arithmetic, logical, and mathematical
//! operations that can be performed on CHIP-8 registers. These operations form the
//! core computational capabilities of the CHIP-8 virtual machine.

use crate::{Chip8, Chip8Error};
use rand::Rng;

impl Chip8 {
    /// **6XNN - LD Vx, byte**: Set register Vx to the immediate value NN.
    ///
    /// This instruction loads an 8-bit constant into register Vx.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15)
    /// * `nn` - 8-bit immediate value to load
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    pub(super) fn set_vx_to_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = nn;
        Ok(())
    }

    /// **7XNN - ADD Vx, byte**: Add immediate value NN to register Vx.
    ///
    /// This instruction adds an 8-bit constant to register Vx. The addition
    /// wraps around on overflow (no carry flag is set).
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15)
    /// * `nn` - 8-bit immediate value to add
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    pub(super) fn add_nn_to_vx(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = vx.wrapping_add(nn);
        Ok(())
    }

    /// **8XY0 - LD Vx, Vy**: Copy the value of register Vy into register Vx.
    ///
    /// This instruction performs a simple register-to-register copy operation.
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `y` - Source register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    pub(super) fn set_vx_to_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        let &vy = self
            .registers
            .get(y)
            .ok_or(Chip8Error::InvalidRegister(y))?;
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = vy;
        Ok(())
    }

    /// **8XY1 - OR Vx, Vy**: Perform bitwise OR operation between Vx and Vy, store result in Vx.
    ///
    /// This instruction performs a logical OR operation on each bit of the two registers.
    /// The result is stored in register Vx.
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `y` - Source register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    pub(super) fn or_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        let &vy = self
            .registers
            .get(y)
            .ok_or(Chip8Error::InvalidRegister(y))?;
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx |= vy;
        Ok(())
    }

    /// **8XY2 - AND Vx, Vy**: Perform bitwise AND operation between Vx and Vy, store result in Vx.
    ///
    /// This instruction performs a logical AND operation on each bit of the two registers.
    /// The result is stored in register Vx.
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `y` - Source register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    pub(super) fn and_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        let &vy = self
            .registers
            .get(y)
            .ok_or(Chip8Error::InvalidRegister(y))?;
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx &= vy;
        Ok(())
    }

    /// **8XY3 - XOR Vx, Vy**: Perform bitwise XOR operation between Vx and Vy, store result in Vx.
    ///
    /// This instruction performs a logical exclusive OR operation on each bit of the two registers.
    /// The result is stored in register Vx.
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `y` - Source register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    pub(super) fn xor_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        let &vy = self
            .registers
            .get(y)
            .ok_or(Chip8Error::InvalidRegister(y))?;
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx ^= vy;
        Ok(())
    }

    /// **8XY4 - ADD Vx, Vy**: Add Vy to Vx, set VF to carry flag.
    ///
    /// This instruction adds the values in registers Vx and Vy. If the result
    /// overflows beyond 255, VF is set to 1, otherwise VF is set to 0.
    /// The result is stored in Vx (only the lower 8 bits).
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `y` - Source register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets VF register to 1 if carry occurs, 0 otherwise.
    pub(super) fn add_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
        Ok(())
    }

    /// **8XY5 - SUB Vx, Vy**: Subtract Vy from Vx, set VF to NOT borrow flag.
    ///
    /// This instruction subtracts Vy from Vx. If Vx > Vy, then VF is set to 1,
    /// otherwise VF is set to 0. The result is stored in Vx.
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `y` - Source register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets VF register to 1 if no borrow occurs (Vx >= Vy), 0 if borrow occurs.
    pub(super) fn sub_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
        Ok(())
    }

    /// **8XY6 - SHR Vx**: Shift Vx right by one bit, set VF to the shifted-out bit.
    ///
    /// This instruction shifts the value in register Vx one bit to the right.
    /// The least significant bit (LSB) before the shift is stored in VF.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets VF register to the value of the LSB before the shift operation.
    pub(super) fn shift_vx_right(&mut self, x: usize) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        // Store the bit that will be shifted out
        let shifted_out = *vx & 0x1;
        *vx >>= 1;
        let vf = self
            .registers
            .last_mut()
            .ok_or(Chip8Error::InvalidRegister(0xf))?;
        *vf = shifted_out;
        Ok(())
    }

    /// **8XY7 - SUBN Vx, Vy**: Subtract Vx from Vy, store result in Vx, set VF to NOT borrow flag.
    ///
    /// This instruction subtracts Vx from Vy and stores the result in Vx.
    /// If Vy > Vx, then VF is set to 1, otherwise VF is set to 0.
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `y` - Source register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if either register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets VF register to 1 if no borrow occurs (Vy >= Vx), 0 if borrow occurs.
    pub(super) fn sub_vy_vx(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
        Ok(())
    }

    /// **8XYE - SHL Vx**: Shift Vx left by one bit, set VF to the shifted-out bit.
    ///
    /// This instruction shifts the value in register Vx one bit to the left.
    /// The most significant bit (MSB) before the shift is stored in VF.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index (0-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    ///
    /// # Side Effects
    ///
    /// Sets VF register to the value of the MSB before the shift operation.
    pub(super) fn shift_vx_left(&mut self, x: usize) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        // Store the bit that will be shifted out (MSB)
        let shifted_out = (*vx >> 7) & 0x1;
        *vx <<= 1;
        let vf = self
            .registers
            .last_mut()
            .ok_or(Chip8Error::InvalidRegister(0xf))?;
        *vf = shifted_out;
        Ok(())
    }

    /// **CXNN - RND Vx, byte**: Generate random number, AND with NN, store in Vx.
    ///
    /// This instruction generates a random 8-bit number, performs a bitwise AND
    /// operation with the immediate value NN, and stores the result in register Vx.
    /// This is commonly used for random number generation with a specific range or mask.
    ///
    /// # Arguments
    ///
    /// * `x` - Destination register index (0-15)
    /// * `nn` - 8-bit mask value to AND with the random number
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if the register index is out of bounds.
    pub(super) fn set_vx_to_random_and_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = rand::rng().random_range(0..=255) & nn;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{tests::run_instruction, *};

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
    fn test_op_7xkk_add_vx_byte_overflow() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[5] = 0xFF;
        run_instruction(&mut chip8, 0x7501).unwrap();
        assert_eq!(chip8.registers[5], 0); // Should wrap around
    }

    #[test]
    fn test_op_8xy0_ld_vx_vy() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[2] = 0x42;
        run_instruction(&mut chip8, 0x8120).unwrap();
        assert_eq!(chip8.registers[1], 0x42);
    }

    #[test]
    fn test_op_8xy1_or_vx_vy() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0b11001100;
        chip8.registers[2] = 0b10101010;
        run_instruction(&mut chip8, 0x8121).unwrap();
        assert_eq!(chip8.registers[1], 0b11101110);
    }

    #[test]
    fn test_op_8xy2_and_vx_vy() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0b11001100;
        chip8.registers[2] = 0b10101010;
        run_instruction(&mut chip8, 0x8122).unwrap();
        assert_eq!(chip8.registers[1], 0b10001000);
    }

    #[test]
    fn test_op_8xy3_xor_vx_vy() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0b11001100;
        chip8.registers[2] = 0b10101010;
        run_instruction(&mut chip8, 0x8123).unwrap();
        assert_eq!(chip8.registers[1], 0b01100110);
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
    fn test_op_8xy5_sub_vx_vy_no_borrow() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 30;
        chip8.registers[2] = 10;
        run_instruction(&mut chip8, 0x8125).unwrap();
        assert_eq!(chip8.registers[1], 20);
        assert_eq!(chip8.registers[0xF], 1, "VF should be 1 for no borrow");
    }

    #[test]
    fn test_op_8xy5_sub_vx_vy_with_borrow() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 10;
        chip8.registers[2] = 30;
        run_instruction(&mut chip8, 0x8125).unwrap();
        assert_eq!(chip8.registers[1], 236); // 256 - 20
        assert_eq!(chip8.registers[0xF], 0, "VF should be 0 for borrow");
    }

    #[test]
    fn test_op_8xy6_shr_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0b10101011;
        run_instruction(&mut chip8, 0x8126).unwrap();
        assert_eq!(chip8.registers[1], 0b01010101);
        assert_eq!(chip8.registers[0xF], 1, "VF should contain shifted out bit");
    }

    #[test]
    fn test_op_8xy7_subn_vx_vy() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 10;
        chip8.registers[2] = 30;
        run_instruction(&mut chip8, 0x8127).unwrap();
        assert_eq!(chip8.registers[1], 20);
        assert_eq!(chip8.registers[0xF], 1, "VF should be 1 for no borrow");
    }

    #[test]
    fn test_op_8xye_shl_vx() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.registers[1] = 0b10101010;
        run_instruction(&mut chip8, 0x812E).unwrap();
        assert_eq!(chip8.registers[1], 0b01010100);
        assert_eq!(chip8.registers[0xF], 1, "VF should contain shifted out bit");
    }

    #[test]
    fn test_op_cxkk_rnd_vx() {
        let mut chip8 = Chip8::new().unwrap();
        // Run multiple times to ensure randomness
        let mut results = Vec::new();
        for _ in 0..10 {
            chip8.registers[1] = 0;
            run_instruction(&mut chip8, 0xC1FF).unwrap();
            results.push(chip8.registers[1]);
            chip8.reset().unwrap();
        }

        // Check that we got some variation
        let unique_count = results
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert!(
            unique_count > 1,
            "Random number generator should produce different values"
        );
    }

    #[test]
    fn test_op_cxkk_rnd_vx_mask() {
        let mut chip8 = Chip8::new().unwrap();
        // Test with mask 0x0F (only lower 4 bits)
        for _ in 0..10 {
            chip8.registers[1] = 0;
            run_instruction(&mut chip8, 0xC10F).unwrap();
            assert!(
                chip8.registers[1] <= 0x0F,
                "Random value should be masked to 4 bits"
            );
            chip8.reset().unwrap();
        }
    }
}
