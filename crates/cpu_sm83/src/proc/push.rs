use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_push(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am);
    cpu.stack_push2(value);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    use crate::cpu16::MockCpu16;
    use crate::instruction::AddressingMode as AM;

    #[test]
    fn push() {
        let addr = 0x1212u16;
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM::Direct_AF)).once().return_const(addr);
        mock.expect_stack_push().with(eq(0x12)).times(2).return_const(());

        assert_eq!(proc_push(&mut mock, 0xF5, &AM::Direct_AF), 16);
    }
}
