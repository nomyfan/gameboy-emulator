use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, CbInstruction};

pub(crate) fn proc_cb(cpu: &mut impl Cpu16, _opcode: u8) -> u8 {
    fn decode_addressing_mode(opcode: u8) -> AddressingMode {
        match opcode {
            0 => AddressingMode::Direct_B,
            1 => AddressingMode::Direct_C,
            2 => AddressingMode::Direct_D,
            3 => AddressingMode::Direct_E,
            4 => AddressingMode::Direct_H,
            5 => AddressingMode::Direct_L,
            6 => AddressingMode::Indirect_HL,
            7 => AddressingMode::Direct_A,
            _ => unreachable!("Only B,C,D,E,H,L,HL,A are valid for CB instruction."),
        }
    }

    fn decode_inst(opcode: u8) -> CbInstruction {
        match opcode {
            0x00..=0x07 => CbInstruction::RLC,
            0x08..=0x0F => CbInstruction::RRC,
            0x10..=0x17 => CbInstruction::RL,
            0x18..=0x1F => CbInstruction::RR,
            0x20..=0x27 => CbInstruction::SLA,
            0x28..=0x2F => CbInstruction::SRA,
            0x30..=0x37 => CbInstruction::SWAP,
            0x38..=0x3F => CbInstruction::SRL,
            0x40..=0x7F => CbInstruction::BIT,
            0x80..=0xBF => CbInstruction::RES,
            0xC0..=0xFF => CbInstruction::SET,
        }
    }

    let cb_opcode = cpu.fetch_data(&AddressingMode::PC1) as u8;
    let am = decode_addressing_mode(cb_opcode & 0b111);

    match decode_inst(cb_opcode) {
        CbInstruction::RLC => {
            // 左移1位，MSB换到MLB。
            let msb = (cb_opcode >> 7) & 1;
            let new_value = (cb_opcode << 1) | msb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::RRC => {
            // 右移1位，MLB换到MSB。
            let mlb = cb_opcode & 1;
            let new_value = (cb_opcode >> 1) | (mlb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::RL => {
            // 左移1位，Flag C作为MLB。
            let msb = (cb_opcode >> 7) & 1;
            let mlb = if cpu.flags().3 { 1 } else { 0 };
            let new_value = (cb_opcode << 1) | mlb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::RR => {
            // 右移1位，Flag C作为MSB。
            let mlb = cb_opcode & 1;
            let msb = if cpu.flags().3 { 1 } else { 0 };
            let new_value = (cb_opcode >> 1) | (msb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::SLA => {
            // 左移1位。
            let msb = (cb_opcode >> 7) & 1;
            let new_value = cb_opcode << 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::SRA => {
            // 右移1位。Arithmetic shift.
            let mlb = cb_opcode & 1;
            let new_value = (cb_opcode as i8) >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::SWAP => {
            // 高低4位交换。
            let new_value = ((cb_opcode & 0xF0) >> 4) | ((cb_opcode & 0x0F) << 4);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(false));
        }
        CbInstruction::SRL => {
            // 右移1位。Logical shift.
            let mlb = cb_opcode & 1;
            let new_value = cb_opcode >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::BIT => {
            // BIT tests.
            let bit = (cb_opcode - 0x40) / 8;
            cpu.set_flags(Some((cb_opcode & (1 << bit)) == 0), Some(false), Some(true), None);
        }
        CbInstruction::RES => {
            // Set specific bit to be zero.
            let bit = (cb_opcode - 0x80) / 8;
            let new_value = cb_opcode & (!(1 << bit));

            cpu.write_data(&am, 0, new_value as u16);
        }
        CbInstruction::SET => {
            // Set specific bit to be one.
            let bit = (cb_opcode - 0xC0) / 8;
            let new_value = cb_opcode | (1 << bit);

            cpu.write_data(&am, 0, new_value as u16);
        }
    }

    get_cycles(0xCB).0 + if let AddressingMode::Indirect_HL = am { 16 } else { 8 }
}
