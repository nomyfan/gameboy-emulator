pub(crate) struct Instruction {
    pub(crate) opcode: u8,
    pub(crate) ty: InstructionType,
    pub(crate) cond: Option<Condition>,
    pub(crate) operand1: Option<AddressingMode>,
    pub(crate) operand2: Option<AddressingMode>,
}

impl core::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instruction")
            .field("opcode", &format_args!("{:#02X}", &self.opcode))
            .field("ty", &self.ty)
            .field("cond", &self.cond)
            .field("operand1", &self.operand1)
            .field("operand2", &self.operand2)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) enum InstructionType {
    NOP,
    LD,
    INC,
    DEC,
    JR,
    JP,
    CALL,
    ADD,
    SUB,
    PUSH,
    POP,
    RET,
    RETI,
    RST,
    AND,
    OR,
    XOR,
    STOP,
    DI,
    EI,
    HALT, // TODO
}

#[derive(Debug)]
pub(crate) enum AddressingMode {
    /// Register direct
    Direct(Register),
    /// Register indirect
    Indirect(Register),
    /// PC-relative 1 byte
    PC1,
    /// PC-relative 2 bytes
    PC2,
}

#[derive(Debug)]
pub(crate) enum Condition {
    Z,
    NZ,
    C,
    NC,
}

#[derive(Debug)]
pub(crate) enum Register {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
}

macro_rules! inst_simple {
    ($opcode:expr, $ty:expr) => {
        Instruction { opcode: $opcode, ty: $ty, cond: None, operand1: None, operand2: None }
    };
}

macro_rules! inst_operand1 {
    ($opcode:expr, $ty:expr, $op1:expr) => {
        Instruction { opcode: $opcode, ty: $ty, cond: None, operand1: Some($op1), operand2: None }
    };
}

macro_rules! inst_ld {
    ($opcode:expr, $op1:expr, $op2:expr) => {
        Instruction {
            opcode: $opcode,
            ty: InstructionType::LD,
            cond: None,
            operand1: Some($op1),
            operand2: Some($op2),
        }
    };
}

macro_rules! inst_inc {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::INC, $op1)
    };
}

macro_rules! inst_dec {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::DEC, $op1)
    };
}

macro_rules! inst_jp {
    ($opcode:expr, $op1:expr, $cond:expr) => {
        Instruction {
            opcode: $opcode,
            ty: InstructionType::JP,
            cond: Some($cond),
            operand1: Some($op1),
            operand2: None,
        }
    };
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::JP, $op1)
    };
}

macro_rules! inst_jr {
    ($opcode:expr, $cond:expr) => {
        Instruction {
            opcode: $opcode,
            ty: InstructionType::JR,
            cond: Some($cond),
            operand1: Some(AddressingMode::PC1),
            operand2: None,
        }
    };
    ($opcode:expr) => {
        inst_operand1!($opcode, InstructionType::JR, AddressingMode::PC1)
    };
}

macro_rules! inst_call {
    ($opcode:expr, $cond:expr) => {
        Instruction {
            opcode: $opcode,
            ty: InstructionType::CALL,
            cond: Some($cond),
            operand1: Some(AddressingMode::PC2),
            operand2: None,
        }
    };
    ($opcode:expr) => {
        inst_operand1!($opcode, InstructionType::CALL, AddressingMode::PC2)
    };
}

macro_rules! inst_add {
    ($opcode:expr, $op1:expr, $op2:expr) => {
        Instruction {
            opcode: $opcode,
            ty: InstructionType::ADD,
            cond: None,
            operand1: Some($op1),
            operand2: Some($op2),
        }
    };
}

macro_rules! inst_sub {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::SUB, $op1)
    };
}

macro_rules! inst_and {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::AND, $op1)
    };
}

macro_rules! inst_or {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::OR, $op1)
    };
}

macro_rules! inst_xor {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::XOR, $op1)
    };
}

macro_rules! inst_push {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::PUSH, $op1)
    };
}

macro_rules! inst_pop {
    ($opcode:expr, $op1:expr) => {
        inst_operand1!($opcode, InstructionType::POP, $op1)
    };
}

macro_rules! inst_ret {
    ($opcode:expr, $cond:expr) => {
        Instruction {
            opcode: $opcode,
            ty: InstructionType::RET,
            cond: Some($cond),
            operand1: None,
            operand2: None,
        }
    };
    ($opcode:expr) => {
        inst_simple!($opcode, InstructionType::RET)
    };
}

