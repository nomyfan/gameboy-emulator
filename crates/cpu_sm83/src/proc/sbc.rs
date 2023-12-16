use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_sbc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let c = if cpu.flags().3 { 1u8 } else { 0u8 };

    let value = a.wrapping_sub(data).wrapping_sub(c);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) - ((c & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) - (c as i16) < 0;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;
    type AM = AddressingMode;

    #[test]
    fn sbc_carry() {
        let opcode = 0xDE;
        let am1 = AM::Direct_A;
        let am2 = AM::PC1;

        let cases = [
            ((2u16, 1u16, true), (0u16, (true, false, false))),
            ((2, 1, false), (1, (false, false, false))),
        ];

        for ((a, v, cv), (ret, (z, h, c))) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).once().return_const(a);
            mock.expect_fetch_data().with(eq(am2)).once().return_const(v);
            mock.expect_flags().with().once().return_const((false, false, false, cv));

            mock.expect_write_data().with(eq(am1), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(z)), eq(Some(true)), eq(Some(h)), eq(Some(c)))
                .return_const(());

            proc_sbc(&mut mock, opcode, &am2);
        }
    }

    #[test]
    fn sbc_set_flags() {
        let opcode = 0xDE;
        let am1 = AM::Direct_A;
        let am2 = AM::PC1;

        let cases = [
            // z
            ((2u16, 2u16, false), (0u16, (true, false, false))),
            // h
            ((0xFE, 0xEF, false), (0xF, (false, true, false))),
            // c
            ((0xEF, 0xFF, false), (0xF0, (false, false, true))),
        ];

        for ((a, v, cv), (ret, (z, h, c))) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).once().return_const(a);
            mock.expect_fetch_data().with(eq(am2)).once().return_const(v);
            mock.expect_flags().with().once().return_const((false, false, false, cv));

            mock.expect_write_data().with(eq(am1), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(z)), eq(Some(true)), eq(Some(h)), eq(Some(c)))
                .return_const(());

            proc_sbc(&mut mock, opcode, &am2);
        }
    }
}
