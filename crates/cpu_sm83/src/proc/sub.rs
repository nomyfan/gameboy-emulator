use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::alu::sub::alu_sub;
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

pub(crate) fn proc_sub(cpu: &mut impl Cpu16, opcode: u8, am: &AddressingMode) -> u8 {
    let data = cpu.fetch_data(am) as u8;
    let a = cpu_fetch_a(cpu);

    let (value, z, h, c) = alu_sub(a, data);

    cpu_write_a(cpu, value);
    cpu.set_flags(Some(z), Some(true), Some(h), Some(c));

    get_cycles(opcode).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    use crate::cpu16::MockCpu16;
    type Am = AddressingMode;

    #[test]
    fn sub() {
        let opcode = 0xD6u8;
        let am_a = Am::Direct_A;
        let am_8 = Am::Eight;

        let cases = [
            (1u16, 1u16, 0u16, (true, false, false)),
            (2, 1, 1, (false, false, false)),
            (1, 3, -2i8 as u8 as u16, (false, true, true)),
            (0x11, 0x21, 0xF0, (false, false, true)),
        ];

        for (a, v, ret, (z, h, c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am_a)).once().return_const(a);
            mock.expect_fetch_data().with(eq(am_8)).once().return_const(v);
            mock.expect_write_data().with(eq(am_a), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(z)), eq(Some(true)), eq(Some(h)), eq(Some(c)))
                .return_const(());
            proc_sub(&mut mock, opcode, &am_8);
        }
    }
}
