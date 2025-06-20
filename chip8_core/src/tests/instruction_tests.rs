use crate::*;

#[test]
fn test_new() {
    let chip8 = Chip8::new().unwrap();

    // Verify initial state
    assert_eq!(chip8.pc, 0x200);
    assert_eq!(chip8.sp, 0);
    assert_eq!(chip8.i, 0);
    assert_eq!(chip8.dt, 0);
    assert_eq!(chip8.st, 0);

    // Verify font was loaded
    let font_in_memory = &chip8.memory[FONT_START_ADDRESS..FONT_START_ADDRESS + FONT_SET.len()];
    assert_eq!(font_in_memory, FONT_SET);
}

#[test]
fn test_reset() {
    let mut chip8 = Chip8::new().unwrap();
    // Set some state to non-default values
    chip8.memory[0x300] = 0xFF;
    chip8.registers[0] = 0xAA;
    chip8.pc = 0x300;
    chip8.sp = 5;
    chip8.i = 0x123;
    chip8.stack[0] = 0x456;
    chip8.dt = 10;
    chip8.st = 20;
    chip8.framebuffer[0] = 1;
    chip8.keyboard[0] = 1;

    chip8.reset().unwrap();

    // Verify all fields were reset
    assert_eq!(chip8.registers, [0; 16]);
    assert_eq!(chip8.pc, 0x200);
    assert_eq!(chip8.sp, 0);
    assert_eq!(chip8.i, 0);
    assert_eq!(chip8.stack, [0; 16]);
    assert_eq!(chip8.dt, 0);
    assert_eq!(chip8.st, 0);
    assert_eq!(chip8.framebuffer, [0; 64 * 32]);
    assert_eq!(chip8.keyboard, [0; 16]);

    // Verify memory is cleared except for the font
    let font_end = FONT_START_ADDRESS + FONT_SET.len();
    let font_in_memory = &chip8.memory[FONT_START_ADDRESS..font_end];
    assert_eq!(font_in_memory, FONT_SET);
    // Check a byte before the font
    assert_eq!(chip8.memory[FONT_START_ADDRESS - 1], 0);
    // Check a byte after the font
    assert_eq!(chip8.memory[font_end], 0);
    // Check the location that was previously dirty
    assert_eq!(chip8.memory[0x300], 0);
}

#[test]
fn test_load_rom() {
    let mut chip8 = Chip8::new().unwrap();
    let rom_data = vec![0x1, 0x2, 0x3, 0x4];
    chip8.load_rom(&rom_data).unwrap();

    let memory_slice = &chip8.memory[ROM_START_ADDRESS..ROM_START_ADDRESS + rom_data.len()];
    assert_eq!(memory_slice, rom_data.as_slice());
}

#[test]
fn test_load_rom_out_of_bounds() {
    let mut chip8 = Chip8::new().unwrap();
    let rom_size = chip8.memory.len() - ROM_START_ADDRESS + 1;
    let rom_data = vec![0u8; rom_size];

    assert!(matches!(
        chip8.load_rom(&rom_data),
        Err(Chip8Error::LoadRomError)
    ));
}

#[test]
fn test_instructions_decoding() {
    let instruction = 0xABCD;
    let decoded = Instruction::new(instruction);
    assert_eq!(decoded.instruction(), 0xA);
    assert_eq!(decoded.x(), 0xB);
    assert_eq!(decoded.y(), 0xC);
    assert_eq!(decoded.n(), 0xD);
    assert_eq!(decoded.nn(), 0xCD);
    assert_eq!(decoded.nnn(), 0xBCD);
}

#[test]
fn test_fetch_success() {
    let mut chip8 = Chip8::new().unwrap();
    // Load an instruction 0x1234 at the start of ROM space
    chip8.memory[ROM_START_ADDRESS] = 0x12;
    chip8.memory[ROM_START_ADDRESS + 1] = 0x34;

    let initial_pc = chip8.pc;
    let instructions = chip8.fetch().unwrap();

    assert_eq!(instructions.instruction(), 0x1);
    assert_eq!(instructions.x(), 0x2);
    assert_eq!(instructions.y(), 0x3);
    assert_eq!(instructions.n(), 0x4);
    assert_eq!(instructions.nn(), 0x34);
    assert_eq!(instructions.nnn(), 0x234);

    // PC should advance by 2 bytes
    assert_eq!(chip8.pc, initial_pc + 2);
}

#[test]
fn test_fetch_out_of_bounds() {
    let mut chip8 = Chip8::new().unwrap();
    // Set PC to the last byte of memory, where a 2-byte instruction cannot be read
    chip8.pc = (chip8.memory.len() - 1) as u16;
    let initial_pc = chip8.pc;

    let result = chip8.fetch();
    assert!(matches!(result, Err(Chip8Error::PCError(_))));

    // PC should not advance on failure
    assert_eq!(chip8.pc, initial_pc);
}

#[test]
fn test_invalid_opcode() {
    let mut chip8 = Chip8::new().unwrap();
    // 0x0FFF is not a valid opcode
    let result = run_instruction(&mut chip8, 0x0FFF);
    assert!(matches!(result, Err(Chip8Error::InvalidOpCode(_))));
}

#[test]
fn test_instruction_type_flow_control() {
    use crate::instruction::InstructionType;

    // Test return from subroutine (0x00EE)
    let instruction = Instruction::new(0x00EE);
    assert_eq!(instruction.instruction_type(), InstructionType::FlowControl);

    // Test jump to address (0x1NNN)
    let instruction = Instruction::new(0x1234);
    assert_eq!(instruction.instruction_type(), InstructionType::FlowControl);

    // Test call subroutine (0x2NNN)
    let instruction = Instruction::new(0x2ABC);
    assert_eq!(instruction.instruction_type(), InstructionType::FlowControl);

    // Test jump to V0 + NNN (0xBNNN)
    let instruction = Instruction::new(0xB123);
    assert_eq!(instruction.instruction_type(), InstructionType::FlowControl);
}

