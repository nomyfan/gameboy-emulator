use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_ccf(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_flags(None, Some(false), Some(false), Some(!cpu.flags().3));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::proc_ccf;
    use crate::cpu16::MockCpu16;
    use crate::instruction::get_cycles;
    use mockall::predicate::*;

    #[test]
    fn ccf() {
        let mut mock = MockCpu16::new();

        let cases = [true, false];
        for c in cases.into_iter() {
            mock.expect_flags().with().once().return_const((false, false, false, c));
            mock.expect_set_flags()
                .with(eq(None), eq(Some(false)), eq(Some(false)), eq(Some(!c)))
                .once()
                .return_const(());

            assert_eq!(proc_ccf(&mut mock, 0x3F), get_cycles(0x3F).0);
        }
    }
}
