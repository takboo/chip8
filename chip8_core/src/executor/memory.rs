use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// Annn - LD I, addr: Set I = nnn
    pub fn set_i_to_nnn(&mut self, nnn: u16) -> Result<(), Chip8Error> {
        self.i = nnn;
        dbg!("set i to nnn");
        Ok(())
    }

    /// Fx07 - LD Vx, DT: Set Vx = delay timer value
    pub fn set_vx_to_delay_timer(&mut self, x: usize) -> Result<(), Chip8Error> {
        let vx = self
            .registers
            .get_mut(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        *vx = self.dt;
        dbg!("set vx to dt");
        Ok(())
    }

    /// Fx15 - LD DT, Vx: Set delay timer = Vx
    pub fn set_delay_timer_to_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        self.dt = vx;
        dbg!("set dt to vx");
        Ok(())
    }

    /// Fx18 - LD ST, Vx: Set sound timer = Vx
    pub fn set_sound_timer_to_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        self.st = vx;
        dbg!("set st to vx");
        Ok(())
    }

    /// Fx1E - ADD I, Vx: Set I = I + Vx
    pub fn add_vx_to_i(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        self.i = self.i.wrapping_add(vx as u16);
        dbg!("add vx to i");
        Ok(())
    }

    /// Fx29 - LD F, Vx: Set I = location of sprite for digit Vx
    pub fn set_i_to_font_location(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        // Each font character is 5 bytes, font starts at FONT_START_ADDRESS
        self.i = crate::consts::FONT_START_ADDRESS as u16 + (vx as u16 * 5);
        dbg!("set i to font location for digit vx");
        Ok(())
    }

    /// Fx33 - LD B, Vx: Store BCD representation of Vx in memory locations I, I+1, and I+2
    pub fn store_bcd_of_vx(&mut self, x: usize) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        let memory = self
            .memory
            .get_mut(self.i as usize..self.i as usize + 3)
            .ok_or(Chip8Error::IndexError(self.i))?;
        memory[0] = vx / 100;
        memory[1] = (vx % 100) / 10;
        memory[2] = vx % 10;
        dbg!(
            "convert vx to three decimal digits and store in memory at the address in the index register I"
        );
        Ok(())
    }

    /// Fx55 - LD [I], Vx: Store registers V0 through Vx in memory starting at location I
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
        dbg!("copy registers to memory at the address in the index register I");
        Ok(())
    }

    /// Fx65 - LD Vx, [I]: Read registers V0 through Vx from memory starting at location I
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
        dbg!("copy memory to registers at the address in the index register I");
        Ok(())
    }
}
