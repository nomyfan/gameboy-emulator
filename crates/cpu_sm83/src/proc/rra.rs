use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::rra::alu_rra;
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rra(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let (value, c) = alu_rra(cpu_fetch_a(cpu), cpu.flags().3);

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cpu16::MockCpu16, instruction::AddressingMode as Am};
    use mockall::predicate::*;

    #[test]
    fn rra() {
        let opcode = 0x1F;
        let am_a = Am::Direct_A;

        let cases = [
            ((0b001_0001u16, true), (0b1000_1000, true)),
            ((0b001_0001, false), (0b0000_1000, true)),
            ((0b0001_0000, true), (0b1000_1000, false)),
            ((0b0001_0000, false), (0b0000_1000, false)),
        ];

        for ((in_a, in_flag_c), (out_a, out_flag_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am_a)).once().return_const(in_a);
            mock.expect_flags().once().return_const([false, false, false, in_flag_c]);
            mock.expect_write_data().with(eq(am_a), always(), eq(out_a)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(out_flag_c)))
                .return_const(());

            proc_rra(&mut mock, opcode);
        }
    }
}
