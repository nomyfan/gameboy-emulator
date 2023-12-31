use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_ld(
    cpu: &mut impl Cpu16,
    opcode: u8,
    am1: &AddressingMode,
    am2: &AddressingMode,
) -> u8 {
    let mut operand2 = cpu.fetch_data(am2);
    let mut operand1 = cpu.fetch_data(am1);

    if opcode == 0xE0 || opcode == 0xE2 {
        // (a8) / (C)
        operand1 |= 0xFF00;
    }

    if opcode == 0xF0 || opcode == 0xF2 {
        // (a8) / (C)
        operand2 |= 0xFF00;
    }

    if opcode == 0xF8 {
        // SP+r8
        let unsigned_r8 = cpu.fetch_data(&AddressingMode::Eight);

        let h = (operand2 & 0xF) + (unsigned_r8 & 0xF) > 0xF;
        let c = (operand2 & 0xFF) + (unsigned_r8 & 0xFF) > 0xFF;

        operand2 = operand2.wrapping_add_signed(unsigned_r8 as i8 as i16);

        cpu.set_flags(Some(false), Some(false), Some(h), Some(c));
    }

    if opcode == 0xF0 || opcode == 0xF2 || opcode == 0xFA {
        // (a8) / (C) / (a16)
        operand2 = cpu.bus_read(operand2) as u16;
    }

    if opcode == 0xE2 || opcode == 0xEA {
        cpu.write_data(&AddressingMode::Eight, operand1, operand2);
    } else {
        cpu.write_data(am1, operand1, operand2);
    }

    if opcode == 0x22 || opcode == 0x2A {
        // HL+
        cpu.inc_dec_hl(true);
    }
    if opcode == 0x32 || opcode == 0x3A {
        // HL-
        cpu.inc_dec_hl(false);
    }

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    use super::*;

    type Am = AddressingMode;

    #[test]
    fn ld() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AddressingMode::Direct_A)).return_const(0x12u16);
        mock.expect_fetch_data().with(eq(AddressingMode::Direct_B)).once().return_const(0x1234u16);
        mock.expect_write_data()
            .with(eq(AddressingMode::Direct_A), always(), function(|value| (value & 0xFF) == 0x34))
            .once()
            .return_const(());

        let am_a = Am::Direct_A;
        let am_b = Am::Direct_B;
        let opcode = 0x78;

        assert_eq!(proc_ld(&mut mock, opcode, &am_a, &am_b), get_cycles(opcode).0);
    }

    #[test]
    fn ld_hl_plus_minus() {
        let cases = [
            (0x22, (Am::Indirect_HL, 0x12u16), (Am::Direct_A, 0x34), true),
            (0x2A, (Am::Direct_A, 0x12), (Am::Indirect_HL, 0x34), true),
            (0x32, (Am::Indirect_HL, 0x12u16), (Am::Direct_A, 0x34), false),
            (0x3A, (Am::Direct_A, 0x12), (Am::Indirect_HL, 0x34), false),
        ];

        for (opcode, (am1, ret1), (am2, ret2), inc) in cases.into_iter() {
            let mut mock = MockCpu16::new();

            mock.expect_fetch_data().with(eq(am1)).return_const(ret1);
            mock.expect_fetch_data().with(eq(am2)).once().return_const(ret2);
            mock.expect_write_data()
                .with(eq(am1), always(), function(move |value| (value & 0xFF) == ret2))
                .once()
                .return_const(());
            mock.expect_inc_dec_hl().with(eq(inc)).once().return_const(());
            assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
        }
    }

    #[test]
    fn ld_sp_r8() {
        let am_hl = Am::Direct_HL;
        let am_sp = Am::Direct_SP;
        let opcode = 0xF8;

        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(am_sp)).once().return_const(2u16);
        mock.expect_fetch_data().with(eq(Am::Eight)).once().return_const((-1i8) as u16);
        mock.expect_fetch_data().with(eq(am_hl)).return_const(0x34u16);
        mock.expect_write_data().with(eq(am_hl), always(), eq(2 - 1)).return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(Some(false)), eq(Some(true)), eq(Some(true)))
            .once()
            .return_const(());

        assert_eq!(proc_ld(&mut mock, opcode, &am_hl, &am_sp), get_cycles(opcode).0);
    }

    #[test]
    fn ld_add_0xff00() {
        // (a8) / (C)
        let cases = [
            (0xE0, (Am::Eight, 0x12u8), (Am::Direct_A, 0x34u8), 0xFF12u16, 0x34u8),
            (0xE2, (Am::Direct_C, 0x12), (Am::Direct_A, 0x34), 0xFF12, 0x34),
        ];

        for (opcode, (am1, val1), (am2, val2), addr, value) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am2)).once().return_const(val2 as u16);
            mock.expect_fetch_data().with(eq(am1)).once().return_const(val1 as u16);
            mock.expect_write_data()
                .with(eq(Am::Eight), eq(addr), eq(value as u16))
                .return_const(());

            assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
        }
    }

    #[test]
    fn ld_add_0xff00_2() {
        // (a8) / (C)
        let cases = [
            (0xF0, (Am::Direct_A, 0x12u8), (Am::Eight, 0x34u8), 0xFF34, 0x34u8),
            (0xF2, (Am::Direct_A, 0x12), (Am::Direct_C, 0x34), 0xFF34, 0x34),
        ];

        for (opcode, (am1, val1), (am2, val2), addr, value) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am2)).once().return_const(val2 as u16);
            mock.expect_fetch_data().with(eq(am1)).once().return_const(val1 as u16);
            mock.expect_write_data().with(eq(am1), always(), eq(value as u16)).return_const(());
            mock.expect_bus_read().with(eq(addr)).once().return_const(value);

            assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
        }
    }
}
