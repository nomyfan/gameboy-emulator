mod ld;
use crate::Cpu;
pub use ld::LD;

pub trait Instr {
    fn exec(self, cpu: &mut Cpu);
}

pub enum Instruction {
    LD(LD),
}
