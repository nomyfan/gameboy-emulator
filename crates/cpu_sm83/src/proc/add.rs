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
    let sum = if opcode == 0xE8 {
        operand1.wrapping_add_signed(operand2 as u8 as i8 as i16)
    } else {
        operand1.wrapping_add(operand2)
    };

    let z = if opcode == 0x09 || opcode == 0x19 || opcode == 0x29 || opcode == 0x39 {
        None
    } else if opcode == 0xE8 {
        Some(false)
    } else {
        Some(sum as u8 == 0)
    };

    let (h, c) =
        // 16 bits
        if opcode == 0x09 || opcode == 0x19 || opcode == 0x29 || opcode == 0x39 || opcode == 0xE8 {
            let h = (operand1 & 0xFFF) + (operand2 & 0xFFF) >= 0x1000;
            let c = (operand1 as u32) + (operand2 as u32) >= 0x10000;
            (Some(h), Some(c))
        } else { // 8 bits
            let h = (operand1 & 0xF) + (operand2 & 0xF) >= 0x10;
            let c = (operand1 & 0xFF) + (operand2 & 0xFF) >= 0x100;
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
}
