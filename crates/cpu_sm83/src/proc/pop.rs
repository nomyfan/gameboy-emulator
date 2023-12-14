use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_pop(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.stack_pop2();
    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}
