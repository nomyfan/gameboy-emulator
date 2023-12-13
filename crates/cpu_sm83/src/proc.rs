use crate::{
    instruction::{get_cycles, AddressingMode, CbInstruction, Condition},
    Cpu,
};

fn check_condition<BUS: gb_shared::Memory>(cond: Option<&Condition>, cpu: &Cpu<BUS>) -> bool {
    match cond {
        None => true,
        Some(Condition::C) if cpu.flag_c() => true,
        Some(Condition::NC) if !cpu.flag_c() => true,
        Some(Condition::Z) if cpu.flag_z() => true,
        Some(Condition::NZ) if !cpu.flag_z() => true,
        _ => false,
    }
}

pub(crate) fn proc_inc<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let value = cpu.fetch_data(am);
    let value = value.wrapping_add(1);

    if opcode != 0x03 && opcode != 0x13 && opcode != 0x23 && opcode != 0x33 {
        cpu.set_flags(Some(value == 0), Some(false), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

pub(crate) fn proc_dec<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let value = cpu.fetch_data(am);
    let value = value.wrapping_sub(1);

    if opcode != 0x0B && opcode != 0x1B && opcode != 0x2B && opcode != 0x3B {
        cpu.set_flags(Some(value == 0), Some(true), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

pub(crate) fn proc_jp<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    cond: &Option<Condition>,
    am: &AddressingMode,
) -> u8 {
    let addr = cpu.fetch_data(am);
    if check_condition(cond.as_ref(), cpu) {
        cpu.pc = addr;

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

pub(crate) fn proc_jr<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    cond: &Option<Condition>,
) -> u8 {
    let r8 = cpu.fetch_data(&AddressingMode::PC1) as u8;
    if check_condition(cond.as_ref(), cpu) {
        cpu.pc = cpu.pc.wrapping_add(r8 as u16);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

pub(crate) fn proc_ld<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am1: &AddressingMode,
    am2: &AddressingMode,
) -> u8 {
    let mut operand2 = cpu.fetch_data(am2);
    let mut operand1 = cpu.fetch_data(am1);

    if opcode == 0xE0 || opcode == 0xE2 {
        // (a8) / (C)
        operand1 |= 0xFF00;
    }
    if opcode == 0xF0 || opcode == 0xF2 {
        // (a8) / (C)
        operand2 |= 0xFF00;
    }
    if opcode == 0xF8 {
        // SP+r8
        let r8 = cpu.read_pc();
        operand2 = operand2.wrapping_add(r8 as u16);
    }

    if opcode == 0xFA {
        // LD A,(a16)
        operand2 = cpu.bus_read(operand2) as u16;
    }

    cpu.write_data(am1, operand1, operand2);

    if opcode == 0x22 || opcode == 0x2A {
        // HL+
        cpu.inc_hl();
    }
    if opcode == 0x32 || opcode == 0x3A {
        // HL-
        cpu.dec_hl();
    }

    get_cycles(opcode).0
}

pub(crate) fn proc_add<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am1: &AddressingMode,
    am2: &AddressingMode,
) -> u8 {
    let operand2 = cpu.fetch_data(am2);
    let operand1 = cpu.fetch_data(am1);
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

    cpu.write_data(am1, 0, sum);
    cpu.set_flags(z, Some(false), h, c);

    get_cycles(opcode).0
}

pub(crate) fn proc_adc<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu.reg_a;
    let c = if cpu.flag_c() { 1u8 } else { 0u8 };

    let value = a.wrapping_add(data).wrapping_add(c);
    let h = (value & 0xF) + (a & 0xF) + (c & 0xF) > 0xF;
    let c = (value as u16) + (a as u16) + (c as u16) > 0xFF;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(h), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_sub<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let data = cpu.fetch_data(am);
    let a = cpu.reg_a;
    let value = a.wrapping_sub(data as u8);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) < 0;

    cpu.reg_a = value;
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_sbc<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu.reg_a;
    let c = if cpu.flag_c() { 1u8 } else { 0u8 };

    let value = a.wrapping_sub(data).wrapping_sub(c);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) - ((c & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) - (c as i16) < 0;

    cpu.reg_a = value;
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_call<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    cond: &Option<Condition>,
) -> u8 {
    let value = cpu.fetch_data(&AddressingMode::PC2);
    if check_condition(cond.as_ref(), cpu) {
        cpu.stack_push2(cpu.pc);
        cpu.pc = value;

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

pub(crate) fn proc_push<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let value = cpu.fetch_data(am);
    cpu.stack_push2(value);

    get_cycles(opcode).0
}

pub(crate) fn proc_pop<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let value = cpu.stack_pop2();
    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

pub(crate) fn proc_ret<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    cond: &Option<Condition>,
) -> u8 {
    if check_condition(cond.as_ref(), cpu) {
        cpu.pc = cpu.stack_pop2();
        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

pub(crate) fn proc_reti<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    cpu.ime = true;
    proc_ret(cpu, opcode, &None);

    get_cycles(opcode).0
}

pub(crate) fn proc_rst<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    let value: u16 = match opcode {
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
    cpu.stack_push2(cpu.pc);
    cpu.pc = value;

    get_cycles(opcode).0
}

pub(crate) fn proc_and<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let value = cpu.reg_a & operand;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(true), Some(false));

    get_cycles(opcode).0
}

pub(crate) fn proc_or<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let value = cpu.reg_a | operand;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(false), Some(false));

    get_cycles(opcode).0
}

pub(crate) fn proc_xor<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let value = cpu.reg_a ^ operand;

    cpu.reg_a = value;
    cpu.set_flags(Some(value == 0), Some(false), Some(false), Some(false));

    get_cycles(opcode).0
}

