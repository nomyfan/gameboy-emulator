use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_daa(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let mut acc = 0;
    let mut c = false;
    let a = cpu_fetch_a(cpu);

    let (_, flag_n, flag_h, flag_c) = cpu.flags();

    if flag_h || (!flag_n && (a & 0xF) > 0x09) {
        acc |= 0x06;
    }

    if flag_c || (!flag_n && a > 0x99) {
        acc |= 0x60;
        c = true;
    }

    let value = if flag_n { a.wrapping_sub(acc) } else { a.wrapping_add(acc) };
    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), None, Some(false), Some(c));

    get_cycles(opcode).0
}
