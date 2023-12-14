use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_adc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let c = if cpu.flags().3 { 1u8 } else { 0u8 };

    let value = a.wrapping_add(data).wrapping_add(c);
    let h = (value & 0xF) + (a & 0xF) + (c & 0xF) > 0xF;
    let c = (value as u16) + (a as u16) + (c as u16) > 0xFF;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), Some(false), Some(h), Some(c));

    get_cycles(opcode).0
}
