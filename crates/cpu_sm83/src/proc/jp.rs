use super::utils::check_condition;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, Condition};

pub(crate) fn proc_jp(
    cpu: &mut impl Cpu16,
    opcode: u8,
    cond: &Option<Condition>,
    am: &AddressingMode,
) -> u8 {
    let addr = cpu.fetch_data(am);
    if check_condition(cond.as_ref(), cpu) {
        cpu.jp(addr);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

#[cfg(test)]
mod tests {
    use super::proc_jp;
    use crate::cpu16::MockCpu16;
    use crate::instruction::{get_cycles, AddressingMode as AM, Condition};
    use mockall::predicate::*;

    #[test]
    fn jp_condition_success() {
        let cases = [
            (0xE9u8, AM::Indirect_HL, None, (false, false, false, false)),
            (0xD2, AM::PC2, Some(Condition::NC), (false, true, true, false)),
        ];

        for (opcode, am, cond, (z, n, h, c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am)).once().returning(|_| 1);
            mock.expect_flags()
                .times(if cond.is_none() { 0 } else { 1 })
                .returning(move || (z, n, h, c));
            mock.expect_jp().with(eq(1)).once().returning(|_| {});

            assert_eq!(proc_jp(&mut mock, opcode, &cond, &am), get_cycles(opcode).0)
        }
    }

    #[test]
    fn jp_condition_fail() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM::PC2)).once().returning(|_| 1);
        mock.expect_flags().once().returning(move || (false, true, true, true));

        assert_eq!(proc_jp(&mut mock, 0xCA, &Some(Condition::Z), &AM::PC2), get_cycles(0xCA).1)
    }
}
