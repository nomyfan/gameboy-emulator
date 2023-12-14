use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_ei(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    // TODO: We need another cycle to effect.
    cpu.set_ime(true);

    get_cycles(opcode).0
}
