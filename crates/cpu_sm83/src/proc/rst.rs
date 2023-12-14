use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rst(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let value: u16 = match opcode {
        0xC7 => 0x00,
        0xCF => 0x08,
        0xD7 => 0x10,
        0xDF => 0x18,
        0xE7 => 0x20,
        0xEF => 0x28,
        0xF7 => 0x30,
        0xFF => 0x38,
        _ => unreachable!(),
    };
    cpu.stack_push_pc();
    cpu.jp(value);

    get_cycles(opcode).0
}
