use crate::{Chip8, Chip8Error};
use rand::Rng;

impl Chip8 {
    /// 6xkk - LD Vx, byte: Set Vx = kk
    pub fn set_vx_to_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = nn;
        dbg!("set vx to nn");
        Ok(())
    }

    /// 7xkk - ADD Vx, byte: Set Vx = Vx + kk
    pub fn add_nn_to_vx(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = vx.wrapping_add(nn);
        dbg!("add nn to vx");
        Ok(())
    }

    /// 8xy0 - LD Vx, Vy: Set Vx = Vy
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
        dbg!("set vy to vx");
        Ok(())
    }

    /// 8xy1 - OR Vx, Vy: Set Vx = Vx OR Vy
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
        dbg!("binary OR vy to vx");
        Ok(())
    }

    /// 8xy2 - AND Vx, Vy: Set Vx = Vx AND Vy
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
        dbg!("binary AND vy to vx");
        Ok(())
    }

    /// 8xy3 - XOR Vx, Vy: Set Vx = Vx XOR Vy
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
        dbg!("binary XOR vy to vx");
        Ok(())
    }

    /// 8xy4 - ADD Vx, Vy: Set Vx = Vx + Vy, set VF = carry
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
        dbg!("add vy to vx with overflow flag setup");
        Ok(())
    }

    /// 8xy5 - SUB Vx, Vy: Set Vx = Vx - Vy, set VF = NOT borrow
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
        dbg!("sub vy from vx");
        Ok(())
    }

    /// 8xy6 - SHR Vx: Set Vx = Vx SHR 1
    pub fn shift_vx_right(&mut self, x: usize) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        // shift vx one bit to the right, then set vf to the one bit was shifted out
        let shifted_out = *vx & 0x1;
        *vx >>= 1;
        let vf = self
            .registers
            .last_mut()
            .ok_or(Chip8Error::InvalidRegister(0xf))?;
        *vf = shifted_out;
        dbg!("shift vx one bit to the right then set vf to the one bit was shifted out");
        Ok(())
    }

    /// 8xy7 - SUBN Vx, Vy: Set Vx = Vy - Vx, set VF = NOT borrow
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
        dbg!("sub vx from vy then set vx");
        Ok(())
    }

    /// 8xyE - SHL Vx: Set Vx = Vx SHL 1
    pub fn shift_vx_left(&mut self, x: usize) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        // shift vx one bit to the left, then set vf to the one bit was shifted out
        let shifted_out = (*vx >> 7) & 0x1;
        *vx <<= 1;
        let vf = self
            .registers
            .last_mut()
            .ok_or(Chip8Error::InvalidRegister(0xf))?;
        *vf = shifted_out;
        dbg!("shift vx one bit to the left then set vf to the one bit was shifted out");
        Ok(())
    }

    /// Cxkk - RND Vx, byte: Set Vx = random byte AND kk
    pub fn set_vx_to_random_and_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = rand::rng().random_range(0..=255) & nn;
        dbg!("set vx to random number and nn");
        Ok(())
    }
}
