use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_ccf(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_flags(None, Some(false), Some(false), Some(!cpu.flags().3));

    get_cycles(opcode).0
}