#[test]
fn test_instruction_type_conditional_skip() {
    use crate::instruction::InstructionType;

    // Test skip if Vx == NN (0x3XNN)
    let instruction = Instruction::new(0x3456);
    assert_eq!(
        instruction.instruction_type(),
        InstructionType::ConditionalSkip
    );

    // Test skip if Vx != NN (0x4XNN)
    let instruction = Instruction::new(0x4789);
    assert_eq!(
        instruction.instruction_type(),
        InstructionType::ConditionalSkip
    );

    // Test skip if Vx == Vy (0x5XY0)
    let instruction = Instruction::new(0x5AB0);
    assert_eq!(
        instruction.instruction_type(),
        InstructionType::ConditionalSkip
    );

    // Test skip if Vx != Vy (0x9XY0)
    let instruction = Instruction::new(0x9CD0);
    assert_eq!(
        instruction.instruction_type(),
        InstructionType::ConditionalSkip
    );

    // Test skip if key pressed (0xEX9E)
    let instruction = Instruction::new(0xE19E);
    assert_eq!(
        instruction.instruction_type(),
        InstructionType::ConditionalSkip
    );

    // Test skip if key not pressed (0xEXA1)
    let instruction = Instruction::new(0xE2A1);
    assert_eq!(
        instruction.instruction_type(),
        InstructionType::ConditionalSkip
    );
}

#[test]
fn test_instruction_type_register_op() {
    use crate::instruction::InstructionType;

    // Test set Vx = NN (0x6XNN)
    let instruction = Instruction::new(0x6123);
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    // Test add NN to Vx (0x7XNN)
    let instruction = Instruction::new(0x7456);
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    // Test arithmetic operations (0x8XY_)
    let instruction = Instruction::new(0x8120); // Vx = Vy
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x8341); // Vx |= Vy
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x8562); // Vx &= Vy
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x8783); // Vx ^= Vy
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x89A4); // Vx += Vy
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x8BC5); // Vx -= Vy
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x8DE6); // Vx >>= 1
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x8F17); // Vx = Vy - Vx
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);

    let instruction = Instruction::new(0x823E); // Vx <<= 1
    assert_eq!(instruction.instruction_type(), InstructionType::RegisterOp);
}

#[test]
fn test_instruction_type_memory_op() {
    use crate::instruction::InstructionType;

    // Test set I = NNN (0xANNN)
    let instruction = Instruction::new(0xA123);
    assert_eq!(instruction.instruction_type(), InstructionType::MemoryOp);

    // Test I += Vx (0xFX1E)
    let instruction = Instruction::new(0xF31E);
    assert_eq!(instruction.instruction_type(), InstructionType::MemoryOp);

    // Test I = sprite address for Vx (0xFX29)
    let instruction = Instruction::new(0xF529);
    assert_eq!(instruction.instruction_type(), InstructionType::MemoryOp);

    // Test store BCD of Vx (0xFX33)
    let instruction = Instruction::new(0xF733);
    assert_eq!(instruction.instruction_type(), InstructionType::MemoryOp);

    // Test store V0-Vx to memory (0xFX55)
    let instruction = Instruction::new(0xF955);
    assert_eq!(instruction.instruction_type(), InstructionType::MemoryOp);

    // Test load V0-Vx from memory (0xFX65)
    let instruction = Instruction::new(0xFB65);
    assert_eq!(instruction.instruction_type(), InstructionType::MemoryOp);
}

#[test]
fn test_instruction_type_display() {
    use crate::instruction::InstructionType;

    // Test clear screen (0x00E0)
    let instruction = Instruction::new(0x00E0);
    assert_eq!(instruction.instruction_type(), InstructionType::Display);

    // Test draw sprite (0xDXYN)
    let instruction = Instruction::new(0xD123);
    assert_eq!(instruction.instruction_type(), InstructionType::Display);
}

#[test]
fn test_instruction_type_input_output() {
    use crate::instruction::InstructionType;

    // Test wait for key press (0xFX0A)
    let instruction = Instruction::new(0xF50A);
    assert_eq!(instruction.instruction_type(), InstructionType::InputOutput);
}

#[test]
fn test_instruction_type_timer() {
    use crate::instruction::InstructionType;

    // Test Vx = delay timer (0xFX07)
    let instruction = Instruction::new(0xF307);
    assert_eq!(instruction.instruction_type(), InstructionType::Timer);

    // Test delay timer = Vx (0xFX15)
    let instruction = Instruction::new(0xF515);
    assert_eq!(instruction.instruction_type(), InstructionType::Timer);

    // Test sound timer = Vx (0xFX18)
    let instruction = Instruction::new(0xF718);
    assert_eq!(instruction.instruction_type(), InstructionType::Timer);
}

#[test]
fn test_instruction_type_random() {
    use crate::instruction::InstructionType;

    // Test Vx = random() & NN (0xCXNN)
    let instruction = Instruction::new(0xC456);
    assert_eq!(instruction.instruction_type(), InstructionType::Random);
}

// Helper to run a single instruction
fn run_instruction(chip8: &mut Chip8, instruction: u16) -> Result<(), Chip8Error> {
    let pc = chip8.pc as usize;
    chip8.memory[pc] = (instruction >> 8) as u8;
    chip8.memory[pc + 1] = (instruction & 0xFF) as u8;
    chip8.run()
}
