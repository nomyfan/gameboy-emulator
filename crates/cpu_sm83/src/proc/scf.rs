use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_scf(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_flags(None, Some(false), Some(false), Some(true));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    #[test]
    fn scf() {
        let mut mock = MockCpu16::new();
        mock.expect_set_flags()
            .once()
            .with(eq(None), eq(Some(false)), eq(Some(false)), eq(Some(true)))
            .return_const(());

        proc_scf(&mut mock, 0x37);
    }
}
