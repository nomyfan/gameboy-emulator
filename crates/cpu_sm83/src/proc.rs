use crate::{
    instructions::{AddressingMode, Condition, Instruction, Register},
    Cpu,
};

fn check_condition<BUS>(cond: Option<&Condition>, cpu: &Cpu<BUS>) -> bool
where
    BUS: io::IO,
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
    BUS: io::IO,
{
    // Register only
    let addr = inst.operand1.as_ref().unwrap();
    let value = cpu.fetch_data(addr);
    let value = value + 1;

    if inst.opcode != 0x03 && inst.opcode != 0x13 && inst.opcode != 0x23 && inst.opcode != 0x33 {
        cpu.set_flags(Some(value == 0), Some(false), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(addr, 0, value);
}

pub(crate) fn proc_dec<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    // Register only
    let addr = inst.operand1.as_ref().unwrap();
    let value = cpu.fetch_data(addr);
    let value = value - 1;

    if inst.opcode != 0x0B && inst.opcode != 0x1B && inst.opcode != 0x2B && inst.opcode != 0x3B {
        cpu.set_flags(Some(value == 0), Some(true), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(addr, 0, value);
}

pub(crate) fn proc_jp<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    let a16 = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.pc = a16;
    }
}

pub(crate) fn proc_jr<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    let r8 = cpu.fetch_data(inst.operand1.as_ref().unwrap()) as u8;
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.pc += r8 as u16;
    }
}

pub(crate) fn proc_ld<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    let mut operand2 = cpu.fetch_data(inst.operand2.as_ref().unwrap());
    let mut operand1 = cpu.fetch_data(inst.operand1.as_ref().unwrap());

    let opcode = inst.opcode;
    if opcode == 0xE0 || opcode == 0xE2 {
        // (a8), (C)
        operand1 = 0xFF00 | operand1;
    }
    if opcode == 0xF0 || opcode == 0xF2 {
        // (a8), (C)
        operand2 = 0xFF00 | operand2;
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
    BUS: io::IO,
{
    let opcode = inst.opcode;
    let operand2 = cpu.fetch_data(inst.operand2.as_ref().unwrap());
    let operand1 = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    let sum = operand1 + operand2;

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

pub(crate) fn proc_sub<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    let data = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    let value = cpu.reg_a - data as u8;

    let z = value == 0;
    let h = (cpu.reg_a as i16 & 0xF) - (data as i16 & 0xF) < 0;
    let c = (cpu.reg_a as i16) - (data as i16) < 0;

    cpu.write_data(&AddressingMode::Direct(Register::A), 0, value as u16);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));
}

pub(crate) fn proc_call<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    let value = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.stack_push2(value);
        cpu.pc = value;
    }
}

pub(crate) fn proc_push<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    let value = cpu.fetch_data(inst.operand1.as_ref().unwrap());
    cpu.stack_push2(value);
}

pub(crate) fn proc_pop<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    let value = cpu.stack_pop2();
    cpu.write_data(inst.operand1.as_ref().unwrap(), 0, value);
}

pub(crate) fn proc_ret<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    if check_condition(inst.cond.as_ref(), cpu) {
        cpu.pc = cpu.stack_pop2();
    }
}

pub(crate) fn proc_reti<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
{
    proc_ret(cpu, inst);
    cpu.interrupt_master_enable = true;
}

pub(crate) fn proc_rst<BUS>(cpu: &mut Cpu<BUS>, inst: &Instruction)
where
    BUS: io::IO,
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
