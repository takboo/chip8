//! Display operation implementations for CHIP-8.
//!
//! This module contains implementations for all display-related instructions,
//! including screen clearing and sprite drawing. The CHIP-8 display is a 64x32
//! monochrome screen where sprites are drawn using XOR operations.

use crate::{Chip8, Chip8Error};

impl Chip8 {
    /// **DXYN - DRW Vx, Vy, nibble**: Draw N-byte sprite at coordinates (Vx, Vy).
    ///
    /// This instruction draws a sprite starting at memory location I at coordinates
    /// (Vx, Vy) on the display. The sprite is N bytes tall and 8 pixels wide.
    /// Each byte represents a row of 8 pixels. Sprites are drawn using XOR,
    /// so if a sprite pixel overlaps with an existing pixel, both pixels are turned off.
    ///
    /// # Arguments
    ///
    /// * `x` - Register index containing X coordinate (0-15)
    /// * `y` - Register index containing Y coordinate (0-15)
    /// * `n` - Height of the sprite in bytes (1-15)
    ///
    /// # Errors
    ///
    /// Returns `Chip8Error::InvalidRegister` if register indices are out of bounds.
    /// Returns `Chip8Error::IndexError` if memory location I is invalid.
    /// Returns `Chip8Error::FrameBufferOverflow` if framebuffer access is out of bounds.
    ///
    /// # Side Effects
    ///
    /// - Modifies pixels in the framebuffer using XOR operation
    /// - Sets VF register to 1 if any pixel collision occurs, 0 otherwise
    /// - Sets display_updated flag to true to indicate screen refresh needed
    /// - Coordinates wrap around screen boundaries (X: 0-63, Y: 0-31)
    ///
    /// # Examples
    ///
    /// Drawing a 5-byte tall sprite (like a font character) at position (10, 15):
    /// ```
    /// # use chip8_core::Chip8;
    /// let mut chip8 = Chip8::new().unwrap();
    /// chip8.set_vx_to_nn(0, 10).unwrap(); // X coordinate in V0
    /// chip8.set_vx_to_nn(1, 15).unwrap(); // Y coordinate in V1
    /// chip8.set_i_to_nnn(0x200).unwrap(); // Sprite data location
    /// chip8.draw_sprite(0, 1, 5).unwrap(); // Draw 5-byte sprite
    /// ```
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
        Ok(())
    }
}
