use crate::{
    instructions::{AddressingMode, CbInstructionType, Condition, Instruction, Register},
    Cpu,
};

fn check_condition<BUS>(cond: Option<&Condition>, cpu: &Cpu<BUS>) -> bool
where
    BUS: gb_shared::Memory,
{
    match cond {
        None => true,
        Some(Condition::C) if cpu.flag_c() => true,
        Some(Condition::NC) if !cpu.flag_c() => true,
        Some(Condition::Z) if cpu.flag_z() => true,
        Some(Condition::NZ) if !cpu.flag_z() => true,
        _ => false,
    }
}

pub(crate) fn proc_inc<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    // Register only
    let addr = inst.operand1.as_ref().unwrap();
    let value = cpu.fetch_data(addr);
    let value = value.wrapping_add(1);

    if inst.opcode != 0x03 && inst.opcode != 0x13 && inst.opcode != 0x23 && inst.opcode != 0x33 {
        cpu.set_flags(Some(value == 0), Some(false), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(addr, 0, value);
}

pub(crate) fn proc_dec<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    // Register only
    let addr = inst.operand1.as_ref().unwrap();
    let value = cpu.fetch_data(addr);
    let value = value.wrapping_sub(1);

    if inst.opcode != 0x0B && inst.opcode != 0x1B && inst.opcode != 0x2B && inst.opcode != 0x3B {
        cpu.set_flags(Some(value == 0), Some(true), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(addr, 0, value);
}

pub(crate) fn proc_jp<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let addr = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.pc = addr;
    }
}

pub(crate) fn proc_jr<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let addr = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.pc += addr as u16;
    }
}

pub(crate) fn proc_ld<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let mut operand2 = cpu.fetch_data(inst.operand2.as_ref().unwrap());
    let mut operand1 = cpu.fetch_data(inst.operand1.as_ref().unwrap());

    let opcode = inst.opcode;
    if opcode == 0xE0 || opcode == 0xE2 {
        // (a8), (C)
        operand1 |= 0xFF00;
    }
    if opcode == 0xF0 || opcode == 0xF2 {
        // (a8), (C)
        operand2 |= 0xFF00;
    }
    if opcode == 0xF8 {
        // SP+r8
        let r8 = cpu.read_pc();
        operand2 += r8 as u16;
    }

    cpu.write_data(inst.operand1.as_ref().unwrap(), operand1, operand2);

    if opcode == 0x22 || opcode == 0x2A {
        // HL+
        cpu.inc_hl();
    }
    if opcode == 0x32 || opcode == 0x3A {
        // HL-
        cpu.dec_hl();
    }
}

pub(crate) fn proc_add<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let opcode = inst.opcode;
    let operand2 = cpu.fetch_data(inst.operand2.as_ref().unwrap());
    let operand1 = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    let sum = operand1.wrapping_add(operand2);

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

    cpu.write_data(inst.operand1.as_ref().unwrap(), 0, sum);
    cpu.set_flags(z, Some(false), h, c);
}

pub(crate) fn proc_adc<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let data = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    let a = cpu.reg_a;
    let c = if cpu.flag_c() { 1u8 } else { 0u8 };

    let value = a.wrapping_add(data).wrapping_add(c);
    let h = (value & 0xF) + (a & 0xF) + (c & 0xF) > 0xF;
    let c = (value as u16) + (a as u16) + (c as u16) > 0xFF;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(h), Some(c));
}

pub(crate) fn proc_sub<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let data = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    let a = cpu.reg_a;
    let value = a.wrapping_sub(data as u8);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) < 0;

    cpu.reg_a = value;
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));
}

pub(crate) fn proc_sbc<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let data = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    let a = cpu.reg_a;
    let c = if cpu.flag_c() { 1u8 } else { 0u8 };

    let value = a.wrapping_sub(data).wrapping_sub(c);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) - ((c & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) - (c as i16) < 0;

    cpu.reg_a = value;
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));
}

pub(crate) fn proc_call<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.stack_push2(value);
        cpu.pc = value;
    }
}

pub(crate) fn proc_push<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    cpu.stack_push2(value);
}

pub(crate) fn proc_pop<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.stack_pop2();
    cpu.write_data(inst.operand1.as_ref().unwrap(), 0, value);
}

pub(crate) fn proc_ret<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.pc = cpu.stack_pop2();
    }
}

pub(crate) fn proc_reti<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    cpu.interrupt_master_enable = true;
    proc_ret(cpu, inst);
}

pub(crate) fn proc_rst<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let value: u16 = match inst.opcode {
        0xC7 => 0x00,
        0xCF => 0x08,
        0xD7 => 0x10,
        0xDF => 0x18,
        0xE7 => 0x20,
        0xEF => 0x28,
        0xF7 => 0x30,
        0xFF => 0x38,
        _ => unreachable!(),
    };
    cpu.stack_push2(value);
    cpu.pc = value;
}

pub(crate) fn proc_and<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let operand = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    let value = cpu.reg_a & operand;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(true), Some(false));
}

pub(crate) fn proc_or<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let operand = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    let value = cpu.reg_a | operand;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(false), Some(false));
}

pub(crate) fn proc_xor<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let operand = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    let value = cpu.reg_a ^ operand;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(false), Some(false));
}

pub(crate) fn proc_di<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    cpu.interrupt_master_enable = false;
}

pub(crate) fn proc_ei<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    cpu.interrupt_master_enable = true;
}

pub(crate) fn proc_halt<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    cpu.halted = true;
}

pub(crate) fn proc_stop<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    cpu.stopped = true;
}

pub(crate) fn proc_rlca<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.reg_a;
    let c = (value >> 7) & 1;

    // Move the MSB(it) to the LSB(it)
    cpu.reg_a = (value << 1) | c;
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));
}

