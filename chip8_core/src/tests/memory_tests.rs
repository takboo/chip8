use crate::*;

// Helper to run a single instruction
fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
    let pc = chip8.pc as usize;
    chip8.memory[pc] = (instruction >> 8) as u8;
    chip8.memory[pc + 1] = (instruction & 0xFF) as u8;
    chip8.run()
}

#[test]
fn test_op_annn_ld_i() {
    let mut chip8 = Chip8::new().unwrap();
    run_instruction(&mut chip8, 0xA123).unwrap();
    assert_eq!(chip8.i, 0x123);
}

#[test]
fn test_op_fx07_ld_vx_dt() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.dt = 42;
    run_instruction(&mut chip8, 0xF107).unwrap();
    assert_eq!(chip8.registers[1], 42);
}

#[test]
fn test_op_fx15_ld_dt_vx() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[2] = 100;
    run_instruction(&mut chip8, 0xF215).unwrap();
    assert_eq!(chip8.dt, 100);
}

#[test]
fn test_op_fx18_ld_st_vx() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[3] = 200;
    run_instruction(&mut chip8, 0xF318).unwrap();
    assert_eq!(chip8.st, 200);
}

#[test]
fn test_op_fx1e_add_i_vx() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.i = 0x100;
    chip8.registers[4] = 0x50;
    run_instruction(&mut chip8, 0xF41E).unwrap();
    assert_eq!(chip8.i, 0x150);
}

#[test]
fn test_op_fx1e_add_i_vx_overflow() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.i = 0xFFF0;
    chip8.registers[4] = 0x20;
    run_instruction(&mut chip8, 0xF41E).unwrap();
    assert_eq!(chip8.i, 0x10); // Should wrap around
}

#[test]
fn test_op_fx29_ld_f_vx() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0xA; // Digit A
    run_instruction(&mut chip8, 0xF129).unwrap();
    // Font for digit A should be at FONT_START_ADDRESS + (0xA * 5)
    let expected_address = FONT_START_ADDRESS as u16 + (0xA * 5);
    assert_eq!(chip8.i, expected_address);
}

#[test]
fn test_op_fx29_ld_f_vx_all_digits() {
    let mut chip8 = Chip8::new().unwrap();
    for digit in 0..=0xF {
        chip8.registers[1] = digit;
        run_instruction(&mut chip8, 0xF129).unwrap();
        let expected_address = FONT_START_ADDRESS as u16 + (digit as u16 * 5);
        assert_eq!(chip8.i, expected_address);
        chip8.reset().unwrap();
    }
}

#[test]
fn test_op_fx33_ld_b_vx() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[0] = 123;
    chip8.i = 0x300;
    run_instruction(&mut chip8, 0xF033).unwrap();
    assert_eq!(chip8.memory[0x300], 1);
    assert_eq!(chip8.memory[0x301], 2);
    assert_eq!(chip8.memory[0x302], 3);
}

#[test]
fn test_op_fx33_ld_b_vx_single_digit() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[0] = 7;
    chip8.i = 0x300;
    run_instruction(&mut chip8, 0xF033).unwrap();
    assert_eq!(chip8.memory[0x300], 0);
    assert_eq!(chip8.memory[0x301], 0);
    assert_eq!(chip8.memory[0x302], 7);
}

#[test]
fn test_op_fx33_ld_b_vx_max_value() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[0] = 255;
    chip8.i = 0x300;
    run_instruction(&mut chip8, 0xF033).unwrap();
    assert_eq!(chip8.memory[0x300], 2);
    assert_eq!(chip8.memory[0x301], 5);
    assert_eq!(chip8.memory[0x302], 5);
}

#[test]
fn test_op_fx55_ld_i_vx() {
    let mut chip8 = Chip8::new().unwrap();
    for i in 0..=5 {
        chip8.registers[i] = i as u8;
    }
    chip8.i = 0x300;
    run_instruction(&mut chip8, 0xF555).unwrap();
    for i in 0..=5 {
        assert_eq!(chip8.memory[0x300 + i], i as u8);
    }
}

#[test]
fn test_op_fx55_ld_i_vx_partial() {
    let mut chip8 = Chip8::new().unwrap();
    for i in 0..16 {
        chip8.registers[i] = i as u8 + 10;
    }
    chip8.i = 0x300;
    run_instruction(&mut chip8, 0xF255).unwrap(); // Only store V0-V2

    // Check stored registers
    for i in 0..=2 {
        assert_eq!(chip8.memory[0x300 + i], i as u8 + 10);
    }

    // Check that other memory locations weren't modified
    assert_eq!(chip8.memory[0x303], 0);
}

#[test]
fn test_op_fx65_ld_vx_i() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.i = 0x300;
    for i in 0..=5 {
        chip8.memory[0x300 + i] = i as u8;
    }

    run_instruction(&mut chip8, 0xF565).unwrap();
    for i in 0..=5 {
        assert_eq!(chip8.registers[i], i as u8);
    }
}

#[test]
fn test_timer_operations() {
    let mut chip8 = Chip8::new().unwrap();

    // Set delay timer
    chip8.registers[1] = 60;
    run_instruction(&mut chip8, 0xF115).unwrap();
    assert_eq!(chip8.dt, 60);

    // Read delay timer
    chip8.pc = 0x200; // Reset PC
    run_instruction(&mut chip8, 0xF207).unwrap();
    assert_eq!(chip8.registers[2], 60);

    // Set sound timer
    chip8.pc = 0x200; // Reset PC
    chip8.registers[3] = 30;
    run_instruction(&mut chip8, 0xF318).unwrap();
    assert_eq!(chip8.st, 30);
}

#[test]
fn test_memory_boundary_conditions() {
    let mut chip8 = Chip8::new().unwrap();

    // Test near memory boundary
    chip8.i = 4093; // Near end of memory
    chip8.registers[0] = 123;
    run_instruction(&mut chip8, 0xF033).unwrap();
    assert_eq!(chip8.memory[4093], 1);
    assert_eq!(chip8.memory[4094], 2);
    assert_eq!(chip8.memory[4095], 3);
}

#[test]
fn test_index_register_overflow_boundary() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.i = 0xFFFF;
    chip8.registers[1] = 1;
    run_instruction(&mut chip8, 0xF11E).unwrap();
    assert_eq!(chip8.i, 0); // Should wrap to 0
}

#[test]
fn test_font_loading_verification() {
    let chip8 = Chip8::new().unwrap();

    // Verify all font characters are loaded correctly
    for i in 0..FONT_SET.len() {
        assert_eq!(chip8.memory[FONT_START_ADDRESS + i], FONT_SET[i]);
    }

    // Verify memory before font is clear
    assert_eq!(chip8.memory[FONT_START_ADDRESS - 1], 0);

    // Verify memory after font is clear
    assert_eq!(chip8.memory[FONT_START_ADDRESS + FONT_SET.len()], 0);
}
