use super::utils::{check_condition, cpu_stack_pop2};
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, Condition};

pub(crate) fn proc_ret(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    if check_condition(cond.as_ref(), cpu) {
        let addr = cpu_stack_pop2(cpu);
        cpu.jp(addr);
        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}
