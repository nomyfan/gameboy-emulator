use super::utils::{cpu_fetch_a, cpu_write_a};
use crate::cpu16::Cpu16;
use crate::instruction::{get_cycles, AddressingMode};

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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    use crate::cpu16::MockCpu16;
    type AM = AddressingMode;

    #[test]
    fn sub() {
        let opcode = 0xD6u8;
        let am1 = AM::Direct_A;
        let am2 = AM::PC1;

        let cases = [
            (1u16, 1u16, 0u16, (true, false, false)),
            (2, 1, 1, (false, false, false)),
            (1, 3, -2i8 as u8 as u16, (false, true, true)),
            (0x11, 0x21, 0xF0, (false, false, true)),
        ];

        for (a, v, ret, (z, h, c)) in cases.into_iter() {
            let mut mock = MockCpu16::new();
            mock.expect_fetch_data().with(eq(am1)).once().return_const(a);
            mock.expect_fetch_data().with(eq(am2)).once().return_const(v);
            mock.expect_write_data().with(eq(am1), always(), eq(ret)).once().return_const(());
            mock.expect_set_flags()
                .once()
                .with(eq(Some(z)), eq(Some(true)), eq(Some(h)), eq(Some(c)))
                .return_const(());
            proc_sub(&mut mock, opcode, &am2);
        }
    }
}
