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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    use crate::cpu16::MockCpu16;

    #[test]
    fn rst() {
        let mut mock = MockCpu16::new();
        mock.expect_stack_push_pc().once().return_const(());
        mock.expect_jp().with(eq(0x08)).once().return_const(());

        assert_eq!(proc_rst(&mut mock, 0xCF), 16);
    }
}
