use crate::*;

// Helper to run a single instruction
fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
    let pc = chip8.pc as usize;
    chip8.memory[pc] = (instruction >> 8) as u8;
    chip8.memory[pc + 1] = (instruction & 0xFF) as u8;
    chip8.run()
}

#[test]
fn test_op_00e0_cls() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.framebuffer.iter_mut().for_each(|p| *p = 1);
    chip8.display_updated = false;
    run_instruction(&mut chip8, 0x00E0).unwrap();
    assert!(chip8.framebuffer.iter().all(|&p| p == 0));
    assert!(chip8.is_display_updated());
}

#[test]
fn test_op_1nnn_jp() {
    let mut chip8 = Chip8::new().unwrap();
    run_instruction(&mut chip8, 0x1ABC).unwrap();
    assert_eq!(chip8.pc, 0x0ABC);
}

#[test]
fn test_op_2nnn_call_and_00ee_ret() {
    let mut chip8 = Chip8::new().unwrap();
    let initial_pc = chip8.pc;

    // CALL 0x300
    run_instruction(&mut chip8, 0x2300).unwrap();
    assert_eq!(chip8.pc, 0x300, "PC should jump to subroutine address");
    assert_eq!(chip8.sp, 1, "Stack pointer should increment");
    assert_eq!(
        chip8.stack[0],
        initial_pc + 2,
        "Return address should be on stack"
    );

    // Let's test the run command for RET
    let mut chip8 = Chip8::new().unwrap();
    chip8.pc = 0x300;
    chip8.sp = 1;
    chip8.stack[0] = 0x250;

    run_instruction(&mut chip8, 0x00EE).unwrap();
    assert_eq!(chip8.pc, 0x250);
    assert_eq!(chip8.sp, 0);
}

#[test]
fn test_op_3xkk_se_vx_byte_skip() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[3] = 0x42;
    let initial_pc = chip8.pc;
    run_instruction(&mut chip8, 0x3342).unwrap();
    assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
}

#[test]
fn test_op_3xkk_se_vx_byte_no_skip() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[3] = 0x42;
    let initial_pc = chip8.pc;
    run_instruction(&mut chip8, 0x3343).unwrap();
    assert_eq!(chip8.pc, initial_pc + 2, "PC should not skip");
}

#[test]
fn test_op_4xkk_sne_vx_byte_skip() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[3] = 0x42;
    let initial_pc = chip8.pc;
    run_instruction(&mut chip8, 0x4343).unwrap(); // Different value, use 4xxx for SNE
    assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
}

#[test]
fn test_op_5xy0_se_vx_vy_skip() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0x42;
    chip8.registers[2] = 0x42;
    let initial_pc = chip8.pc;
    run_instruction(&mut chip8, 0x5120).unwrap();
    assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
}

#[test]
fn test_op_9xy0_sne_vx_vy_skip() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[1] = 0x42;
    chip8.registers[2] = 0x43;
    let initial_pc = chip8.pc;
    run_instruction(&mut chip8, 0x9120).unwrap();
    assert_eq!(chip8.pc, initial_pc + 4, "PC should skip next instruction");
}

#[test]
fn test_op_bnnn_jp_v0() {
    let mut chip8 = Chip8::new().unwrap();
    chip8.registers[0] = 0x05;
    run_instruction(&mut chip8, 0xB200).unwrap();
    assert_eq!(chip8.pc, 0x205, "PC should be V0 + nnn");
}

#[test]
fn test_nested_subroutine_calls() {
    let mut chip8 = Chip8::new().unwrap();
    let initial_pc = chip8.pc;

    // First call
    run_instruction(&mut chip8, 0x2300).unwrap();
    assert_eq!(chip8.pc, 0x300);
    assert_eq!(chip8.sp, 1);
    assert_eq!(chip8.stack[0], initial_pc + 2);

    // Second nested call
    run_instruction(&mut chip8, 0x2400).unwrap();
    assert_eq!(chip8.pc, 0x400);
    assert_eq!(chip8.sp, 2);
    assert_eq!(chip8.stack[1], 0x302);

    // Return from second call
    run_instruction(&mut chip8, 0x00EE).unwrap();
    assert_eq!(chip8.pc, 0x302);
    assert_eq!(chip8.sp, 1);

    // Return from first call
    run_instruction(&mut chip8, 0x00EE).unwrap();
    assert_eq!(chip8.pc, initial_pc + 2);
    assert_eq!(chip8.sp, 0);
}
