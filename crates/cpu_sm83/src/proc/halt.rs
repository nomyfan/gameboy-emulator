use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_halt(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_halt(true);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::proc_halt;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    #[test]
    fn halt() {
        let mut mock = MockCpu16::new();
        mock.expect_set_halt().once().with(eq(true)).return_const(());

        assert_eq!(proc_halt(&mut mock, 0xF3), 4);
    }
}
