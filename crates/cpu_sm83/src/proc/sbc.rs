use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_sbc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let c = if cpu.flags().3 { 1u8 } else { 0u8 };

    let value = a.wrapping_sub(data).wrapping_sub(c);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) - ((c & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) - (c as i16) < 0;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}
