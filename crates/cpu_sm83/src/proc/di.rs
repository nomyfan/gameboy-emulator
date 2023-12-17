use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_di(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_ime(false);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::proc_di;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    #[test]
    fn di() {
        let mut mock = MockCpu16::new();
        mock.expect_set_ime().once().with(eq(false)).return_const(());

        assert_eq!(proc_di(&mut mock, 0xF3), 4);
    }
}
