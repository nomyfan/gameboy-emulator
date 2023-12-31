use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode, CbInstruction};

pub(crate) fn proc_cb(cpu: &mut impl Cpu16) -> u8 {
    fn decode_addressing_mode(opcode: u8) -> AddressingMode {
        match opcode {
            0 => AddressingMode::Direct_B,
            1 => AddressingMode::Direct_C,
            2 => AddressingMode::Direct_D,
            3 => AddressingMode::Direct_E,
            4 => AddressingMode::Direct_H,
            5 => AddressingMode::Direct_L,
            6 => AddressingMode::Indirect_HL,
            7 => AddressingMode::Direct_A,
            _ => unreachable!("Only B,C,D,E,H,L,HL,A are valid for CB instruction."),
        }
    }

    fn decode_inst(opcode: u8) -> CbInstruction {
        match opcode {
            0x00..=0x07 => CbInstruction::RLC,
            0x08..=0x0F => CbInstruction::RRC,
            0x10..=0x17 => CbInstruction::RL,
            0x18..=0x1F => CbInstruction::RR,
            0x20..=0x27 => CbInstruction::SLA,
            0x28..=0x2F => CbInstruction::SRA,
            0x30..=0x37 => CbInstruction::SWAP,
            0x38..=0x3F => CbInstruction::SRL,
            0x40..=0x7F => CbInstruction::BIT,
            0x80..=0xBF => CbInstruction::RES,
            0xC0..=0xFF => CbInstruction::SET,
        }
    }

    let cb_opcode = cpu.fetch_data(&AddressingMode::Eight) as u8;
    let am = decode_addressing_mode(cb_opcode & 0b111);
    let value = cpu.fetch_data(&am) as u8;

    match decode_inst(cb_opcode) {
        CbInstruction::RLC => {
            // 左移1位，MSB换到MLB。
            let msb = (value >> 7) & 1;
            let new_value = (value << 1) | msb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::RRC => {
            // 右移1位，MLB换到MSB。
            let mlb = value & 1;
            let new_value = (value >> 1) | (mlb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::RL => {
            // 左移1位，Flag C作为MLB。
            let msb = (value >> 7) & 1;
            let mlb = if cpu.flags().3 { 1 } else { 0 };
            let new_value = (value << 1) | mlb;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::RR => {
            // 右移1位，Flag C作为MSB。
            let mlb = value & 1;
            let msb = if cpu.flags().3 { 1 } else { 0 };
            let new_value = (value >> 1) | (msb << 7);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::SLA => {
            let msb = (value >> 7) & 1;
            let new_value = value << 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::SRA => {
            let mlb = value & 1;
            let new_value = ((value as i8) >> 1) as u8;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::SWAP => {
            // 高低4位交换。
            let new_value = ((value & 0xF0) >> 4) | ((value & 0x0F) << 4);

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(false));
        }
        CbInstruction::SRL => {
            let mlb = value & 1;
            let new_value = value >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::BIT => {
            let bit = (cb_opcode & 0b111000) >> 3;
            cpu.set_flags(Some((value & (1 << bit)) == 0), Some(false), Some(true), None);
        }
        CbInstruction::RES => {
            let bit = (cb_opcode & 0b111000) >> 3;
            let new_value = value & (!(1 << bit));

            cpu.write_data(&am, 0, new_value as u16);
        }
        CbInstruction::SET => {
            let bit = (cb_opcode & 0b111000) >> 3;
            let new_value = value | (1 << bit);

            cpu.write_data(&am, 0, new_value as u16);
        }
    }

    get_cycles(0xCB).0 + if let AddressingMode::Indirect_HL = am { 16 } else { 8 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;
    use crate::instruction::AddressingMode as Am;
    use mockall::predicate::*;

    const AM_8: Am = Am::Eight;
    const AM_HL: Am = Am::Indirect_HL;

    #[test]
    fn rlc_set_flag_c() {
        let cases = [
            //
            ((0b1010_1000u16), (0b0101_0001u16, true)),
            ((0b0100_1000), (0b1001_0000, false)),
        ];

        for (in_v, (out_v, out_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x06u16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);
            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn rlc_set_flag_z() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x06u16);
        mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(0u16);
        mock.expect_write_data().with(eq(AM_HL), always(), eq(0u16)).return_const(());
        mock.expect_set_flags()
            .with(eq(Some(true)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
            .once()
            .return_const(());

        proc_cb(&mut mock);
    }

    #[test]
    fn rrc_set_flag_c() {
        let cases = [
            //
            ((0b0010_1001u16), (0b1001_0100u16, true)),
            ((0b0100_1000), (0b0010_0100, false)),
        ];

        for (in_v, (out_v, out_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x0Eu16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);
            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn rrc_set_flag_z() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x0Eu16);
        mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(0u16);
        mock.expect_write_data().with(eq(AM_HL), always(), eq(0u16)).return_const(());
        mock.expect_set_flags()
            .with(eq(Some(true)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
            .once()
            .return_const(());

        proc_cb(&mut mock);
    }

    #[test]
    fn rl_set_flag_c() {
        let cases = [
            //
            ((0b1000_1000u16, true), (0b0001_0001u16, true)),
            ((0b1000_1000u16, false), (0b0001_0000, true)),
            ((0b0100_1000, true), (0b1001_0001, false)),
            ((0b0100_1000, false), (0b1001_0000, false)),
        ];

        for ((in_v, in_c), (out_v, out_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x16u16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);
            mock.expect_flags().once().return_const([false, false, false, in_c]);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn rl_set_flag_z() {
        let cases = [
            //
            ((0u16, false), (false)),
            ((0b1000_0000u16, false), (true)),
        ];

        for ((in_v, in_c), out_c) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x16u16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);
            mock.expect_flags().once().return_const([false, false, false, in_c]);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(0)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(true)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn rr_set_flag_c() {
        let cases = [
            //
            ((0b1000_1001u16, true), (0b1100_0100u16, true)),
            ((0b1000_1001, false), (0b0100_0100, true)),
            ((0b0100_1000, true), (0b1010_0100, false)),
            ((0b0100_1000, false), (0b0010_0100, false)),
        ];

        for ((in_v, in_c), (out_v, out_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x1Eu16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);
            mock.expect_flags().once().return_const([false, false, false, in_c]);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn rr_set_flag_z() {
        let cases = [
            //
            ((0u16, false), (false)),
            ((0b0000_0001u16, false), (true)),
        ];

        for ((in_v, in_c), out_c) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x1Eu16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);
            mock.expect_flags().once().return_const([false, false, false, in_c]);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(0)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(true)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn sla() {
        let cases = [
            //
            ((0b1001_0000u16), (0b0010_0000u16, false, true)),
            ((0b0100_0000), (0b1000_0000, false, false)),
            ((0u16), (0, true, false)),
        ];

        for (in_v, (out_v, out_z, out_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x26u16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(out_z)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn sra_set_flag_z() {
        let cases = [
            //
            ((0b0001_0001u16), (0b0000_1000u16, false, true)),
            ((0b1000_0000), (0b1100_0000, false, false)),
            ((0), (0, true, false)),
        ];

        for (in_v, (out_v, out_z, out_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x2Eu16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(out_z)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn swap_set_flag_z() {
        let cases = [
            //
            ((0b1010_0101u16), (0b0101_1010u16, false)),
            ((0), (0, true)),
        ];

        for (in_v, (out_v, out_z)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x36u16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(out_z)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn srl_set_flag_c() {
        let cases =
            [((0b0001_0001u16), (0b0000_1000u16, true)), ((0b1000_0000), (0b0100_0000, false))];

        for (in_v, (out_v, out_c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x3Eu16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(false)), eq(Some(out_c)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn srl_set_flag_z() {
        let cases = [
            //
            ((0b1000_0000u16), (0b0100_0000, false)),
            ((0), (0, true)),
        ];

        for (in_v, (out_v, out_z)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x3Eu16);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);

            mock.expect_write_data().with(eq(AM_HL), always(), eq(out_v)).return_const(());
            mock.expect_set_flags()
                .with(eq(Some(out_z)), eq(Some(false)), eq(Some(false)), eq(Some(false)))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn bit_set_flag_z() {
        let cases = [
            //
            ((0x46u16, 1u16), (false)),
            ((0x46u16, 0b0100_0000), (true)),
        ];
        for ((in_cb_opcode, in_v), out_z) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AM_8)).once().return_const(in_cb_opcode);
            mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(in_v);
            mock.expect_set_flags()
                .with(eq(Some(out_z)), eq(Some(false)), eq(Some(true)), eq(None))
                .once()
                .return_const(());

            proc_cb(&mut mock);
        }
    }

    #[test]
    fn res() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0x96u16);
        mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(0xFFu16);
        mock.expect_write_data().with(eq(AM_HL), always(), eq(0b1111_1011)).once().return_const(());

        proc_cb(&mut mock);
    }

    #[test]
    fn set() {
        let mut mock = MockCpu16::new();
        mock.expect_fetch_data().with(eq(AM_8)).once().return_const(0xE6u16);
        mock.expect_fetch_data().with(eq(AM_HL)).once().return_const(0u16);
        mock.expect_write_data().with(eq(AM_HL), always(), eq(0b0001_0000)).once().return_const(());

        proc_cb(&mut mock);
    }
}
