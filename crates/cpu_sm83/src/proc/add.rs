use crate::alu::add::*;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_add(
    cpu: &mut impl Cpu16,
    opcode: u8,
    am1: &AddressingMode,
    am2: &AddressingMode,
) -> u8 {
    let operand2 = cpu.fetch_data(am2);
    let operand1 = cpu.fetch_data(am1);

    let is_rr = (opcode & 0x09) == 0x09;
    let is_sp_r8 = opcode == 0xE8;

    if is_rr {
        let (sum, h, c) = alu_add_16(operand1, operand2);
        cpu.write_data(am1, 0, sum);
        cpu.set_flags(None, Some(false), Some(h), Some(c));
    } else if is_sp_r8 {
        let (sum, h, c) = alu_add_r8(operand1, operand2 as u8 as i8);
        cpu.write_data(am1, 0, sum);
        cpu.set_flags(Some(false), Some(false), Some(h), Some(c));
    } else {
        let (sum, z, h, c) = alu_add_8(operand1 as u8, operand2 as u8);
        cpu.write_data(am1, 0, sum as u16);
        cpu.set_flags(Some(z), Some(false), Some(h), Some(c));
    }

    get_cycles(opcode).0
}
