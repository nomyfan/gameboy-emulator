use crate::cpu16::Cpu16;
use crate::instruction::{AddressingMode, Condition};

pub(crate) fn check_condition(cond: Option<&Condition>, cpu: &mut impl Cpu16) -> bool {
    match cond {
        Some(cond) => match cond {
            Condition::Z => cpu.flags().0,
            Condition::NZ => !cpu.flags().0,
            Condition::C => cpu.flags().3,
            Condition::NC => !cpu.flags().3,
        },
        None => true,
    }
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
        let cases = [(true, false), (true, true), (false, true), (false, false)];

        for (z, c) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_flags().return_const((z, false, false, c));

            assert_eq!(check_condition(None.as_ref(), &mut mock), true);
            assert_eq!(check_condition(Some(Condition::C).as_ref(), &mut mock), c);
            assert_eq!(check_condition(Some(Condition::NC).as_ref(), &mut mock), !c);
            assert_eq!(check_condition(Some(Condition::Z).as_ref(), &mut mock), z);
            assert_eq!(check_condition(Some(Condition::NZ).as_ref(), &mut mock), !z);
        }
    }
}
