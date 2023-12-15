use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_add(
    cpu: &mut impl Cpu16,
    opcode: u8,
    am1: &AddressingMode,
    am2: &AddressingMode,
) -> u8 {
    let operand2 = cpu.fetch_data(am2);
    let operand1 = cpu.fetch_data(am1);

    let is_rr = (opcode & 0x09) == 0x09;
    let is_sp_r8 = opcode == 0xE8;
    let is_16bits = is_rr || is_sp_r8;

    let sum = if is_sp_r8 {
        operand1.wrapping_add_signed(operand2 as u8 as i8 as i16)
    } else if is_16bits {
        operand1.wrapping_add(operand2)
    } else {
        (operand1 as u8).wrapping_add(operand2 as u8) as u16
    };

    let z = if is_rr {
        None
    } else if is_sp_r8 {
        Some(false)
    } else {
        Some(sum as u8 == 0)
    };

    let (h, c) =
        // 16 bits
        if is_16bits {
            let h = (operand1 & 0xFFF) + (operand2 & 0xFFF) > 0xFFF;
            let c = (operand1 as u32) + (operand2 as u32) > 0xFFFF;
            (Some(h), Some(c))
        } else { // 8 bits
            let h = (operand1 & 0xF) + (operand2 & 0xF) > 0xF;
            let c = (operand1 & 0xFF) + (operand2 & 0xFF) > 0xFF;
            (Some(h), Some(c))
        };

    cpu.write_data(am1, 0, sum);
    cpu.set_flags(z, Some(false), h, c);

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    type AM = AddressingMode;

    #[test]
    fn add_sp_r8() {
        let opcode = 0xE8u8;
        let am1 = AM::Direct_SP;
        let am2 = AM::PC1;

        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(am1)).return_const(1u16);
        mock.expect_fetch_data().with(eq(am2)).return_const((-1i8) as u16);
        mock.expect_write_data().with(eq(am1), always(), eq(0)).return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(Some(false)), eq(Some(true)), eq(Some(true)))
            .return_const(());

        assert_eq!(proc_add(&mut mock, opcode, &am1, &am2), 16);
    }

    #[test]
    fn add_r() {
        let cases = [
            (0x85u8, AM::Direct_A, AM::Direct_L),
            (0x86u8, AM::Direct_A, AM::Indirect_HL),
            (0xC5u8, AM::Direct_A, AM::PC1),
        ];

        for (opcode, am1, am2) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).return_const(1u16);
            mock.expect_fetch_data().with(eq(am2)).return_const(2u16);
            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
                .return_const(());
            mock.expect_write_data().with(eq(am1), always(), eq(3)).return_const(());

            proc_add(&mut mock, opcode, &am1, &am2);
        }
    }

    #[test]
    fn add_rr() {
        let am1 = AM::Direct_HL;
        let am2 = AM::Direct_BC;

        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(am1)).return_const(1u16);
        mock.expect_fetch_data().with(eq(am2)).return_const(2u16);
        mock.expect_set_flags()
            .with(eq(None), eq(Some(false)), eq(Some(false)), eq(Some(false)))
            .return_const(());
        mock.expect_write_data().with(eq(am1), always(), eq(3u16)).return_const(());

        proc_add(&mut mock, 0x09, &am1, &am2);
    }

    #[test]
    fn add_set_flags_rr() {
        let opcode = 0x29u8;
        let am1 = AM::Direct_HL;
        let am2 = AM::Direct_DE;

        let cases = [
            // c
            (0xFFFFu16, 1u16, 0u16, true, true),
            // h
            (0xFFF, 1, 0x1000, true, false),
            //
            (0x00, 0, 0, false, false),
        ];

        for (r1, r2, ret, h, c) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).return_const(r1);
            mock.expect_fetch_data().with(eq(am2)).return_const(r2);
            mock.expect_write_data().with(eq(am1), always(), eq(ret)).return_const(());
            mock.expect_set_flags()
                .with(eq(None), eq(Some(false)), eq(Some(h)), eq(Some(c)))
                .return_const(());

            proc_add(&mut mock, opcode, &am1, &am2);
        }
    }

    #[test]
    fn add_set_flags_r() {
        let opcode = 0x86u8;
        let am1 = AM::Direct_A;
        let am2 = AM::Indirect_HL;

        let cases = [
            // c
            (0xFFu8, 1u8, 0u16, true, true, true),
            // h
            (0xF, 1, 0x10, false, true, false),
            // z
            (0x00, 0, 0, true, false, false),
        ];

        for (r1, r2, ret, z, h, c) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).return_const(r1);
            mock.expect_fetch_data().with(eq(am2)).return_const(r2);
            mock.expect_write_data().with(eq(am1), always(), eq(ret)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(z)), eq(Some(false)), eq(Some(h)), eq(Some(c)))
                .return_const(());

            proc_add(&mut mock, opcode, &am1, &am2);
        }
    }
}
