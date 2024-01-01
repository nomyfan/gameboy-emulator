use super::utils::cpu_fetch_a;
use crate::alu::cp::alu_cp;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_cp(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let (z, h, c) = alu_cp(cpu_fetch_a(cpu), cpu.fetch_data(am) as u8);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}
