use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_cpl(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    cpu_write_a(cpu, !a);
    cpu.set_flags(None, Some(true), Some(true), None);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use crate::instruction::AddressingMode as Am;
    use mockall::predicate::*;

    #[test]
    fn cpl() {
        let am_a = Am::Direct_A;
        let cases = [(1u16, 0xFEu16), (0, 0xFF)];

        for (a, ret) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am_a)).once().return_const(a);
            mock.expect_write_data().with(eq(am_a), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(None), eq(Some(true)), eq(Some(true)), eq(None))
                .return_const(());

            proc_cpl(&mut mock, 0x2F);
        }
    }
}
