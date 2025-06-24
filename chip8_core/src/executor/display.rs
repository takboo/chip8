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
    pub(super) fn draw_sprite(&mut self, x: usize, y: usize, n: u8) -> Result<(), Chip8Error> {
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
                .read_byte(self.i as usize + row)
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

#[cfg(test)]
mod tests {
    use crate::{tests::run_instruction, *};

    #[test]
    fn test_op_dxyn_drw() {
        let mut chip8 = Chip8::new().unwrap();
        // Load a simple 8x1 sprite (a horizontal line) into memory at 0x300
        chip8.i = 0x300;
        let value = [0xFF];
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");
        // Set Vx and Vy to draw at (10, 5)
        chip8.registers[1] = 10;
        chip8.registers[2] = 5;

        // Draw a sprite of height 1 from register V1, V2
        run_instruction(&mut chip8, 0xD121).unwrap();

        // Check that the pixels are set correctly
        for i in 0..8 {
            assert_eq!(chip8.framebuffer[5 * 64 + (10 + i)], 1);
        }
        // Check that VF is 0 (no collision)
        assert_eq!(chip8.registers[0xF], 0);
        assert!(chip8.is_display_updated());
    }

    #[test]
    fn test_op_dxyn_drw_collision() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        let value = [0b11000000];
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");
        chip8.registers[1] = 10;
        chip8.registers[2] = 5;

        // Pre-set a pixel that will collide
        chip8.framebuffer[5 * 64 + 10] = 1;

        run_instruction(&mut chip8, 0xD121).unwrap();

        // The first pixel was on, it should be turned off
        assert_eq!(chip8.framebuffer[5 * 64 + 10], 0);
        // The second pixel was off, it should be turned on
        assert_eq!(chip8.framebuffer[5 * 64 + 11], 1);
        // Check that VF is 1 (collision)
        assert_eq!(chip8.registers[0xF], 1);
    }

    #[test]
    fn test_op_dxyn_drw_multiline_sprite() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        // Load a 2-line sprite
        let value = [0b11110000, 0b00001111];
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");
        chip8.registers[1] = 5; // x position
        chip8.registers[2] = 3; // y position

        run_instruction(&mut chip8, 0xD122).unwrap(); // Draw 2-line sprite

        // Check first line
        for i in 0..4 {
            assert_eq!(chip8.framebuffer[3 * 64 + (5 + i)], 1);
        }
        for i in 4..8 {
            assert_eq!(chip8.framebuffer[3 * 64 + (5 + i)], 0);
        }

        // Check second line
        for i in 0..4 {
            assert_eq!(chip8.framebuffer[4 * 64 + (5 + i)], 0);
        }
        for i in 4..8 {
            assert_eq!(chip8.framebuffer[4 * 64 + (5 + i)], 1);
        }
    }

    #[test]
    fn test_op_dxyn_drw_wrapping() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        let value = [0xFF];
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");
        // Position at edge of screen
        chip8.registers[1] = 60; // x position (screen is 64 wide)
        chip8.registers[2] = 31; // y position (screen is 32 tall)

        run_instruction(&mut chip8, 0xD121).unwrap();

        // Only the first 4 pixels should be drawn (since x=60, only pixels 60-63 fit)
        for i in 0..4 {
            assert_eq!(chip8.framebuffer[31 * 64 + (60 + i)], 1);
        }
    }

    #[test]
    fn test_op_dxyn_drw_bottom_edge() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        let value = [0xFF, 0xFF];
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");
        // Position at bottom edge
        chip8.registers[1] = 0;
        chip8.registers[2] = 31; // Last row

        run_instruction(&mut chip8, 0xD122).unwrap(); // Try to draw 2 lines

        // Only the first line should be drawn
        for i in 0..8 {
            assert_eq!(chip8.framebuffer[31 * 64 + i], 1);
        }
        // Second line should not exist (would be row 32, which is out of bounds)
    }

    #[test]
    fn test_op_dxyn_drw_coordinate_wrapping() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        let value = [0b10000001];
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");

        // Use coordinates that would wrap
        chip8.registers[1] = 70; // Should wrap to 70 % 64 = 6
        chip8.registers[2] = 40; // Should wrap to 40 % 32 = 8

        run_instruction(&mut chip8, 0xD121).unwrap();

        // Check wrapped position (6, 8)
        assert_eq!(chip8.framebuffer[8 * 64 + 6], 1); // First bit
        assert_eq!(chip8.framebuffer[8 * 64 + 13], 1); // Last bit (6+7)
    }

    #[test]
    fn test_sprite_xor_behavior() {
        let mut chip8 = Chip8::new().unwrap();
        chip8.i = 0x300;
        let value = [0xFF];
        chip8
            .memory
            .write_at(&value, 0x300)
            .expect("Failed to write memory");
        chip8.registers[1] = 10;
        chip8.registers[2] = 5;

        // Draw sprite first time
        run_instruction(&mut chip8, 0xD121).unwrap();
        // All pixels should be on
        for i in 0..8 {
            assert_eq!(chip8.framebuffer[5 * 64 + (10 + i)], 1);
        }

        // Reset PC for second draw
        chip8.pc = 0x200;

        // Draw same sprite again (should XOR and turn pixels off)
        run_instruction(&mut chip8, 0xD121).unwrap();
        // All pixels should be off now
        for i in 0..8 {
            assert_eq!(chip8.framebuffer[5 * 64 + (10 + i)], 0);
        }
        // Should have collision detection
        assert_eq!(chip8.registers[0xF], 1);
    }
}
