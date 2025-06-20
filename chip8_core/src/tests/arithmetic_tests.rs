use crate::*;

// Helper to run a single instruction
fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
    let pc = chip8.pc as usize;
    chip8.memory[pc] = (instruction >> 8) as u8;
    chip8.memory[pc + 1] = (instruction & 0xFF) as u8;
    chip8.run()
}

#[test]
fn test_op_6xkk_ld_vx_byte() {
    let mut chip8 = Chip8::new().unwrap();
    run_instruction(&mut chip8, 0x65AB).unwrap();
    assert_eq!(chip8.registers[5], 0xAB);
}

#[test]
fn test_op_7xkk_add_vx_byte() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[5] = 10;
    run_instruction(&mut chip8, 0x7505).unwrap();
    assert_eq!(chip8.registers[5], 15);
}

#[test]
fn test_op_7xkk_add_vx_byte_overflow() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[5] = 0xFF;
    run_instruction(&mut chip8, 0x7501).unwrap();
    assert_eq!(chip8.registers[5], 0); // Should wrap around
}

#[test]
fn test_op_8xy0_ld_vx_vy() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[2] = 0x42;
    run_instruction(&mut chip8, 0x8120).unwrap();
    assert_eq!(chip8.registers[1], 0x42);
}

#[test]
fn test_op_8xy1_or_vx_vy() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0b11001100;
    chip8.registers[2] = 0b10101010;
    run_instruction(&mut chip8, 0x8121).unwrap();
    assert_eq!(chip8.registers[1], 0b11101110);
}

#[test]
fn test_op_8xy2_and_vx_vy() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0b11001100;
    chip8.registers[2] = 0b10101010;
    run_instruction(&mut chip8, 0x8122).unwrap();
    assert_eq!(chip8.registers[1], 0b10001000);
}

#[test]
fn test_op_8xy3_xor_vx_vy() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0b11001100;
    chip8.registers[2] = 0b10101010;
    run_instruction(&mut chip8, 0x8123).unwrap();
    assert_eq!(chip8.registers[1], 0b01100110);
}

#[test]
fn test_op_8xy4_add_vx_vy_no_carry() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 10;
    chip8.registers[2] = 20;
    run_instruction(&mut chip8, 0x8124).unwrap();
    assert_eq!(chip8.registers[1], 30);
    assert_eq!(chip8.registers[0xF], 0, "VF should be 0 for no carry");
}

#[test]
fn test_op_8xy4_add_vx_vy_with_carry() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0xFF;
    chip8.registers[2] = 0x01;
    run_instruction(&mut chip8, 0x8124).unwrap();
    assert_eq!(chip8.registers[1], 0);
    assert_eq!(chip8.registers[0xF], 1, "VF should be 1 for carry");
}

#[test]
fn test_op_8xy5_sub_vx_vy_no_borrow() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 30;
    chip8.registers[2] = 10;
    run_instruction(&mut chip8, 0x8125).unwrap();
    assert_eq!(chip8.registers[1], 20);
    assert_eq!(chip8.registers[0xF], 1, "VF should be 1 for no borrow");
}

#[test]
fn test_op_8xy5_sub_vx_vy_with_borrow() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 10;
    chip8.registers[2] = 30;
    run_instruction(&mut chip8, 0x8125).unwrap();
    assert_eq!(chip8.registers[1], 236); // 256 - 20
    assert_eq!(chip8.registers[0xF], 0, "VF should be 0 for borrow");
}

#[test]
fn test_op_8xy6_shr_vx() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0b10101011;
    run_instruction(&mut chip8, 0x8126).unwrap();
    assert_eq!(chip8.registers[1], 0b01010101);
    assert_eq!(chip8.registers[0xF], 1, "VF should contain shifted out bit");
}

#[test]
fn test_op_8xy7_subn_vx_vy() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 10;
    chip8.registers[2] = 30;
    run_instruction(&mut chip8, 0x8127).unwrap();
    assert_eq!(chip8.registers[1], 20);
    assert_eq!(chip8.registers[0xF], 1, "VF should be 1 for no borrow");
}

#[test]
fn test_op_8xye_shl_vx() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0b10101010;
    run_instruction(&mut chip8, 0x812E).unwrap();
    assert_eq!(chip8.registers[1], 0b01010100);
    assert_eq!(chip8.registers[0xF], 1, "VF should contain shifted out bit");
}

#[test]
fn test_op_cxkk_rnd_vx() {
    let mut chip8 = Chip8::new().unwrap();
    // Run multiple times to ensure randomness
    let mut results = Vec::new();
    for _ in 0..10 {
        chip8.registers[1] = 0;
        run_instruction(&mut chip8, 0xC1FF).unwrap();
        results.push(chip8.registers[1]);
        chip8.reset().unwrap();
    }

    // Check that we got some variation
    let unique_count = results
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert!(
        unique_count > 1,
        "Random number generator should produce different values"
    );
}

#[test]
fn test_op_cxkk_rnd_vx_mask() {
    let mut chip8 = Chip8::new().unwrap();
    // Test with mask 0x0F (only lower 4 bits)
    for _ in 0..10 {
        chip8.registers[1] = 0;
        run_instruction(&mut chip8, 0xC10F).unwrap();
        assert!(
            chip8.registers[1] <= 0x0F,
            "Random value should be masked to 4 bits"
        );
        chip8.reset().unwrap();
    }
}
