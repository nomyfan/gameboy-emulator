use super::utils::check_condition;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, Condition};

pub(crate) fn proc_call(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    let value = cpu.fetch_data(&AddressingMode::Sixteen);
    if check_condition(cond.as_ref(), cpu) {
        cpu.stack_push_pc();
        cpu.jp(value);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}
