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
    let sum = if opcode == 0xE8 {
        operand1.wrapping_add_signed(operand2 as u8 as i8 as i16)
    } else {
        operand1.wrapping_add(operand2)
    };

    let z = if opcode == 0x09 || opcode == 0x19 || opcode == 0x29 || opcode == 0x39 {
        None
    } else if opcode == 0xE8 {
        Some(false)
    } else {
        Some(sum as u8 == 0)
    };

    let (h, c) =
        // 16 bits
        if opcode == 0x09 || opcode == 0x19 || opcode == 0x29 || opcode == 0x39 || opcode == 0xE8 {
            let h = (operand1 & 0xFFF) + (operand2 & 0xFFF) >= 0x1000;
            let c = (operand1 as u32) + (operand2 as u32) >= 0x10000;
            (Some(h), Some(c))
        } else { // 8 bits
            let h = (operand1 & 0xF) + (operand2 & 0xF) >= 0x10;
            let c = (operand1 & 0xFF) + (operand2 & 0xFF) >= 0x100;
            (Some(h), Some(c))
        };

    cpu.write_data(am1, 0, sum);
    cpu.set_flags(z, Some(false), h, c);

    get_cycles(opcode).0
}
