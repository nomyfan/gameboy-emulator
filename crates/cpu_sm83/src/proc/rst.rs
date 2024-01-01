use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rst(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let addrs = [0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38];
    // 11xxx111
    let addr = addrs[((opcode & 0x38) >> 3) as usize];
    cpu.stack_push_pc();
    cpu.jp(addr);

    get_cycles(opcode).0
}
