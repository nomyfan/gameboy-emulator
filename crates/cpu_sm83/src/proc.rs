use crate::{
    cpu16::Cpu16,
    instruction::{get_cycles, AddressingMode, CbInstruction, Condition},
};

fn check_condition(cond: Option<&Condition>, cpu: &mut impl Cpu16) -> bool {
    match cond {
        Some(cond) => match cond {
            Condition::Z => cpu.flags().0,
            Condition::NZ => !cpu.flags().0,
            Condition::C => cpu.flags().3,
            Condition::NC => !cpu.flags().0,
        },
        None => true,
    }
}

fn cpu_stack_push2(cpu: &mut impl Cpu16, value: u16) {
    cpu.stack_push((value >> 8) as u8);
    cpu.stack_push(value as u8);
}

fn cpu_stack_pop2(cpu: &mut impl Cpu16) -> u16 {
    let lo = cpu.stack_pop();
    let hi = cpu.stack_pop();

    ((hi as u16) << 8) | (lo as u16)
}

#[inline]
fn cpu_fetch_a(cpu: &mut impl Cpu16) -> u8 {
    cpu.fetch_data(&AddressingMode::Direct_A) as u8
}

#[inline]
fn cpu_write_a(cpu: &mut impl Cpu16, value: u8) {
    cpu.write_data(&AddressingMode::Direct_A, 0, value as u16);
}

pub(crate) fn proc_inc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am);
    let value = value.wrapping_add(1);

    if opcode != 0x03 && opcode != 0x13 && opcode != 0x23 && opcode != 0x33 {
        cpu.set_flags(Some(value == 0), Some(false), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

pub(crate) fn proc_dec(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am);
    let value = value.wrapping_sub(1);

    if opcode != 0x0B && opcode != 0x1B && opcode != 0x2B && opcode != 0x3B {
        cpu.set_flags(Some(value == 0), Some(true), Some((value & 0xF) == 0), None);
    }

    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

pub(crate) fn proc_jp(
    cpu: &mut impl Cpu16,
    opcode: u8,
    cond: &Option<Condition>,
    am: &AddressingMode,
) -> u8 {
    let addr = cpu.fetch_data(am);
    if check_condition(cond.as_ref(), cpu) {
        cpu.jp(addr);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

pub(crate) fn proc_jr(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    let unsigned_r8 = cpu.fetch_data(&AddressingMode::PC1) as u8;
    if check_condition(cond.as_ref(), cpu) {
        cpu.jr(unsigned_r8 as i8);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

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
        let unsigned_r8 = cpu.fetch_data(&AddressingMode::PC1);

        let h = (operand2 & 0xF) + (unsigned_r8 & 0xF) > 0xF;
        let c = (operand2 & 0xFF) + (unsigned_r8 & 0xFF) > 0xFF;

        operand2 = operand2.wrapping_add_signed(unsigned_r8 as i8 as i16);

        cpu.set_flags(Some(false), Some(false), Some(h), Some(c));
    }

    if opcode == 0xF0 || opcode == 0xF2 || opcode == 0xFA {
        // (a8) / (C) / (a16)
        operand2 = cpu.bus_read(operand2) as u16;
    }

    cpu.write_data(am1, operand1, operand2);

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

pub(crate) fn proc_adc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let c = if cpu.flags().3 { 1u8 } else { 0u8 };

    let value = a.wrapping_add(data).wrapping_add(c);
    let h = (value & 0xF) + (a & 0xF) + (c & 0xF) > 0xF;
    let c = (value as u16) + (a as u16) + (c as u16) > 0xFF;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), Some(false), Some(h), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_sub(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am);
    let a = cpu_fetch_a(cpu);
    let value = a.wrapping_sub(data as u8);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) < 0;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_sbc(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let c = if cpu.flags().3 { 1u8 } else { 0u8 };

    let value = a.wrapping_sub(data).wrapping_sub(c);

    let z = value == 0;
    let h = ((a & 0xF) as i16) - ((data & 0xF) as i16) - ((c & 0xF) as i16) < 0;
    let c = (a as i16) - (data as i16) - (c as i16) < 0;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_call(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    let value = cpu.fetch_data(&AddressingMode::PC2);
    if check_condition(cond.as_ref(), cpu) {
        cpu.stack_push_pc();
        cpu.jp(value);

        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

pub(crate) fn proc_push(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am);
    cpu_stack_push2(cpu, value);

    get_cycles(opcode).0
}

pub(crate) fn proc_pop(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu_stack_pop2(cpu);
    cpu.write_data(am, 0, value);

    get_cycles(opcode).0
}

pub(crate) fn proc_ret(cpu: &mut impl Cpu16, opcode: u8, cond: &Option<Condition>) -> u8 {
    if check_condition(cond.as_ref(), cpu) {
        let addr = cpu_stack_pop2(cpu);
        cpu.jp(addr);
        return get_cycles(opcode).0;
    }

    get_cycles(opcode).1
}

pub(crate) fn proc_reti(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_ime(true);
    proc_ret(cpu, opcode, &None);

    get_cycles(opcode).0
}

pub(crate) fn proc_rst(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let value: u16 = match opcode {
        0xC7 => 0x00,
        0xCF => 0x08,
        0xD7 => 0x10,
        0xDF => 0x18,
        0xE7 => 0x20,
        0xEF => 0x28,
        0xF7 => 0x30,
        0xFF => 0x38,
        _ => unreachable!(),
    };
    cpu.stack_push_pc();
    cpu.jp(value);

    get_cycles(opcode).0
}

pub(crate) fn proc_and(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let value = a & operand;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), Some(false), Some(true), Some(false));

    get_cycles(opcode).0
}

pub(crate) fn proc_or(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let value = a | operand;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), Some(false), Some(false), Some(false));

    get_cycles(opcode).0
}

pub(crate) fn proc_xor(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let operand = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);
    let value = a ^ operand;

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), Some(false), Some(false), Some(false));

    get_cycles(opcode).0
}