pub(crate) fn proc_rla<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.reg_a;
    let c = (value >> 7) & 1 == 1;
    let carry = if cpu.flag_c() { 1u8 } else { 0u8 };

    cpu.reg_a = (value << 1) | carry;
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));
}

pub(crate) fn proc_rrca<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.reg_a;
    let c = value & 1;

    // Move the LSB(it) to the MSB(it)
    cpu.reg_a = (value >> 1) | (c << 7);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));
}

pub(crate) fn proc_rra<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.reg_a;
    let c = value & 1 == 1;
    let carry = if cpu.flag_c() { 1u8 } else { 0u8 };

    cpu.reg_a = (value >> 1) | (carry << 7);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));
}

pub(crate) fn proc_daa<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    let mut acc = 0;
    let mut c = false;
    let a = cpu.reg_a;

    let flag_h = cpu.flag_h();
    let flag_c = cpu.flag_c();
    let flag_n = cpu.flag_n();

    if flag_h || (!flag_n && (a & 0xF) > 9) {
        acc |= 0x06;
    }

    if flag_c || (!flag_n && a > 99) {
        acc |= 0x60;
        c = true;
    }

    cpu.reg_a = if flag_n { a.wrapping_sub(acc) } else { a.wrapping_add(acc) };
    cpu.set_flags(Some(cpu.reg_a == 0), None, Some(false), Some(c));
}

pub(crate) fn proc_cpl<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    cpu.reg_a = !cpu.reg_a;
    cpu.set_flags(None, Some(true), Some(true), None);
}

pub(crate) fn proc_scf<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    cpu.set_flags(None, Some(false), Some(false), Some(true));
}

pub(crate) fn proc_ccf<BUS>(cpu: &mut Cpu<BUS>)
where
    BUS: gb_shared::Memory,
{
    cpu.set_flags(None, Some(false), Some(false), Some(!cpu.flag_c()));
}

pub(crate) fn proc_cp<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    let value = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    let a = cpu.reg_a;

    cpu.set_flags(Some(value == a), Some(true), Some((a & 0x0F) < (value & 0x0F)), Some(a < value));
}

pub(crate) fn proc_cb<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: gb_shared::Memory,
{
    fn decode_addressing_mode(value: u8) -> AddressingMode {
        match value {
            0 => AddressingMode::Direct(Register::B),
            1 => AddressingMode::Direct(Register::C),
            2 => AddressingMode::Direct(Register::D),
            3 => AddressingMode::Direct(Register::E),
            4 => AddressingMode::Direct(Register::H),
            5 => AddressingMode::Direct(Register::L),
            6 => AddressingMode::Indirect(Register::HL),
            7 => AddressingMode::Direct(Register::A),
            _ => unreachable!("Only B,C,D,E,H,L,HL,A are valid for CB instruction."),
        }
    }

    fn decode_inst(value: u8) -> CbInstructionType {
        match value {
            0x00..=0x07 => CbInstructionType::RLC,
            0x08..=0x0F => CbInstructionType::RRC,
            0x10..=0x17 => CbInstructionType::RL,
            0x18..=0x1F => CbInstructionType::RR,
            0x20..=0x27 => CbInstructionType::SLA,
            0x28..=0x2F => CbInstructionType::SRA,
            0x30..=0x37 => CbInstructionType::SWAP,
            0x38..=0x3F => CbInstructionType::SRL,
            0x40..=0x7F => CbInstructionType::BIT,
            0x80..=0xBF => CbInstructionType::RES,
            0xC0..=0xFF => CbInstructionType::SET,
        }
    }

    let value = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    let am = decode_addressing_mode(value & 0b111);

    match decode_inst(value) {
        CbInstructionType::RLC => {
            // 左移1位，MSB换到MLB。
            let msb = (value >> 7) & 1;
            let new_value = (value << 1) | msb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstructionType::RRC => {
            // 右移1位，MLB换到MSB。
            let mlb = value & 1;
            let new_value = (value >> 1) | (mlb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstructionType::RL => {
            // 左移1位，Flag C作为MLB。
            let msb = (value >> 7) & 1;
            let mlb = if cpu.flag_c() { 1 } else { 0 };
            let new_value = (value << 1) | mlb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstructionType::RR => {
            // 右移1位，Flag C作为MSB。
            let mlb = value & 1;
            let msb = if cpu.flag_c() { 1 } else { 0 };
            let new_value = (value >> 1) | (msb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstructionType::SLA => {
            // 左移1位。
            let msb = (value >> 7) & 1;
            let new_value = value << 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstructionType::SRA => {
            // 右移1位。Arithmetic shift.
            let mlb = value & 1;
            let new_value = (value as i8) >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstructionType::SWAP => {
            // 高低4位交换。
            let new_value = ((value & 0xF0) >> 4) | ((value & 0x0F) << 4);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(false));
        }
        CbInstructionType::SRL => {
            // 右移1位。Logical shift.
            let mlb = value & 1;
            let new_value = value >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstructionType::BIT => {
            // BIT tests.
            let bit = (value - 0x40) / 8;
            cpu.set_flags(Some((value & (1 << bit)) == 0), Some(false), Some(true), None);
        }
        CbInstructionType::RES => {
            // Set specific bit to be zero.
            let bit = (value - 0x80) / 8;
            let new_value = value & (!(1 << bit));

            cpu.write_data(&am, 0, new_value as u16);
        }
        CbInstructionType::SET => {
            // Set specific bit to be one.
            let bit = (value - 0xC0) / 8;
            let new_value = value | (1 << bit);

            cpu.write_data(&am, 0, new_value as u16);
        }
    }
}
