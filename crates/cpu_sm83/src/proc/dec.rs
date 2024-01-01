use crate::alu::dec::*;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_dec(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am);

    if (opcode & 0xB) != 0xB {
        let (value, z, h) = alu_dec_8(value as u8);
        cpu.set_flags(Some(z), Some(true), Some(h), None);
        cpu.write_data(am, 0, value as u16);
    } else {
        cpu.write_data(am, 0, alu_dec_16(value));
    }

    get_cycles(opcode).0
}
