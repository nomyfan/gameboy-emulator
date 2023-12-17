use crate::cpu16::Cpu16;
use crate::instruction::get_cycles;

pub(crate) fn proc_stop(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.stop();

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::proc_stop;
    use crate::cpu16::MockCpu16;

    #[test]
    fn stop() {
        let mut mock = MockCpu16::new();
        mock.expect_stop().once().return_const(());

        assert_eq!(proc_stop(&mut mock, 0x10), 4);
    }
}
