use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_push(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am);
    cpu.stack_push2(value);

    get_cycles(opcode).0
}
