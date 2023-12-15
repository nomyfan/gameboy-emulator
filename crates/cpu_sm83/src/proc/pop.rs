use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_pop(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.stack_pop2();
    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::{always, eq};

    use crate::cpu16::MockCpu16;
    use crate::instruction::AddressingMode as AM;

    #[test]
    fn pop() {
        let addr = 0x1212u16;
        let mut mock = MockCpu16::new();
        mock.expect_stack_pop().times(2).return_const(0x12);
        mock.expect_write_data()
            .once()
            .with(eq(AM::Direct_BC), always(), eq(addr))
            .return_const(());

        assert_eq!(proc_pop(&mut mock, 0xC1, &AM::Direct_BC), 12);
    }
}
