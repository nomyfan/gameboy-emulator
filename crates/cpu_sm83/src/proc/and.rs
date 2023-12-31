use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_and(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let value = a & operand;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), Some(false), Some(true), Some(false));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    type Am = AddressingMode;

    #[test]
    fn and() {
        let opcode = 0xE6u8;
        let am_a = Am::Direct_A;
        let am_8 = Am::Eight;

        let cases = [(1u16, 0u16, 0u16, true), (0, 1, 0, true), (1, 1, 1, false), (0, 0, 0, true)];

        for (a, v, ret, z) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am_a)).once().return_const(a);
            mock.expect_fetch_data().with(eq(am_8)).once().return_const(v);
            mock.expect_write_data().with(eq(am_a), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(z)), eq(Some(false)), eq(Some(true)), eq(Some(false)))
                .return_const(());
            proc_and(&mut mock, opcode, &am_8);
        }
    }
}
