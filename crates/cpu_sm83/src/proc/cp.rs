use super::utils::cpu_fetch_a;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_cp(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);

    cpu.set_flags(Some(value == a), Some(true), Some((a & 0x0F) < (value & 0x0F)), Some(a < value));

    get_cycles(opcode).0
}
