use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// Ex9E - SKP Vx: Skip next instruction if key with the value of Vx is pressed
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
        dbg!("skip next instruction if key is pressed");
        Ok(())
    }

    /// ExA1 - SKNP Vx: Skip next instruction if key with the value of Vx is not pressed
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
        dbg!("skip next instruction if key is not pressed");
        Ok(())
    }

    /// Fx0A - LD Vx, K: Wait for a key press, store the value of the key in Vx
    pub fn wait_for_key_press(&mut self, x: usize) -> Result<(), Chip8Error> {
        // wait for a keypress, store the value of the key in vx
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
            // Re-run this instruction
            self.pc = self.pc.wrapping_sub(2);
        }
        dbg!("wait for a keypress, store the value of the key in vx");
        Ok(())
    }
}
