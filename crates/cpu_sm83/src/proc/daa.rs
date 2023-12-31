use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::daa::alu_daa;
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

/// Decimal adjust A
pub(crate) fn proc_daa(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let (_, flag_n, flag_h, flag_c) = cpu.flags();

    let (value, z, c) = alu_daa(a, flag_n, flag_h, flag_c);

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), None, Some(false), Some(c));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use crate::instruction::AddressingMode as Am;
    use mockall::predicate::*;

    const OPCODE: u8 = 0x27;
    const AM: Am = Am::Direct_A;

    #[test]
    fn daa_flag_h_set_then_add_0x6() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM)).once().return_const(0x00u16);
        mock.expect_flags().once().return_const([false, false, true, false]);
        mock.expect_write_data().with(eq(AM), always(), eq(0x6)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(None), eq(Some(false)), eq(Some(false)))
            .return_const(());

        proc_daa(&mut mock, OPCODE);
    }

    #[test]
    fn daa_reg_a_over_0x9_then_add_0x6() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM)).once().return_const(0xAu16);
        mock.expect_flags().once().return_const([false, false, false, false]);
        mock.expect_write_data().with(eq(AM), always(), eq(0xA + 0x6)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(None), eq(Some(false)), eq(Some(false)))
            .return_const(());

        proc_daa(&mut mock, OPCODE);
    }

    #[test]
    fn daa_flag_c_set_then_add_0x60() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM)).once().return_const(0x00u16);
        mock.expect_flags().once().return_const([false, false, false, true]);
        mock.expect_write_data().with(eq(AM), always(), eq(0x60)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(None), eq(Some(false)), eq(Some(true)))
            .return_const(());

        proc_daa(&mut mock, OPCODE);
    }

    #[test]
    fn daa_reg_a_over_0x90_then_add_0x60() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM)).once().return_const(0xB0u16);
        mock.expect_flags().once().return_const([false, false, false, false]);
        mock.expect_write_data()
            .with(eq(AM), always(), eq(0xB0u8.wrapping_add(0x60) as u16))
            .once()
            .return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(None), eq(Some(false)), eq(Some(true)))
            .return_const(());

        proc_daa(&mut mock, OPCODE);
    }

    #[test]
    fn daa_flag_n_set_then_subtract() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM)).once().return_const(0xAAu16);
        mock.expect_flags().once().return_const([false, true, true, true]);
        mock.expect_write_data()
            .with(eq(AM), always(), eq(0xAAu8.wrapping_sub(0x66) as u16))
            .once()
            .return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(None), eq(Some(false)), eq(Some(true)))
            .return_const(());

        proc_daa(&mut mock, OPCODE);
    }

    #[test]
    fn daa_set_flag_z_false() {
        let cases = [
            //
            ((0x66u16, (true, true, true)), (true)),
            (((-0x66i8) as u8 as u16, (false, true, true)), (true)),
            ((0, (false, false, false)), (false)),
            ((0, (true, false, false)), (false)),
        ];

        for ((in_a, (in_n, in_h, in_c)), out_c) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM)).once().return_const(in_a);
            mock.expect_flags().once().return_const([false, in_n, in_h, in_c]);
            mock.expect_write_data().with(eq(AM), always(), eq(0)).once().return_const(());
            mock.expect_set_flags()
                .with(eq(Some(true)), eq(None), eq(Some(false)), eq(Some(out_c)))
                .return_const(());

            proc_daa(&mut mock, OPCODE);
        }
    }
}
