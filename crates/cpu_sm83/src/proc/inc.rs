use crate::alu::inc::*;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_inc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am);

    if (opcode & 0x03) != 0x03 {
        let (value, z, h) = alu_inc_8(value as u8);
        cpu.write_data(am, 0, value as u16);
        cpu.set_flags(Some(z), Some(false), Some(h), None);
    } else {
        cpu.write_data(am, 0, alu_inc_16(value));
    }

    get_cycles(opcode).0
}
