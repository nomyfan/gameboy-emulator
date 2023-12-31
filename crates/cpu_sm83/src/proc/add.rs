use crate::alu::add::*;
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

    if is_rr {
        let (sum, h, c) = alu_add_16(operand1, operand2);
        cpu.write_data(am1, 0, sum);
        cpu.set_flags(None, Some(false), Some(h), Some(c));
    } else if is_sp_r8 {
        let (sum, h, c) = alu_add_sp_r8(operand1, operand2 as u8 as i8);
        cpu.write_data(am1, 0, sum);
        cpu.set_flags(Some(false), Some(false), Some(h), Some(c));
    } else {
        let (sum, z, h, c) = alu_add_8(operand1 as u8, operand2 as u8);
        cpu.write_data(am1, 0, sum as u16);
        cpu.set_flags(Some(z), Some(false), Some(h), Some(c));
    }

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use mockall::predicate::*;

    type Am = AddressingMode;

    #[test]
    fn add_sp_r8() {
        let opcode = 0xE8u8;
        let am_sp = Am::Direct_SP;
        let am_8 = Am::Eight;

        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(am_sp)).once().return_const(1u16);
        mock.expect_fetch_data().with(eq(am_8)).once().return_const((-1i8) as u16);
        mock.expect_write_data().with(eq(am_sp), always(), eq(0)).once().return_const(());
        mock.expect_set_flags()
            .with(eq(Some(false)), eq(Some(false)), eq(Some(true)), eq(Some(true)))
            .once()
            .return_const(());

        assert_eq!(proc_add(&mut mock, opcode, &am_sp, &am_8), 16);
    }

    #[test]
    fn add_r() {
        let cases = [
            (0x85u8, Am::Direct_A, Am::Direct_L),
            (0x86u8, Am::Direct_A, Am::Indirect_HL),
            (0xC5u8, Am::Direct_A, Am::Eight),
        ];

        for (opcode, am1, am2) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).once().return_const(1u16);
            mock.expect_fetch_data().with(eq(am2)).once().return_const(2u16);
            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
                .once()
                .return_const(());
            mock.expect_write_data().with(eq(am1), always(), eq(3)).once().return_const(());

            proc_add(&mut mock, opcode, &am1, &am2);
        }
    }

    #[test]
    fn add_rr() {
        let am_hl = Am::Direct_HL;
        let am_bc = Am::Direct_BC;

        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(am_hl)).once().return_const(1u16);
        mock.expect_fetch_data().with(eq(am_bc)).once().return_const(2u16);
        mock.expect_set_flags()
            .with(eq(None), eq(Some(false)), eq(Some(false)), eq(Some(false)))
            .once()
            .return_const(());
        mock.expect_write_data().with(eq(am_hl), always(), eq(3u16)).once().return_const(());

        proc_add(&mut mock, 0x09, &am_hl, &am_bc);
    }

    #[test]
    fn add_rr_set_flags() {
        let opcode = 0x29u8;
        let am_hl = Am::Direct_HL;
        let am_de = Am::Direct_DE;

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
            mock.expect_fetch_data().with(eq(am_hl)).once().return_const(r1);
            mock.expect_fetch_data().with(eq(am_de)).once().return_const(r2);
            mock.expect_write_data().with(eq(am_hl), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .with(eq(None), eq(Some(false)), eq(Some(h)), eq(Some(c)))
                .once()
                .return_const(());

            proc_add(&mut mock, opcode, &am_hl, &am_de);
        }
    }

    #[test]
    fn add_r_set_flags() {
        let opcode = 0x86u8;
        let am_a = Am::Direct_A;
        let am_ind_hl = Am::Indirect_HL;

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
            mock.expect_fetch_data().with(eq(am_a)).once().return_const(r1);
            mock.expect_fetch_data().with(eq(am_ind_hl)).once().return_const(r2);
            mock.expect_write_data().with(eq(am_a), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .with(eq(Some(z)), eq(Some(false)), eq(Some(h)), eq(Some(c)))
                .once()
                .return_const(());

            proc_add(&mut mock, opcode, &am_a, &am_ind_hl);
        }
    }
}
