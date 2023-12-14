use crate::cpu16::Cpu16;
use crate::instruction::{AddressingMode, Condition};

pub(crate) fn check_condition(cond: Option<&Condition>, cpu: &mut impl Cpu16) -> bool {
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

pub(crate) fn cpu_stack_push2(cpu: &mut impl Cpu16, value: u16) {
    cpu.stack_push((value >> 8) as u8);
    cpu.stack_push(value as u8);
}

pub(crate) fn cpu_stack_pop2(cpu: &mut impl Cpu16) -> u16 {
    let lo = cpu.stack_pop();
    let hi = cpu.stack_pop();

    ((hi as u16) << 8) | (lo as u16)
}

#[inline]
pub(crate) fn cpu_fetch_a(cpu: &mut impl Cpu16) -> u8 {
    cpu.fetch_data(&AddressingMode::Direct_A) as u8
}

#[inline]
pub(crate) fn cpu_write_a(cpu: &mut impl Cpu16, value: u8) {
    cpu.write_data(&AddressingMode::Direct_A, 0, value as u16);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu16::MockCpu16;

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
}
