use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rrca(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = a & 1;

    // Move the LSB(it) to the MSB(it)
    cpu_write_a(cpu, (a >> 1) | (c << 7));
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));

    get_cycles(opcode).0
}
