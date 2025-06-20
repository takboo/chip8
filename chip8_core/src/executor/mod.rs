use crate::instruction::Instruction;
use crate::{Chip8, Chip8Error};

pub mod arithmetic;
pub mod display;
pub mod flow_control;
pub mod input;
pub mod memory;

pub trait InstructionExecutor {
    fn execute(&mut self, instruction: &Instruction) -> Result<(), Chip8Error>;
}

impl Chip8 {
    /// 执行单个指令
    pub fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), Chip8Error> {
        let (instr, x, y, n) = (
            instruction.instruction(),
            instruction.x(),
            instruction.y(),
            instruction.n(),
        );
        let nn = instruction.nn();
        let nnn = instruction.nnn();

        match (instr, x, y, n) {
            // 流程控制指令
            (0, 0, 0xE, 0) => self.clear_screen(),
            (0, 0, 0xE, 0xE) => self.return_from_subroutine(),
            (1, _, _, _) => self.jump_to_address(nnn),
            (2, _, _, _) => self.call_subroutine(nnn),

            // 条件跳转指令
            (3, _, _, _) => self.skip_if_vx_equals_nn(x, nn),
            (4, _, _, _) => self.skip_if_vx_not_equals_nn(x, nn),
            (5, _, _, 0) => self.skip_if_vx_equals_vy(x, y),
            (9, _, _, 0) => self.skip_if_vx_not_equals_vy(x, y),

            // 数据操作指令
            (6, _, _, _) => self.set_vx_to_nn(x, nn),
            (7, _, _, _) => self.add_nn_to_vx(x, nn),

            // 算术和逻辑指令
            (8, _, _, 0) => self.set_vx_to_vy(x, y),
            (8, _, _, 1) => self.or_vx_vy(x, y),
            (8, _, _, 2) => self.and_vx_vy(x, y),
            (8, _, _, 3) => self.xor_vx_vy(x, y),
            (8, _, _, 4) => self.add_vx_vy(x, y),
            (8, _, _, 5) => self.sub_vx_vy(x, y),
            (8, _, _, 6) => self.shift_vx_right(x),
            (8, _, _, 7) => self.sub_vy_vx(x, y),
            (8, _, _, 0xE) => self.shift_vx_left(x),

            // 内存和索引指令
            (0xa, _, _, _) => self.set_i_to_nnn(nnn),
            (0xb, _, _, _) => self.jump_to_v0_plus_nnn(nnn),
            (0xf, _, 0x2, 0x9) => self.set_i_to_font_location(x),
            (0xf, _, 0x3, 0x3) => self.store_bcd_of_vx(x),
            (0xf, _, 0x5, 0x5) => self.store_registers_to_memory(x),
            (0xf, _, 0x6, 0x5) => self.load_registers_from_memory(x),

            // 随机数指令
            (0xc, _, _, _) => self.set_vx_to_random_and_nn(x, nn),

            // 显示指令
            (0xd, _, _, _) => self.draw_sprite(x, y, n),

            // 输入指令
            (0xe, _, 0x9, 0xe) => self.skip_if_key_pressed(x),
            (0xe, _, 0xa, 0x1) => self.skip_if_key_not_pressed(x),
            (0xf, _, 0x0, 0xa) => self.wait_for_key_press(x),

            // 定时器指令
            (0xf, _, 0x0, 0x7) => self.set_vx_to_delay_timer(x),
            (0xf, _, 0x1, 0x5) => self.set_delay_timer_to_vx(x),
            (0xf, _, 0x1, 0x8) => self.set_sound_timer_to_vx(x),
            (0xf, _, 0x1, 0xe) => self.add_vx_to_i(x),

            _ => Err(Chip8Error::InvalidOpCode(format!(
                "Invalid opcode: {}",
                instruction
            ))),
        }
    }
}
