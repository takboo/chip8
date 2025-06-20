use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// Dxyn - DRW Vx, Vy, nibble: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    pub fn draw_sprite(&mut self, x: usize, y: usize, n: u8) -> Result<(), Chip8Error> {
        let &vx = self
            .registers
            .get(x)
            .ok_or(Chip8Error::InvalidRegister(x))?;
        let &vy = self
            .registers
            .get(y)
            .ok_or(Chip8Error::InvalidRegister(y))?;

        let x_coord = (vx % 64) as usize;
        let y_coord = (vy % 32) as usize;
        let height = n as usize;

        let vf = self
            .registers
            .last_mut()
            .ok_or(Chip8Error::InvalidRegister(0xf))?;
        *vf = 0;

        for row in 0..height {
            let y_pos = y_coord + row;
            if y_pos >= 32 {
                break;
            }

            let sprite_byte = self
                .memory
                .get(self.i as usize + row)
                .ok_or(Chip8Error::IndexError(self.i + row as u16))?;

            for col in 0..8 {
                let x_pos = x_coord + col;
                if x_pos >= 64 {
                    continue;
                }

                if (sprite_byte & (0x80 >> col)) != 0 {
                    let pixel_index = y_pos * 64 + x_pos;
                    let pixel = self
                        .framebuffer
                        .get_mut(pixel_index)
                        .ok_or(Chip8Error::FrameBufferOverflow(pixel_index))?;
                    if *pixel == 1 {
                        *vf = 1; // Collision
                    }
                    *pixel ^= 1;
                }
            }
        }
        self.display_updated = true;
        dbg!("draw sprite at vx, vy, n");
        Ok(())
    }
}