macro_rules! inst_reti {
    ($opcode:expr) => {
        inst_simple!($opcode, InstructionType::RETI)
    };
}

macro_rules! inst_rst {
    ($opcode:expr) => {
        inst_simple!($opcode, InstructionType::RST)
    };
}

const INSTRUCTIONS: [Instruction; 209] = [
    // 0x0x
    inst_simple!(0x00, InstructionType::NOP),
    inst_ld!(0x01, AddressingMode::Direct(Register::BC), AddressingMode::PC2),
    inst_ld!(0x02, AddressingMode::Indirect(Register::BC), AddressingMode::Direct(Register::A)),
    inst_inc!(0x03, AddressingMode::Direct(Register::BC)),
    inst_inc!(0x04, AddressingMode::Direct(Register::B)),
    inst_dec!(0x05, AddressingMode::Direct(Register::B)),
    inst_ld!(0x06, AddressingMode::Direct(Register::B), AddressingMode::PC1),
    inst_ld!(0x08, AddressingMode::PC2, AddressingMode::Direct(Register::SP)),
    inst_add!(0x09, AddressingMode::Direct(Register::HL), AddressingMode::Direct(Register::BC)),
    inst_ld!(0x0A, AddressingMode::Direct(Register::A), AddressingMode::Indirect(Register::BC)),
    inst_dec!(0x0B, AddressingMode::Direct(Register::BC)),
    inst_inc!(0x0C, AddressingMode::Direct(Register::C)),
    inst_dec!(0x0D, AddressingMode::Direct(Register::C)),
    inst_ld!(0x0E, AddressingMode::Direct(Register::C), AddressingMode::PC1),
    // 0x1x
    inst_simple!(0x10, InstructionType::STOP),
    inst_ld!(0x11, AddressingMode::Direct(Register::DE), AddressingMode::PC2),
    inst_ld!(0x12, AddressingMode::Indirect(Register::DE), AddressingMode::Direct(Register::A)),
    inst_inc!(0x13, AddressingMode::Direct(Register::DE)),
    inst_inc!(0x14, AddressingMode::Direct(Register::D)),
    inst_dec!(0x15, AddressingMode::Direct(Register::D)),
    inst_ld!(0x16, AddressingMode::Direct(Register::D), AddressingMode::PC1),
    inst_jr!(0x18),
    inst_add!(0x19, AddressingMode::Direct(Register::HL), AddressingMode::Direct(Register::DE)),
    inst_ld!(0x1A, AddressingMode::Direct(Register::A), AddressingMode::Indirect(Register::DE)),
    inst_dec!(0x1B, AddressingMode::Direct(Register::DE)),
    inst_inc!(0x1C, AddressingMode::Direct(Register::E)),
    inst_dec!(0x1D, AddressingMode::Direct(Register::E)),
    inst_ld!(0x1E, AddressingMode::Direct(Register::E), AddressingMode::PC1),
    // 0x2x
    inst_jr!(0x20, Condition::NZ),
    inst_ld!(0x21, AddressingMode::Direct(Register::HL), AddressingMode::PC2),
    inst_ld!(0x22, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::A)),
    inst_inc!(0x23, AddressingMode::Direct(Register::HL)),
    inst_inc!(0x24, AddressingMode::Direct(Register::H)),
    inst_dec!(0x25, AddressingMode::Direct(Register::H)),
    inst_ld!(0x26, AddressingMode::Direct(Register::H), AddressingMode::PC1),
    inst_jr!(0x28, Condition::Z),
    inst_add!(0x29, AddressingMode::Direct(Register::HL), AddressingMode::Direct(Register::HL)),
    inst_ld!(0x2A, AddressingMode::Direct(Register::A), AddressingMode::Indirect(Register::HL)),
    inst_dec!(0x2B, AddressingMode::Direct(Register::HL)),
    inst_inc!(0x2C, AddressingMode::Direct(Register::L)),
    inst_dec!(0x2D, AddressingMode::Direct(Register::L)),
    inst_ld!(0x2E, AddressingMode::Direct(Register::L), AddressingMode::PC1),
    // 0x3x
    inst_jr!(0x30, Condition::NC),
    inst_ld!(0x31, AddressingMode::Direct(Register::SP), AddressingMode::PC2),
    inst_ld!(0x32, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::A)),
    inst_inc!(0x33, AddressingMode::Direct(Register::SP)),
    inst_inc!(0x34, AddressingMode::Indirect(Register::HL)),
    inst_dec!(0x35, AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x36, AddressingMode::Indirect(Register::HL), AddressingMode::PC1),
    inst_jr!(0x38, Condition::C),
    inst_add!(0x39, AddressingMode::Direct(Register::HL), AddressingMode::Direct(Register::SP)),
    inst_ld!(0x3A, AddressingMode::Direct(Register::A), AddressingMode::Indirect(Register::HL)),
    inst_dec!(0x3B, AddressingMode::Direct(Register::SP)),
    inst_inc!(0x3C, AddressingMode::Direct(Register::A)),
    inst_dec!(0x3D, AddressingMode::Direct(Register::A)),
    inst_ld!(0x3E, AddressingMode::Direct(Register::A), AddressingMode::PC1),
    // 0x4x
    inst_ld!(0x40, AddressingMode::Direct(Register::B), AddressingMode::Direct(Register::B)),
    inst_ld!(0x41, AddressingMode::Direct(Register::B), AddressingMode::Direct(Register::C)),
    inst_ld!(0x42, AddressingMode::Direct(Register::B), AddressingMode::Direct(Register::D)),
    inst_ld!(0x43, AddressingMode::Direct(Register::B), AddressingMode::Direct(Register::E)),
    inst_ld!(0x44, AddressingMode::Direct(Register::B), AddressingMode::Direct(Register::H)),
    inst_ld!(0x45, AddressingMode::Direct(Register::B), AddressingMode::Direct(Register::L)),
    inst_ld!(0x46, AddressingMode::Direct(Register::B), AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x47, AddressingMode::Direct(Register::B), AddressingMode::Direct(Register::A)),
    inst_ld!(0x48, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::B)),
    inst_ld!(0x49, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::C)),
    inst_ld!(0x4A, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::D)),
    inst_ld!(0x4B, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::E)),
    inst_ld!(0x4C, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::H)),
    inst_ld!(0x4D, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::L)),
    inst_ld!(0x4E, AddressingMode::Direct(Register::C), AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x4F, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::A)),
    // 0x5x
    inst_ld!(0x50, AddressingMode::Direct(Register::D), AddressingMode::Direct(Register::B)),
    inst_ld!(0x51, AddressingMode::Direct(Register::D), AddressingMode::Direct(Register::C)),
    inst_ld!(0x52, AddressingMode::Direct(Register::D), AddressingMode::Direct(Register::D)),
    inst_ld!(0x53, AddressingMode::Direct(Register::D), AddressingMode::Direct(Register::E)),
    inst_ld!(0x54, AddressingMode::Direct(Register::D), AddressingMode::Direct(Register::H)),
    inst_ld!(0x55, AddressingMode::Direct(Register::D), AddressingMode::Direct(Register::L)),
    inst_ld!(0x56, AddressingMode::Direct(Register::D), AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x57, AddressingMode::Direct(Register::D), AddressingMode::Direct(Register::A)),
    inst_ld!(0x58, AddressingMode::Direct(Register::E), AddressingMode::Direct(Register::B)),
    inst_ld!(0x59, AddressingMode::Direct(Register::E), AddressingMode::Direct(Register::C)),
    inst_ld!(0x5A, AddressingMode::Direct(Register::E), AddressingMode::Direct(Register::D)),
    inst_ld!(0x5B, AddressingMode::Direct(Register::E), AddressingMode::Direct(Register::E)),
    inst_ld!(0x5C, AddressingMode::Direct(Register::E), AddressingMode::Direct(Register::H)),
    inst_ld!(0x5D, AddressingMode::Direct(Register::E), AddressingMode::Direct(Register::L)),
    inst_ld!(0x5E, AddressingMode::Direct(Register::E), AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x5F, AddressingMode::Direct(Register::E), AddressingMode::Direct(Register::A)),
    // 0x6x
    inst_ld!(0x60, AddressingMode::Direct(Register::H), AddressingMode::Direct(Register::B)),
    inst_ld!(0x61, AddressingMode::Direct(Register::H), AddressingMode::Direct(Register::C)),
    inst_ld!(0x62, AddressingMode::Direct(Register::H), AddressingMode::Direct(Register::D)),
    inst_ld!(0x63, AddressingMode::Direct(Register::H), AddressingMode::Direct(Register::E)),
    inst_ld!(0x64, AddressingMode::Direct(Register::H), AddressingMode::Direct(Register::H)),
    inst_ld!(0x65, AddressingMode::Direct(Register::H), AddressingMode::Direct(Register::L)),
    inst_ld!(0x66, AddressingMode::Direct(Register::H), AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x67, AddressingMode::Direct(Register::H), AddressingMode::Direct(Register::A)),
    inst_ld!(0x68, AddressingMode::Direct(Register::L), AddressingMode::Direct(Register::B)),
    inst_ld!(0x69, AddressingMode::Direct(Register::L), AddressingMode::Direct(Register::C)),
    inst_ld!(0x6A, AddressingMode::Direct(Register::L), AddressingMode::Direct(Register::D)),
    inst_ld!(0x6B, AddressingMode::Direct(Register::L), AddressingMode::Direct(Register::E)),
    inst_ld!(0x6C, AddressingMode::Direct(Register::L), AddressingMode::Direct(Register::H)),
    inst_ld!(0x6D, AddressingMode::Direct(Register::L), AddressingMode::Direct(Register::L)),
    inst_ld!(0x6E, AddressingMode::Direct(Register::L), AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x6F, AddressingMode::Direct(Register::L), AddressingMode::Direct(Register::A)),
    // 0x7x
    inst_ld!(0x70, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::B)),
    inst_ld!(0x71, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::C)),
    inst_ld!(0x72, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::D)),
    inst_ld!(0x73, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::E)),
    inst_ld!(0x74, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::H)),
    inst_ld!(0x75, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::L)),
    inst_ld!(0x77, AddressingMode::Indirect(Register::HL), AddressingMode::Direct(Register::A)),
    inst_ld!(0x78, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::B)),
    inst_simple!(0x76, InstructionType::HALT),
    inst_ld!(0x79, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::C)),
    inst_ld!(0x7A, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::D)),
    inst_ld!(0x7B, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::E)),
    inst_ld!(0x7C, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::H)),
    inst_ld!(0x7D, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::L)),
    inst_ld!(0x7E, AddressingMode::Direct(Register::A), AddressingMode::Indirect(Register::HL)),
    inst_ld!(0x7F, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::A)),
    // 0x8x
    inst_add!(0x80, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::B)),
    inst_add!(0x81, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::C)),
    inst_add!(0x82, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::D)),
    inst_add!(0x83, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::E)),
    inst_add!(0x84, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::H)),
    inst_add!(0x85, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::L)),
    inst_add!(0x86, AddressingMode::Direct(Register::A), AddressingMode::Indirect(Register::HL)),
    inst_add!(0x87, AddressingMode::Direct(Register::A), AddressingMode::Direct(Register::A)),
    // 0x9x
    inst_sub!(0x90, AddressingMode::Direct(Register::B)),
    inst_sub!(0x91, AddressingMode::Direct(Register::C)),
    inst_sub!(0x92, AddressingMode::Direct(Register::D)),
    inst_sub!(0x93, AddressingMode::Direct(Register::E)),
    inst_sub!(0x94, AddressingMode::Direct(Register::H)),
    inst_sub!(0x95, AddressingMode::Direct(Register::L)),
    inst_sub!(0x96, AddressingMode::Indirect(Register::HL)),
    inst_sub!(0x97, AddressingMode::Direct(Register::A)),
    // 0xAx
    inst_and!(0xA0, AddressingMode::Direct(Register::B)),
    inst_and!(0xA1, AddressingMode::Direct(Register::C)),
    inst_and!(0xA2, AddressingMode::Direct(Register::D)),
    inst_and!(0xA3, AddressingMode::Direct(Register::E)),
    inst_and!(0xA4, AddressingMode::Direct(Register::H)),
    inst_and!(0xA5, AddressingMode::Direct(Register::L)),
    inst_and!(0xA6, AddressingMode::Indirect(Register::HL)),
    inst_and!(0xA7, AddressingMode::Direct(Register::A)),
    inst_xor!(0xB8, AddressingMode::Direct(Register::B)),
    inst_xor!(0xA9, AddressingMode::Direct(Register::C)),
    inst_xor!(0xAA, AddressingMode::Direct(Register::D)),
    inst_xor!(0xAB, AddressingMode::Direct(Register::E)),
    inst_xor!(0xAC, AddressingMode::Direct(Register::H)),
    inst_xor!(0xAD, AddressingMode::Direct(Register::L)),
    inst_xor!(0xAE, AddressingMode::Indirect(Register::HL)),
    inst_xor!(0xAF, AddressingMode::Direct(Register::A)),
    // 0xBx
    inst_or!(0xB0, AddressingMode::Direct(Register::B)),
    inst_or!(0xB1, AddressingMode::Direct(Register::C)),
    inst_or!(0xB2, AddressingMode::Direct(Register::D)),
    inst_or!(0xB3, AddressingMode::Direct(Register::E)),
    inst_or!(0xB4, AddressingMode::Direct(Register::H)),
    inst_or!(0xB5, AddressingMode::Direct(Register::L)),
    inst_or!(0xB6, AddressingMode::Indirect(Register::HL)),
    inst_or!(0xB7, AddressingMode::Direct(Register::A)),
    // 0xCx
    inst_ret!(0xC0, Condition::NZ),
    inst_pop!(0xC1, AddressingMode::Direct(Register::BC)),
    inst_jp!(0xC2, AddressingMode::PC2, Condition::NZ),
    inst_jp!(0xC3, AddressingMode::PC2),
    inst_call!(0xC4, Condition::NZ),
    inst_push!(0xC5, AddressingMode::Direct(Register::BC)),
    inst_add!(0xC6, AddressingMode::Direct(Register::A), AddressingMode::PC1),
    inst_rst!(0xC7),
    inst_ret!(0xC8, Condition::Z),
    inst_ret!(0xC9),
    inst_jp!(0xCA, AddressingMode::PC2, Condition::Z),
    inst_call!(0xCC, Condition::Z),
    inst_call!(0xCD),
    inst_rst!(0xCF),
    // 0xDx
    inst_ret!(0xD0, Condition::NC),
    inst_pop!(0xD1, AddressingMode::Direct(Register::DE)),
    inst_jp!(0xD2, AddressingMode::PC2, Condition::NZ),
    inst_call!(0xD4, Condition::NC),
    inst_push!(0xD5, AddressingMode::Direct(Register::DE)),
    inst_sub!(0xD6, AddressingMode::PC1),
    inst_rst!(0xD7),
    inst_ret!(0xD8, Condition::C),
    inst_reti!(0xD9),
    inst_jp!(0xDA, AddressingMode::PC2, Condition::C),
    inst_call!(0xDC, Condition::C),
    inst_rst!(0xDF),
    // 0xEx
    inst_ld!(0xE0, AddressingMode::PC1, AddressingMode::Direct(Register::A)),
    inst_pop!(0xE1, AddressingMode::Direct(Register::HL)),
    inst_ld!(0xE2, AddressingMode::Direct(Register::C), AddressingMode::Direct(Register::A)),
    inst_push!(0xE5, AddressingMode::Direct(Register::HL)),
    inst_and!(0xE6, AddressingMode::PC1),
    inst_rst!(0xE7),
    inst_add!(0xE8, AddressingMode::Direct(Register::SP), AddressingMode::PC1),
    inst_jp!(0xE9, AddressingMode::Indirect(Register::HL)),
    inst_ld!(0xEA, AddressingMode::PC2, AddressingMode::Direct(Register::A)),
    inst_xor!(0xEE, AddressingMode::PC1),
    inst_rst!(0xEF),
    // 0xFx
    inst_ld!(0xF0, AddressingMode::Direct(Register::A), AddressingMode::PC1),
    inst_pop!(0xF1, AddressingMode::Direct(Register::AF)),
    inst_ld!(0xF2, AddressingMode::Direct(Register::A), AddressingMode::Indirect(Register::C)),
    inst_simple!(0xF3, InstructionType::DI),
    inst_push!(0xF5, AddressingMode::Direct(Register::AF)),
    inst_or!(0xF6, AddressingMode::PC1),
    inst_rst!(0xF7),
    inst_ld!(
        0xF8,
        AddressingMode::Direct(Register::HL),
        AddressingMode::Direct(Register::SP) /* SP + r8 */
    ),
    inst_ld!(0xF9, AddressingMode::Direct(Register::SP), AddressingMode::Direct(Register::HL)),
    inst_ld!(0xFA, AddressingMode::Direct(Register::A), AddressingMode::PC2),
    inst_simple!(0xFB, InstructionType::EI),
    inst_rst!(0xFF),
];

#[inline]
pub(crate) fn get_instruction(opcode: u8) -> &'static Instruction {
    // TODO index accessing
    INSTRUCTIONS
        .iter()
        .find(|it| it.opcode == opcode)
        .expect(&format!("Expect an instruction whose opcode is {:#02X}", opcode))
}
