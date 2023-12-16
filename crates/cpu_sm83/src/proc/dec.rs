use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_dec(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let mut value = cpu.fetch_data(am);

    if (opcode & 0xB) != 0xB {
        value = (value as u8).wrapping_sub(1) as u16;
        cpu.set_flags(Some(value == 0), Some(true), Some((value & 0xF) == 0), None);
    } else {
        value = value.wrapping_sub(1);
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
    fn dec_rr() {
        let cases = [
            (0x0Bu16, Am::Direct_BC, 0u16, 0xFFFFu16),
            (0x1Bu16, Am::Direct_DE, 0, 0xFFFFu16),
            (0x2Bu16, Am::Direct_HL, 0, 0xFFFFu16),
            (0x3Bu16, Am::Direct_SP, 0, 0xFFFFu16),
        ];

        for (opcode, am, val, ret) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am)).once().return_const(val);
            mock.expect_write_data().with(eq(am), always(), eq(ret)).once().return_const(());

            assert_eq!(proc_dec(&mut mock, opcode as u8, &am), get_cycles(opcode as u8).0);
        }
    }

    #[test]
    fn dec_set_flags() {
        let cases = [
            (0x05u8, Am::Direct_B, 0u8, 0xFFu16, (false, false)),
            (0x35, Am::Indirect_HL, 0x1, 0x0, (true, true)),
        ];

        for (opcode, am, val, ret, (z, h)) in cases {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am)).once().return_const(val as u16);
            mock.expect_write_data().with(eq(am), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .with(eq(Some(z)), eq(Some(true)), eq(Some(h)), eq(None))
                .once()
                .return_const(());

            assert_eq!(proc_dec(&mut mock, opcode, &am), get_cycles(opcode).0);
        }
    }
}
