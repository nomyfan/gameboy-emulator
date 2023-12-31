use super::utils::check_condition;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, Condition};

pub(crate) fn proc_jr(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    let unsigned_r8 = cpu.fetch_data(&AddressingMode::Eight) as u8;
    if check_condition(cond.as_ref(), cpu) {
        cpu.jr(unsigned_r8 as i8);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

#[cfg(test)]
mod tests {
    use super::proc_jr;
    use crate::cpu16::MockCpu16;
    use crate::instruction::{get_cycles, AddressingMode as Am, Condition};
    use mockall::predicate::*;

    #[test]
    fn jr_condition_success() {
        let cases = [
            (0x18u8, None, (false, false, false, false)),
            (0x20, Some(Condition::NZ), (false, true, true, true)),
        ];

        for (opcode, cond, (z, n, h, c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(Am::Eight)).once().return_const((-1i8) as u16);
            mock.expect_flags()
                .times(if cond.is_none() { 0 } else { 1 })
                .returning(move || (z, n, h, c));
            mock.expect_jr().with(eq(-1)).once().return_const(());

            assert_eq!(proc_jr(&mut mock, opcode, &cond), get_cycles(opcode).0)
        }
    }

    #[test]
    fn jr_condition_fail() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(Am::Eight)).once().return_const(1u16);
        mock.expect_flags().once().returning(move || (false, true, true, true));

        assert_eq!(proc_jr(&mut mock, 0x28, &Some(Condition::Z)), get_cycles(0x28).1)
    }
}
