use super::utils::check_condition;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, Condition};

pub(crate) fn proc_ret(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    if check_condition(cond.as_ref(), cpu) {
        let addr = cpu.stack_pop2();
        cpu.jp(addr);
        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use crate::instruction::get_cycles;
    use mockall::predicate::*;

    #[test]
    fn ret_condition_success() {
        let cases = [
            (0xD0u8, Some(Condition::NC), (false, false, false, false)),
            (0xC9, None, (false, false, false, false)),
        ];

        for (opcode, cond, flags) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            let addr = 0x1234u16;
            mock.expect_stack_pop2().once().return_const(addr);
            mock.expect_flags().times(if cond.is_none() { 0 } else { 1 }).return_const(flags);
            mock.expect_jp().once().with(eq(addr)).return_const(());

            assert_eq!(proc_ret(&mut mock, opcode, &cond), get_cycles(opcode).0);
        }
    }

    #[test]
    fn ret_condition_fail() {
        let opcode = 0xC8u8;

        let mut mock = MockCpu16::new();
        mock.expect_flags().once().returning(|| (false, false, false, false));

        assert_eq!(proc_ret(&mut mock, opcode, &Some(Condition::Z)), get_cycles(opcode).1);
    }
}
