use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_xor(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let value = a ^ operand;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), Some(false), Some(false), Some(false));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    type AM = AddressingMode;

    #[test]
    fn xor() {
        let opcode = 0xF6u8;
        let am1 = AM::Direct_A;
        let am2 = AM::PC1;

        let cases = [(1u16, 0u16, 1u16, false), (0, 1, 1, false), (1, 1, 0, true), (0, 0, 0, true)];

        for (a, v, ret, z) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).once().return_const(a);
            mock.expect_fetch_data().with(eq(am2)).once().return_const(v);
            mock.expect_write_data().with(eq(am1), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(z)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
                .return_const(());
            proc_xor(&mut mock, opcode, &am2);
        }
    }
}
