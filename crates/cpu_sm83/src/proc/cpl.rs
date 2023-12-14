use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_cpl(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    cpu_write_a(cpu, !a);
    cpu.set_flags(None, Some(true), Some(true), None);

    get_cycles(opcode).0
}
