use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::daa::alu_daa;
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

/// Decimal adjust A
pub(crate) fn proc_daa(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let (_, flag_n, flag_h, flag_c) = cpu.flags();

    let (value, z, c) = alu_daa(a, flag_n, flag_h, flag_c);

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), None, Some(false), Some(c));

    get_cycles(opcode).0
}
