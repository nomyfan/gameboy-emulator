use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::sub::alu_sub;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_sub(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);

    let (value, z, h, c) = alu_sub(a, data);

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}
