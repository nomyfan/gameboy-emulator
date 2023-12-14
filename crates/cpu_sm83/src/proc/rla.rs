use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rla(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = (a >> 7) & 1 == 1;
    let carry = if cpu.flags().3 { 1u8 } else { 0u8 };

    cpu_write_a(cpu, (a << 1) | carry);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));

    get_cycles(opcode).0
}
