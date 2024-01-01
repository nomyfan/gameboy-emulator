use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::rrca::alu_rrca;
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rrca(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let (value, c) = alu_rrca(cpu_fetch_a(cpu));

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));

    get_cycles(opcode).0
}
