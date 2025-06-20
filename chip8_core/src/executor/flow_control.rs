use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// 00E0 - CLS: Clear the display
    pub fn clear_screen(&mut self) -> Result<(), Chip8Error> {
        self.framebuffer.iter_mut().for_each(|p| *p = 0);
        self.display_updated = true;
        dbg!("clear screen");
        Ok(())
    }

    /// 00EE - RET: Return from a subroutine
    pub fn return_from_subroutine(&mut self) -> Result<(), Chip8Error> {
        self.pop_stack()?;
        dbg!("return, pop pc from stack");
        Ok(())
    }

    /// 1nnn - JP addr: Jump to location nnn
    pub fn jump_to_address(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        self.pc = nnn;
        dbg!("jump to nnn");
        Ok(())
    }

    /// 2nnn - CALL addr: Call subroutine at nnn
    pub fn call_subroutine(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        self.push_stack()?;
        self.pc = nnn;
        dbg!("call a subroutine, should push pc to stack");
        Ok(())
    }

    /// 3xkk - SE Vx, byte: Skip next instruction if Vx = kk
    pub fn skip_if_vx_equals_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        if vx == nn {
            self.pc = self.pc.wrapping_add(2);
        }
        dbg!("condition check vx equal nn");
        Ok(())
    }

    /// 4xkk - SNE Vx, byte: Skip next instruction if Vx != kk
    pub fn skip_if_vx_not_equals_nn(&mut self, x: usize, nn: u8) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        if vx != nn {
            self.pc = self.pc.wrapping_add(2);
        }
        dbg!("condition check vx not equal nn");
        Ok(())
    }

    /// 5xy0 - SE Vx, Vy: Skip next instruction if Vx = Vy
    pub fn skip_if_vx_equals_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
        dbg!("condition check vy equal vx");
        Ok(())
    }

    /// 9xy0 - SNE Vx, Vy: Skip next instruction if Vx != Vy
    pub fn skip_if_vx_not_equals_vy(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
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
        dbg!("condition check vy not equal vx");
        Ok(())
    }

    /// Bnnn - JP V0, addr: Jump to location nnn + V0
    pub fn jump_to_v0_plus_nnn(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        let &v0 = self
            .registers
            .first()
            .ok_or(Chip8Error::InvalidRegister(0x0))?;
        self.pc = nnn.wrapping_add(v0 as u16);
        dbg!("set pc to v0 + nnn");
        Ok(())
    }
}
