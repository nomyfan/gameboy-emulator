use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_ei(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    // TODO: We need another cycle to effect.
    cpu.set_ime(true);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::proc_ei;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    #[test]
    fn ei() {
        let mut mock = MockCpu16::new();
        mock.expect_set_ime().once().with(eq(true)).return_const(());

        assert_eq!(proc_ei(&mut mock, 0x76), 4);
    }
}
