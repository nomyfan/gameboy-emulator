use super::res;
use super::rl;
use super::rlc;
use super::rr;
use super::rrc;
use super::set;
use super::sla;
use super::sra;
use super::srl;
use super::swap;
use crate::Cpu;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum CbInstruction {
    /// Rotate Left Circular
    RLC,
    /// Rotate Right Circular
    RRC,
    /// Rotate Left
    RL,
    /// Rotate Right
    RR,
    /// Shift Left Arithmetic
    SLA,
    /// Shift Right Arithmetic
    SRA,
    SWAP,
    /// Shift Right Logical
    SRL,
    /// Test whether nth is 0.
    BIT,
    /// Reset nth bit to 0
    RES,
    /// Set nth bit to 1
    SET,
}

pub(crate) fn cb<BUS: gb_shared::Bus>(cpu: &mut Cpu<BUS>) {
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

    let cb_opcode = cpu.read_pc();
    let value = match cb_opcode & 0b111 {
        0 => cpu.reg_b,
        1 => cpu.reg_c,
        2 => cpu.reg_d,
        3 => cpu.reg_e,
        4 => cpu.reg_h,
        5 => cpu.reg_l,
        6 => cpu.bus_read(cpu.hl()),
        7 => cpu.reg_a,
        _ => unreachable!(),
    };

    let write_data = |cpu: &mut Cpu<BUS>, value: u8| match cb_opcode & 0b111 {
        0 => cpu.reg_b = value,
        1 => cpu.reg_c = value,
        2 => cpu.reg_d = value,
        3 => cpu.reg_e = value,
        4 => cpu.reg_h = value,
        5 => cpu.reg_l = value,
        6 => cpu.bus_write(cpu.hl(), value),
        7 => cpu.reg_a = value,
        _ => unreachable!(),
    };

    match decode_inst(cb_opcode) {
        CbInstruction::RLC => {
            let (new_value, c) = rlc(value);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::RRC => {
            let (new_value, c) = rrc(value);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::RL => {
            let (new_value, c) = rl(value, cpu.flags().3);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::RR => {
            let (new_value, c) = rr(value, cpu.flags().3);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::SLA => {
            let (new_value, c) = sla(value);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::SRA => {
            let (new_value, c) = sra(value);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::SWAP => {
            let new_value = swap(value);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(false));
        }
        CbInstruction::SRL => {
            let (new_value, c) = srl(value);

            write_data(cpu, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::BIT => {
            let bit = (cb_opcode & 0b111000) >> 3;
            cpu.set_flags(Some(!super::bit(value, bit)), Some(false), Some(true), None);
        }
        CbInstruction::RES => {
            let bit = (cb_opcode & 0b111000) >> 3;
            let new_value = res(value, bit);

            write_data(cpu, new_value);
        }
        CbInstruction::SET => {
            let bit = (cb_opcode & 0b111000) >> 3;
            let new_value = set(value, bit);

            write_data(cpu, new_value);
        }
    }
}
