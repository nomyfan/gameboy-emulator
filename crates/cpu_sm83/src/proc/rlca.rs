use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rlca(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = (a >> 7) & 1;

    // Move the MSB(it) to the LSB(it)
    cpu_write_a(cpu, (a << 1) | c);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));

    get_cycles(opcode).0
}
