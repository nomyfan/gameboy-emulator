use super::ret::proc_ret;
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_reti(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_ime(true);
    proc_ret(cpu, opcode, &None);

    get_cycles(opcode).0
}
