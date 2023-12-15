use super::utils::check_condition;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, Condition};

pub(crate) fn proc_call(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    let value = cpu.fetch_data(&AddressingMode::PC2);
    if check_condition(cond.as_ref(), cpu) {
        cpu.stack_push_pc();
        cpu.jp(value);

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
    fn call_condition_success() {
        let cases = [
            (0xD4u8, Some(Condition::NC), (false, false, false, false)),
            (0xCD, None, (false, false, false, false)),
        ];

        for (opcode, cond, flags) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            let addr = 0x1234u16;
            mock.expect_fetch_data().once().with(eq(AddressingMode::PC2)).return_const(addr);
            mock.expect_flags().times(if cond.is_none() { 0 } else { 1 }).returning(move || flags);
            mock.expect_stack_push_pc().once().return_const(());
            mock.expect_jp().once().with(eq(addr)).return_const(());

            assert_eq!(proc_call(&mut mock, opcode, &cond), get_cycles(opcode).0);
        }
    }

    #[test]
    fn call_condition_fail() {
        let opcode = 0xDCu8;
        let addr = 0x1234u16;

        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().once().with(eq(AddressingMode::PC2)).return_const(addr);
        mock.expect_flags().once().returning(|| (false, false, false, false));
        mock.expect_stack_push_pc().never();

        assert_eq!(proc_call(&mut mock, opcode, &Some(Condition::C)), get_cycles(opcode).1);
    }
}
