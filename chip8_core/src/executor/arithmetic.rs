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
    ///
    /// # Examples
    ///
    /// ```
    /// # use chip8_core::Chip8;
    /// let mut chip8 = Chip8::new().unwrap();
    /// chip8.set_vx_to_nn(0, 0x42).unwrap();
    /// // V0 is now 0x42
    /// ```
    pub fn set_vx_to_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use chip8_core::Chip8;
    /// let mut chip8 = Chip8::new().unwrap();
    /// chip8.set_vx_to_nn(0, 10).unwrap();
    /// chip8.add_nn_to_vx(0, 5).unwrap();
    /// // V0 is now 15
    /// ```
    pub fn add_nn_to_vx(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
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
    pub fn set_vx_to_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
    pub fn or_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
    pub fn and_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
    pub fn xor_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
    pub fn add_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
    pub fn sub_vx_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
    pub fn shift_vx_right(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    pub fn sub_vy_vx(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
    pub fn shift_vx_left(&mut self, x: usize) -> Result<(), Chip8Error> {
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
    ///
    /// # Examples
    ///
    /// Using `nn = 0x0F` will generate a random number between 0-15.
    /// Using `nn = 0xFF` will generate a random number between 0-255.
    pub fn set_vx_to_random_and_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = rand::rng().random_range(0..=255) & nn;
        Ok(())
    }
}
