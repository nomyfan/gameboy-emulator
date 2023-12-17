use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_rrca(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = a & 1;

    // Move the LSB(it) to the MSB(it)
    cpu_write_a(cpu, (a >> 1) | (c << 7));
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use crate::instruction::AddressingMode as Am;
    use mockall::predicate::*;

    #[test]
    fn rrca() {
        let opcode = 0x0F;
        let am_a = Am::Direct_A;

        let cases = [
            //
            ((0b0001_0001u16), (0b1000_1000, true)),
            ((0b1001_0000), (0b0100_1000, false)),
        ];

        for (in_a, (out_a, out_flag_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am_a)).once().return_const(in_a);
            mock.expect_write_data().with(eq(am_a), always(), eq(out_a)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(out_flag_c)))
                .return_const(());

            proc_rrca(&mut mock, opcode);
        }
    }
}
