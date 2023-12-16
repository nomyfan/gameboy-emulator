use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_inc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let mut value = cpu.fetch_data(am);

    if (opcode & 0x03) != 0x03 {
        value = (value as u8).wrapping_add(1) as u16;
        cpu.set_flags(Some(value == 0), Some(false), Some((value & 0xF) == 0), None);
    } else {
        value = value.wrapping_add(1);
    }

    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    type Am = AddressingMode;

    #[test]
    fn inc_rr() {
        let cases = [
            (0x03u8, Am::Direct_BC, 0xFFFFu16, 0u16),
            (0x13, Am::Direct_DE, 0xFFFF, 0),
            (0x23, Am::Direct_HL, 0xFFFF, 0),
            (0x33, Am::Direct_SP, 0xFFFF, 0),
        ];

        for (opcode, am, val, ret) in cases {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am)).once().return_const(val);
            mock.expect_write_data().with(eq(am), always(), eq(ret)).once().return_const(());

            assert_eq!(proc_inc(&mut mock, opcode, &am), get_cycles(opcode).0);
        }
    }

    #[test]
    fn inc_set_flags() {
        let cases = [
            (0x04u8, Am::Direct_B, 0xFFu8, 0u16, (true, true)),
            (0x34, Am::Indirect_HL, 0xF, 0x10, (false, true)),
        ];

        for (opcode, am, val, ret, (z, h)) in cases {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am)).once().return_const(val as u16);
            mock.expect_write_data().with(eq(am), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .with(eq(Some(z)), eq(Some(false)), eq(Some(h)), eq(None))
                .once()
                .return_const(());

            assert_eq!(proc_inc(&mut mock, opcode, &am), get_cycles(opcode).0);
        }
    }
}
