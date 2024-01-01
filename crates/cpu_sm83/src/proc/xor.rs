use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::xor::alu_xor;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_xor(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);

    let (value, z) = alu_xor(a, operand);

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(false), Some(false), Some(false));

    get_cycles(opcode).0
}
