use super::utils::cpu_fetch_a;
use crate::alu::cp::alu_cp;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_cp(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let (z, h, c) = alu_cp(cpu_fetch_a(cpu), cpu.fetch_data(am) as u8);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use crate::cpu16::MockCpu16;

    use super::*;
    use mockall::predicate::*;
    type Am = AddressingMode;

    #[test]
    fn cp() {
        let opcode = 0xFEu8;
        let am_8 = Am::Eight;
        let cases = [
            (0x11u16, 0x21u16, (false, false, true)),
            (1, 1, (true, false, false)),
            (2, 1, (false, false, false)),
            (1, 3, (false, true, true)),
        ];

        for (a, v, (z, h, c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(Am::Direct_A)).once().return_const(a);
            mock.expect_fetch_data().with(eq(am_8)).once().return_const(v);
            mock.expect_set_flags()
                .once()
                .with(eq(Some(z)), eq(Some(true)), eq(Some(h)), eq(Some(c)))
                .return_const(());

            proc_cp(&mut mock, opcode, &am_8);
        }
    }
}
