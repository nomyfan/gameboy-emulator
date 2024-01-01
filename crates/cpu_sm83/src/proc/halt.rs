use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_halt(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_halt(true);

    get_cycles(opcode).0
}