pub(crate) fn proc_di<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    cpu.ime = false;

    get_cycles(opcode).0
}

pub(crate) fn proc_ei<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    // TODO: We need another cycle to effect.
    cpu.ime = true;

    get_cycles(opcode).0
}

pub(crate) fn proc_halt<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    cpu.halted = true;

    get_cycles(opcode).0
}

pub(crate) fn proc_stop<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    cpu.stopped = true;

    get_cycles(opcode).0
}

pub(crate) fn proc_rlca<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    let value = cpu.reg_a;
    let c = (value >> 7) & 1;

    // Move the MSB(it) to the LSB(it)
    cpu.reg_a = (value << 1) | c;
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));

    get_cycles(opcode).0
}

pub(crate) fn proc_rla<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    let value = cpu.reg_a;
    let c = (value >> 7) & 1 == 1;
    let carry = if cpu.flag_c() { 1u8 } else { 0u8 };

    cpu.reg_a = (value << 1) | carry;
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_rrca<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    let value = cpu.reg_a;
    let c = value & 1;

    // Move the LSB(it) to the MSB(it)
    cpu.reg_a = (value >> 1) | (c << 7);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));

    get_cycles(opcode).0
}

pub(crate) fn proc_rra<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    let value = cpu.reg_a;
    let c = value & 1 == 1;
    let carry = if cpu.flag_c() { 1u8 } else { 0u8 };

    cpu.reg_a = (value >> 1) | (carry << 7);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_daa<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    let mut acc = 0;
    let mut c = false;
    let a = cpu.reg_a;

    let flag_h = cpu.flag_h();
    let flag_c = cpu.flag_c();
    let flag_n = cpu.flag_n();

    if flag_h || (!flag_n && (a & 0xF) > 0x09) {
        acc |= 0x06;
    }

    if flag_c || (!flag_n && a > 0x99) {
        acc |= 0x60;
        c = true;
    }

    cpu.reg_a = if flag_n { a.wrapping_sub(acc) } else { a.wrapping_add(acc) };
    cpu.set_flags(Some(cpu.reg_a == 0), None, Some(false), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_cpl<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    cpu.reg_a = !cpu.reg_a;
    cpu.set_flags(None, Some(true), Some(true), None);

    get_cycles(opcode).0
}

pub(crate) fn proc_scf<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    cpu.set_flags(None, Some(false), Some(false), Some(true));

    get_cycles(opcode).0
}

pub(crate) fn proc_ccf<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    cpu.set_flags(None, Some(false), Some(false), Some(!cpu.flag_c()));

    get_cycles(opcode).0
}

pub(crate) fn proc_cp<BUS: gb_shared::Memory>(
    cpu: &mut Cpu<BUS>,
    opcode: u8,
    am: &AddressingMode,
) -> u8 {
    let value = cpu.fetch_data(am) as u8;
    let a = cpu.reg_a;

    cpu.set_flags(Some(value == a), Some(true), Some((a & 0x0F) < (value & 0x0F)), Some(a < value));

    get_cycles(opcode).0
}

pub(crate) fn proc_cb<BUS: gb_shared::Memory>(cpu: &mut Cpu<BUS>, opcode: u8) -> u8 {
    fn decode_addressing_mode(value: u8) -> AddressingMode {
        match value {
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

    fn decode_inst(value: u8) -> CbInstruction {
        match value {
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

    let value = cpu.fetch_data(&AddressingMode::PC1) as u8;
    let am = decode_addressing_mode(value & 0b111);

    match decode_inst(value) {
        CbInstruction::RLC => {
            // 左移1位，MSB换到MLB。
            let msb = (value >> 7) & 1;
            let new_value = (value << 1) | msb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::RRC => {
            // 右移1位，MLB换到MSB。
            let mlb = value & 1;
            let new_value = (value >> 1) | (mlb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::RL => {
            // 左移1位，Flag C作为MLB。
            let msb = (value >> 7) & 1;
            let mlb = if cpu.flag_c() { 1 } else { 0 };
            let new_value = (value << 1) | mlb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::RR => {
            // 右移1位，Flag C作为MSB。
            let mlb = value & 1;
            let msb = if cpu.flag_c() { 1 } else { 0 };
            let new_value = (value >> 1) | (msb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::SLA => {
            // 左移1位。
            let msb = (value >> 7) & 1;
            let new_value = value << 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::SRA => {
            // 右移1位。Arithmetic shift.
            let mlb = value & 1;
            let new_value = (value as i8) >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::SWAP => {
            // 高低4位交换。
            let new_value = ((value & 0xF0) >> 4) | ((value & 0x0F) << 4);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(false));
        }
        CbInstruction::SRL => {
            // 右移1位。Logical shift.
            let mlb = value & 1;
            let new_value = value >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::BIT => {
            // BIT tests.
            let bit = (value - 0x40) / 8;
            cpu.set_flags(Some((value & (1 << bit)) == 0), Some(false), Some(true), None);
        }
        CbInstruction::RES => {
            // Set specific bit to be zero.
            let bit = (value - 0x80) / 8;
            let new_value = value & (!(1 << bit));

            cpu.write_data(&am, 0, new_value as u16);
        }
        CbInstruction::SET => {
            // Set specific bit to be one.
            let bit = (value - 0xC0) / 8;
            let new_value = value | (1 << bit);

            cpu.write_data(&am, 0, new_value as u16);
        }
    }

    get_cycles(opcode).0 + if let AddressingMode::Indirect_HL = am { 16 } else { 8 }
}
