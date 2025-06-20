use crate::*;

// Helper to run a single instruction
fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
    let pc = chip8.pc as usize;
    chip8.memory[pc] = (instruction >> 8) as u8;
    chip8.memory[pc + 1] = (instruction & 0xFF) as u8;
    chip8.run()
}

#[test]
fn test_op_dxyn_drw() {
    let mut chip8 = Chip8::new().unwrap();
    // Load a simple 8x1 sprite (a horizontal line) into memory at 0x300
    chip8.i = 0x300;
    chip8.memory[0x300] = 0xFF;
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
    chip8.memory[0x300] = 0b11000000; // Sprite to draw
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
    chip8.memory[0x300] = 0b11110000;
    chip8.memory[0x301] = 0b00001111;
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
    chip8.memory[0x300] = 0xFF;
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
    chip8.memory[0x300] = 0xFF;
    chip8.memory[0x301] = 0xFF;
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
    chip8.memory[0x300] = 0b10000001;
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
    chip8.memory[0x300] = 0xFF;
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
