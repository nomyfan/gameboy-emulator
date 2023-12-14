use super::utils::check_condition;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, Condition};

pub(crate) fn proc_jp(
    cpu: &mut impl Cpu16,
    opcode: u8,
    cond: &Option<Condition>,
    am: &AddressingMode,
) -> u8 {
    let addr = cpu.fetch_data(am);
    if check_condition(cond.as_ref(), cpu) {
        cpu.jp(addr);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}
