use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::adc::alu_adc;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_adc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let (value, z, h, c) = alu_adc(cpu_fetch_a(cpu), cpu.fetch_data(am) as u8, cpu.flags().3);
    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(false), Some(h), Some(c));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;
    type Am = AddressingMode;

    const OPCODE: u8 = 0xCE;
    const AM_A: Am = Am::Direct_A;
    const AM_8: Am = Am::Eight;

    #[test]
    fn adc_flag_c_set_then_add_carry() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(1u16);
        mock.expect_fetch_data().with(eq(AM_A)).once().return_const(1u16);
        mock.expect_flags().once().return_const([false, false, false, true]);

        mock.expect_write_data().with(eq(AM_A), always(), eq(3)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
            .return_const(());

        proc_adc(&mut mock, OPCODE, &AM_8);
    }

    #[test]
    fn adc_set_flag_c() {
        let a8 = 0x11u8;
        let a = 0xF0u8;
        let ret = a.wrapping_add(a8);

        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(a8 as u16);
        mock.expect_fetch_data().with(eq(AM_A)).once().return_const(a as u16);
        mock.expect_flags().once().return_const([false, false, false, false]);

        mock.expect_write_data().with(eq(AM_A), always(), eq(ret as u16)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(true)))
            .return_const(());

        proc_adc(&mut mock, OPCODE, &AM_8);
    }

    #[test]
    fn adc_set_flag_h() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x1u16);
        mock.expect_fetch_data().with(eq(AM_A)).once().return_const(0xFu16);
        mock.expect_flags().once().return_const([false, false, false, false]);

        mock.expect_write_data().with(eq(AM_A), always(), eq(0x10)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(Some(false)), eq(Some(true)), eq(Some(false)))
            .return_const(());

        proc_adc(&mut mock, OPCODE, &AM_8);
    }

    #[test]
    fn adc_set_flag_z() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(127u16);
        mock.expect_fetch_data().with(eq(AM_A)).once().return_const((-127i8) as u8 as u16);
        mock.expect_flags().once().return_const([false, false, false, false]);

        mock.expect_write_data().with(eq(AM_A), always(), eq(0)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(true)), eq(Some(false)), eq(Some(true)), eq(Some(true)))
            .return_const(());

        proc_adc(&mut mock, OPCODE, &AM_8);
    }
}
