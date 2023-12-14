use super::utils::check_condition;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, Condition};

pub(crate) fn proc_jr(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    let unsigned_r8 = cpu.fetch_data(&AddressingMode::PC1) as u8;
    if check_condition(cond.as_ref(), cpu) {
        cpu.jr(unsigned_r8 as i8);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}
