use crate::alu::bit::alu_bit;
use crate::alu::res::alu_res;
use crate::alu::rla::alu_rla;
use crate::alu::rlca::alu_rlca;
use crate::alu::rra::alu_rra;
use crate::alu::rrca::alu_rrca;
use crate::alu::set::alu_set;
use crate::alu::sla::alu_sla;
use crate::alu::sra::alu_sra;
use crate::alu::srl::alu_srl;
use crate::alu::swap::alu_swap;
use crate::cpu16::{Cpu16, Register, Register16, Register8};

type R8 = Register8;
type R16 = Register16;

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

pub(crate) fn proc_cb(cpu: &mut impl Cpu16) {
    fn decode_register(opcode: u8) -> Register {
        match opcode {
            0 => Register::R8(R8::B),
            1 => Register::R8(R8::C),
            2 => Register::R8(R8::D),
            3 => Register::R8(R8::E),
            4 => Register::R8(R8::H),
            5 => Register::R8(R8::L),
            6 => Register::R16(R16::HL),
            7 => Register::R8(R8::A),
            _ => unreachable!(),
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

    let cb_opcode = cpu.read_pc();
    let reg = decode_register(cb_opcode & 0b111);
    let value = match reg {
        Register::R8(reg) => cpu.read_r8(reg),
        // (HL) only
        Register::R16(reg) => cpu.bus_read(cpu.read_r16(reg)),
    };

    fn write_data(cpu: &mut impl Cpu16, reg: Register, value: u8) {
        match reg {
            Register::R8(reg) => cpu.write_r8(reg, value),
            // (HL) only
            Register::R16(reg) => cpu.bus_write(cpu.read_r16(reg), value),
        }
    }

    match decode_inst(cb_opcode) {
        CbInstruction::RLC => {
            let (new_value, c) = alu_rlca(value);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::RRC => {
            let (new_value, c) = alu_rrca(value);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::RL => {
            let (new_value, c) = alu_rla(value, cpu.flags().3);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::RR => {
            let (new_value, c) = alu_rra(value, cpu.flags().3);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::SLA => {
            let (new_value, c) = alu_sla(value);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::SRA => {
            let (new_value, c) = alu_sra(value);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::SWAP => {
            let new_value = alu_swap(value);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(false));
        }
        CbInstruction::SRL => {
            let (new_value, c) = alu_srl(value);

            write_data(cpu, reg, new_value);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(c));
        }
        CbInstruction::BIT => {
            let bit = (cb_opcode & 0b111000) >> 3;
            cpu.set_flags(Some(alu_bit(value, bit)), Some(false), Some(true), None);
        }
        CbInstruction::RES => {
            let bit = (cb_opcode & 0b111000) >> 3;
            let new_value = alu_res(value, bit);

            write_data(cpu, reg, new_value);
        }
        CbInstruction::SET => {
            let bit = (cb_opcode & 0b111000) >> 3;
            let new_value = alu_set(value, bit);

            write_data(cpu, reg, new_value);
        }
    }
}