pub(crate) fn proc_di(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_ime(false);

    get_cycles(opcode).0
}

pub(crate) fn proc_ei(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    // TODO: We need another cycle to effect.
    cpu.set_ime(true);

    get_cycles(opcode).0
}

pub(crate) fn proc_halt(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_halt(true);

    get_cycles(opcode).0
}

pub(crate) fn proc_stop(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.stop();

    get_cycles(opcode).0
}

pub(crate) fn proc_rlca(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = (a >> 7) & 1;

    // Move the MSB(it) to the LSB(it)
    cpu_write_a(cpu, (a << 1) | c);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));

    get_cycles(opcode).0
}

pub(crate) fn proc_rla(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = (a >> 7) & 1 == 1;
    let carry = if cpu.flags().3 { 1u8 } else { 0u8 };

    cpu_write_a(cpu, (a << 1) | carry);
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_rrca(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = a & 1;

    // Move the LSB(it) to the MSB(it)
    cpu_write_a(cpu, (a >> 1) | (c << 7));
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c == 1));

    get_cycles(opcode).0
}

pub(crate) fn proc_rra(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    let c = a & 1 == 1;
    let carry = if cpu.flags().3 { 1u8 } else { 0u8 };

    cpu_write_a(cpu, (a >> 1) | (carry << 7));
    cpu.set_flags(Some(false), Some(false), Some(false), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_daa(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let mut acc = 0;
    let mut c = false;
    let a = cpu_fetch_a(cpu);

    let (_, flag_n, flag_h, flag_c) = cpu.flags();

    if flag_h || (!flag_n && (a & 0xF) > 0x09) {
        acc |= 0x06;
    }

    if flag_c || (!flag_n && a > 0x99) {
        acc |= 0x60;
        c = true;
    }

    let value = if flag_n { a.wrapping_sub(acc) } else { a.wrapping_add(acc) };
    cpu_write_a(cpu, value);
    cpu.set_flags(Some(value == 0), None, Some(false), Some(c));

    get_cycles(opcode).0
}

pub(crate) fn proc_cpl(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    let a = cpu_fetch_a(cpu);
    cpu_write_a(cpu, !a);
    cpu.set_flags(None, Some(true), Some(true), None);

    get_cycles(opcode).0
}

pub(crate) fn proc_scf(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_flags(None, Some(false), Some(false), Some(true));

    get_cycles(opcode).0
}

pub(crate) fn proc_ccf(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    cpu.set_flags(None, Some(false), Some(false), Some(!cpu.flags().3));

    get_cycles(opcode).0
}

pub(crate) fn proc_cp(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let value = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);

    cpu.set_flags(Some(value == a), Some(true), Some((a & 0x0F) < (value & 0x0F)), Some(a < value));

    get_cycles(opcode).0
}

pub(crate) fn proc_cb(cpu: &mut impl Cpu16, opcode: u8) -> u8 {
    fn decode_addressing_mode(value: u8) -> AddressingMode {
        match value {
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

    fn decode_inst(value: u8) -> CbInstruction {
        match value {
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

    let value = cpu.fetch_data(&AddressingMode::PC1) as u8;
    let am = decode_addressing_mode(value & 0b111);

    match decode_inst(value) {
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
            // 左移1位。
            let msb = (value >> 7) & 1;
            let new_value = value << 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(msb == 1));
        }
        CbInstruction::SRA => {
            // 右移1位。Arithmetic shift.
            let mlb = value & 1;
            let new_value = (value as i8) >> 1;

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
            // 右移1位。Logical shift.
            let mlb = value & 1;
            let new_value = value >> 1;

            cpu.write_data(&am, 0, new_value as u16);
            cpu.set_flags(Some(new_value == 0), Some(false), Some(false), Some(mlb == 1));
        }
        CbInstruction::BIT => {
            // BIT tests.
            let bit = (value - 0x40) / 8;
            cpu.set_flags(Some((value & (1 << bit)) == 0), Some(false), Some(true), None);
        }
        CbInstruction::RES => {
            // Set specific bit to be zero.
            let bit = (value - 0x80) / 8;
            let new_value = value & (!(1 << bit));

            cpu.write_data(&am, 0, new_value as u16);
        }
        CbInstruction::SET => {
            // Set specific bit to be one.
            let bit = (value - 0xC0) / 8;
            let new_value = value | (1 << bit);

            cpu.write_data(&am, 0, new_value as u16);
        }
    }

    get_cycles(opcode).0 + if let AddressingMode::Indirect_HL = am { 16 } else { 8 }
}

#[cfg(test)]
mod tests {
    use crate::cpu16::MockCpu16;
    use crate::instruction::AddressingMode as AM;
    use mockall::predicate::*;

    use super::*;

    #[test]
    fn condition_checking() {
        let mut mock = MockCpu16::new();
        mock.expect_flags().returning(|| (false, false, false, false));

        assert_eq!(check_condition(None.as_ref(), &mut mock), true);
        assert_eq!(check_condition(Some(Condition::C).as_ref(), &mut mock), false);
        assert_eq!(check_condition(Some(Condition::NC).as_ref(), &mut mock), true);
        assert_eq!(check_condition(Some(Condition::Z).as_ref(), &mut mock), false);
        assert_eq!(check_condition(Some(Condition::NZ).as_ref(), &mut mock), true);
    }

    mod proc_ld {
        use super::*;

        #[test]
        fn ld() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(AddressingMode::Direct_A)).returning(|_| 0x12);
            mock.expect_fetch_data()
                .with(eq(AddressingMode::Direct_B))
                .once()
                .returning(|_| 0x1234);
            mock.expect_write_data()
                .with(
                    eq(AddressingMode::Direct_A),
                    always(),
                    function(|value| (value & 0xFF) == 0x34),
                )
                .once()
                .returning(|_, _, _| {});

            let am1 = AM::Direct_A;
            let am2 = AM::Direct_B;
            let opcode = 0x78;

            assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
        }

        #[test]
        fn ld_hl_plus_minus() {
            let cases = [
                (0x22, (AM::Indirect_HL, 0x12u16), (AM::Direct_A, 0x34), true),
                (0x2A, (AM::Direct_A, 0x12), (AM::Indirect_HL, 0x34), true),
                (0x32, (AM::Indirect_HL, 0x12u16), (AM::Direct_A, 0x34), false),
                (0x3A, (AM::Direct_A, 0x12), (AM::Indirect_HL, 0x34), false),
            ];

            for (opcode, (am1, ret1), (am2, ret2), inc) in cases.into_iter() {
                let mut mock = MockCpu16::new();

                mock.expect_fetch_data().with(eq(am1)).returning(move |_| ret1);
                mock.expect_fetch_data().with(eq(am2)).once().returning(move |_| ret2);
                mock.expect_write_data()
                    .with(eq(am1), always(), function(move |value| (value & 0xFF) == ret2))
                    .once()
                    .returning(|_, _, _| {});
                mock.expect_inc_dec_hl().with(eq(inc)).once().returning(|_| {});
                assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
            }
        }

        #[test]
        fn ld_sp_r8() {
            let am1 = AM::Direct_HL;
            let am2 = AM::Direct_SP;
            let opcode = 0xF8;

            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am2)).once().returning(|_| 2);
            mock.expect_fetch_data().with(eq(AM::PC1)).once().returning(|_| (-1i8) as u16);
            mock.expect_fetch_data().with(eq(am1)).returning(|_| 0x34);
            mock.expect_write_data().with(eq(am1), always(), eq(2 - 1)).returning(|_, _, _| {});

            mock.expect_set_flags()
                .with(eq(Some(false)), eq(Some(false)), eq(Some(true)), eq(Some(true)))
                .once()
                .returning(|_, _, _, _| {});

            assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
        }

        #[test]
        fn ld_add_0xff00() {
            // (a8) / (C)
            let cases = [
                (0xE0, (AM::PC1, 0x12u8), (AM::Direct_A, 0x34u8), 0xFF12u16, 0x34u8),
                (0xE2, (AM::Direct_C, 0x12), (AM::Direct_A, 0x34), 0xFF12, 0x34),
            ];

            for (opcode, (am1, val1), (am2, val2), addr, value) in cases.into_iter() {
                let mut mock = MockCpu16::new();
                mock.expect_fetch_data().with(eq(am2)).once().returning(move |_| val2 as u16);
                mock.expect_fetch_data().with(eq(am1)).once().returning(move |_| val1 as u16);
                mock.expect_write_data()
                    .with(eq(am1), eq(addr), eq(value as u16))
                    .returning(|_, _, _| {});

                assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
            }
        }

        #[test]
        fn ld_add_0xff00_2() {
            // (a8) / (C)
            let cases = [
                (0xF0, (AM::Direct_A, 0x12u8), (AM::PC1, 0x34u8), 0xFF34, 0x34u8),
                (0xF2, (AM::Direct_A, 0x12), (AM::Direct_C, 0x34), 0xFF34, 0x34),
            ];

            for (opcode, (am1, val1), (am2, val2), addr, value) in cases.into_iter() {
                let mut mock = MockCpu16::new();
                mock.expect_fetch_data().with(eq(am2)).once().returning(move |_| val2 as u16);
                mock.expect_fetch_data().with(eq(am1)).once().returning(move |_| val1 as u16);
                mock.expect_write_data()
                    .with(eq(am1), always(), eq(value as u16))
                    .returning(|_, _, _| {});
                mock.expect_bus_read().with(eq(addr)).once().returning(move |_| value);

                assert_eq!(proc_ld(&mut mock, opcode, &am1, &am2), get_cycles(opcode).0);
            }
        }
    }
}
