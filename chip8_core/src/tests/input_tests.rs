use crate::*;

// Helper to run a single instruction
fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
    let pc = chip8.pc as usize;
    chip8.memory[pc] = (instruction >> 8) as u8;
    chip8.memory[pc + 1] = (instruction & 0xFF) as u8;
    chip8.run()
}

#[test]
fn test_op_ex9e_skp_key_pressed() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 5; // Key index
    chip8.key_press(5); // Press key 5
    let initial_pc = chip8.pc;

    run_instruction(&mut chip8, 0xE19E).unwrap();
    assert_eq!(
        chip8.pc,
        initial_pc + 4,
        "PC should skip next instruction when key is pressed"
    );
}

#[test]
fn test_op_ex9e_skp_key_not_pressed() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 5; // Key index
    // Don't press any key
    let initial_pc = chip8.pc;

    run_instruction(&mut chip8, 0xE19E).unwrap();
    assert_eq!(
        chip8.pc,
        initial_pc + 2,
        "PC should not skip when key is not pressed"
    );
}

#[test]
fn test_op_exa1_sknp_key_not_pressed() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 5; // Key index
    // Don't press any key
    let initial_pc = chip8.pc;

    run_instruction(&mut chip8, 0xE1A1).unwrap();
    assert_eq!(
        chip8.pc,
        initial_pc + 4,
        "PC should skip next instruction when key is not pressed"
    );
}

#[test]
fn test_op_exa1_sknp_key_pressed() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 5; // Key index
    chip8.key_press(5); // Press key 5
    let initial_pc = chip8.pc;

    run_instruction(&mut chip8, 0xE1A1).unwrap();
    assert_eq!(
        chip8.pc,
        initial_pc + 2,
        "PC should not skip when key is pressed"
    );
}

#[test]
fn test_op_fx0a_ld_vx_k_wait() {
    let mut chip8 = Chip8::new().unwrap();
    let initial_pc = chip8.pc;
    // Run without a key press
    run_instruction(&mut chip8, 0xF30A).unwrap();
    // PC should be rewound, effectively pausing execution
    assert_eq!(chip8.pc, initial_pc);
}

#[test]
fn test_op_fx0a_ld_vx_k_press() {
    let mut chip8 = Chip8::new().unwrap();
    let initial_pc = chip8.pc;
    // Simulate key press for key 0xA
    chip8.key_press(0xA);
    run_instruction(&mut chip8, 0xF30A).unwrap();
    // Register V3 should contain 0xA
    assert_eq!(chip8.registers[3], 0xA);
    // PC should advance normally
    assert_eq!(chip8.pc, initial_pc + 2);
}

#[test]
fn test_key_press_release_cycle() {
    let mut chip8 = Chip8::new().unwrap();

    // Initially no keys pressed
    for i in 0..16 {
        assert_eq!(chip8.keyboard[i], 0);
    }

    // Press key 5
    chip8.key_press(5);
    assert_eq!(chip8.keyboard[5], 1);

    // Release key 5
    chip8.key_release(5);
    assert_eq!(chip8.keyboard[5], 0);
}

#[test]
fn test_multiple_keys_pressed() {
    let mut chip8 = Chip8::new().unwrap();

    // Press multiple keys
    chip8.key_press(0);
    chip8.key_press(5);
    chip8.key_press(15);

    assert_eq!(chip8.keyboard[0], 1);
    assert_eq!(chip8.keyboard[5], 1);
    assert_eq!(chip8.keyboard[15], 1);

    // Other keys should still be unpressed
    assert_eq!(chip8.keyboard[1], 0);
    assert_eq!(chip8.keyboard[7], 0);
}

#[test]
fn test_key_input_invalid_index() {
    let mut chip8 = Chip8::new().unwrap();

    // These should not panic or cause errors
    chip8.key_press(16); // Invalid key
    chip8.key_press(255); // Invalid key
    chip8.key_release(20); // Invalid key

    // All valid keys should still be unpressed
    for i in 0..16 {
        assert_eq!(chip8.keyboard[i], 0);
    }
}

#[test]
fn test_key_detection_priority() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.key_press(0);
    chip8.key_press(5);
    chip8.key_press(10);

    let initial_pc = chip8.pc;
    run_instruction(&mut chip8, 0xF10A).unwrap(); // Wait for key

    // Should detect the first pressed key (lowest index)
    assert_eq!(chip8.registers[1], 0);
    assert_eq!(chip8.pc, initial_pc + 2);
}

#[test]
fn test_key_instruction_with_invalid_key_register() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 16; // Invalid key index

    // This should return an error
    let result = run_instruction(&mut chip8, 0xE19E);
    assert!(matches!(result, Err(Chip8Error::InvalidKey(16))));
}
