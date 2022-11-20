use crate::{
    instructions::{Condition, Instruction},
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
        cpu.set_flags(
            Some(value == 0),
            Some(false),
            Some((value & 0xF) == 0),
            None,
        );
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
    todo!()
}
