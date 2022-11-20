#[derive(Debug)]
pub(crate) struct Instruction {
    pub(crate) opcode: u8,
    pub(crate) ty: InstructionType,
    pub(crate) cond: Option<Condition>,
    pub(crate) operand1: Option<AddressingMode>,
    pub(crate) operand2: Option<AddressingMode>,
}

#[derive(Debug)]
pub(crate) enum InstructionType {
    NOP,
    LD,
    INC,
    DEC,
    JR,
    JP,
    // TODO
}

/// Addressing mode
#[derive(Debug)]
pub(crate) enum AddressingMode {
    /// Register direct
    Rd(Register),
    /// Register indirect
    Ri(Register),
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
        Instruction {
            opcode: $opcode,
            ty: InstructionType::INC,
            cond: None,
            operand1: Some($op1),
            operand2: None,
        }
    };
}

macro_rules! inst_dec {
    ($opcode:expr, $op1:expr) => {
        Instruction {
            opcode: $opcode,
            ty: InstructionType::DEC,
            cond: None,
            operand1: Some($op1),
            operand2: None,
        }
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
        Instruction {
            opcode: $opcode,
            ty: InstructionType::JP,
            cond: None,
            operand1: Some($op1),
            operand2: None,
        }
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
        Instruction {
            opcode: $opcode,
            ty: InstructionType::JR,
            cond: None,
            operand1: Some(AddressingMode::PC1),
            operand2: None,
        }
    };
}

const INSTRUCTIONS: [Instruction; 128] = [
    // 0x0x
    Instruction {
        opcode: 0x00,
        ty: InstructionType::NOP,
        cond: None,
        operand1: None,
        operand2: None,
    },
    inst_ld!(0x01, AddressingMode::Rd(Register::BC), AddressingMode::PC2),
    inst_ld!(0x02, AddressingMode::Ri(Register::BC), AddressingMode::Rd(Register::A)),
    inst_inc!(0x03, AddressingMode::Rd(Register::BC)),
    inst_inc!(0x04, AddressingMode::Rd(Register::B)),
    inst_dec!(0x05, AddressingMode::Rd(Register::B)),
    inst_ld!(0x06, AddressingMode::Rd(Register::B), AddressingMode::PC1),
    inst_ld!(0x08, AddressingMode::PC2, AddressingMode::Rd(Register::SP)),
    inst_ld!(0x0A, AddressingMode::Rd(Register::A), AddressingMode::Ri(Register::BC)),
    inst_dec!(0x0B, AddressingMode::Rd(Register::BC)),
    inst_ld!(0x0E, AddressingMode::Rd(Register::C), AddressingMode::PC1),
    inst_inc!(0x0C, AddressingMode::Rd(Register::C)),
    inst_dec!(0x0D, AddressingMode::Rd(Register::C)),
    // 0x1x
    inst_ld!(0x11, AddressingMode::Rd(Register::DE), AddressingMode::PC2),
    inst_ld!(0x12, AddressingMode::Ri(Register::DE), AddressingMode::Rd(Register::A)),
    inst_inc!(0x13, AddressingMode::Rd(Register::DE)),
    inst_inc!(0x14, AddressingMode::Rd(Register::D)),
    inst_dec!(0x15, AddressingMode::Rd(Register::D)),
    inst_ld!(0x16, AddressingMode::Rd(Register::D), AddressingMode::PC1),
    inst_jr!(0x18),
    inst_ld!(0x1A, AddressingMode::Rd(Register::A), AddressingMode::Ri(Register::DE)),
    inst_dec!(0x1B, AddressingMode::Rd(Register::DE)),
    inst_inc!(0x1C, AddressingMode::Rd(Register::E)),
    inst_dec!(0x1D, AddressingMode::Rd(Register::E)),
    inst_ld!(0x1E, AddressingMode::Rd(Register::E), AddressingMode::PC1),
    // 0x2x
    inst_jr!(0x20, Condition::NZ),
    inst_ld!(0x21, AddressingMode::Rd(Register::HL), AddressingMode::PC2),
    inst_ld!(0x22, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::A)),
    inst_inc!(0x23, AddressingMode::Rd(Register::HL)),
    inst_inc!(0x24, AddressingMode::Rd(Register::H)),
    inst_dec!(0x25, AddressingMode::Rd(Register::H)),
    inst_ld!(0x26, AddressingMode::Rd(Register::H), AddressingMode::PC1),
    inst_jr!(0x28, Condition::Z),
    inst_ld!(0x2A, AddressingMode::Rd(Register::A), AddressingMode::Ri(Register::HL)),
    inst_dec!(0x2B, AddressingMode::Rd(Register::HL)),
    inst_inc!(0x2C, AddressingMode::Rd(Register::L)),
    inst_dec!(0x2D, AddressingMode::Rd(Register::L)),
    inst_ld!(0x2E, AddressingMode::Rd(Register::L), AddressingMode::PC1),
    // 0x3x
    inst_jr!(0x20, Condition::NC),
    inst_ld!(0x31, AddressingMode::Rd(Register::SP), AddressingMode::PC2),
    inst_ld!(0x32, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::A)),
    inst_inc!(0x33, AddressingMode::Rd(Register::SP)),
    inst_inc!(0x34, AddressingMode::Ri(Register::HL)),
    inst_dec!(0x35, AddressingMode::Ri(Register::HL)),
    inst_ld!(0x36, AddressingMode::Ri(Register::HL), AddressingMode::PC1),
    inst_jr!(0x28, Condition::C),
    inst_ld!(0x3A, AddressingMode::Rd(Register::A), AddressingMode::Ri(Register::HL)),
    inst_dec!(0x3B, AddressingMode::Rd(Register::SP)),
    inst_inc!(0x3C, AddressingMode::Rd(Register::A)),
    inst_dec!(0x3D, AddressingMode::Rd(Register::A)),
    inst_ld!(0x3E, AddressingMode::Rd(Register::A), AddressingMode::PC1),
    // 0x4x
    inst_ld!(0x40, AddressingMode::Rd(Register::B), AddressingMode::Rd(Register::B)),
    inst_ld!(0x41, AddressingMode::Rd(Register::B), AddressingMode::Rd(Register::C)),
    inst_ld!(0x42, AddressingMode::Rd(Register::B), AddressingMode::Rd(Register::D)),
    inst_ld!(0x43, AddressingMode::Rd(Register::B), AddressingMode::Rd(Register::E)),
    inst_ld!(0x44, AddressingMode::Rd(Register::B), AddressingMode::Rd(Register::H)),
    inst_ld!(0x45, AddressingMode::Rd(Register::B), AddressingMode::Rd(Register::L)),
    inst_ld!(0x46, AddressingMode::Rd(Register::B), AddressingMode::Ri(Register::HL)),
    inst_ld!(0x47, AddressingMode::Rd(Register::B), AddressingMode::Rd(Register::A)),
    inst_ld!(0x48, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::B)),
    inst_ld!(0x49, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::C)),
    inst_ld!(0x4A, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::D)),
    inst_ld!(0x4B, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::E)),
    inst_ld!(0x4C, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::H)),
    inst_ld!(0x4D, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::L)),
    inst_ld!(0x4E, AddressingMode::Rd(Register::C), AddressingMode::Ri(Register::HL)),
    inst_ld!(0x4F, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::A)),
    // 0x5x
    inst_ld!(0x50, AddressingMode::Rd(Register::D), AddressingMode::Rd(Register::B)),
    inst_ld!(0x51, AddressingMode::Rd(Register::D), AddressingMode::Rd(Register::C)),
    inst_ld!(0x52, AddressingMode::Rd(Register::D), AddressingMode::Rd(Register::D)),
    inst_ld!(0x53, AddressingMode::Rd(Register::D), AddressingMode::Rd(Register::E)),
    inst_ld!(0x54, AddressingMode::Rd(Register::D), AddressingMode::Rd(Register::H)),
    inst_ld!(0x55, AddressingMode::Rd(Register::D), AddressingMode::Rd(Register::L)),
    inst_ld!(0x56, AddressingMode::Rd(Register::D), AddressingMode::Ri(Register::HL)),
    inst_ld!(0x57, AddressingMode::Rd(Register::D), AddressingMode::Rd(Register::A)),
    inst_ld!(0x58, AddressingMode::Rd(Register::E), AddressingMode::Rd(Register::B)),
    inst_ld!(0x59, AddressingMode::Rd(Register::E), AddressingMode::Rd(Register::C)),
    inst_ld!(0x5A, AddressingMode::Rd(Register::E), AddressingMode::Rd(Register::D)),
    inst_ld!(0x5B, AddressingMode::Rd(Register::E), AddressingMode::Rd(Register::E)),
    inst_ld!(0x5C, AddressingMode::Rd(Register::E), AddressingMode::Rd(Register::H)),
    inst_ld!(0x5D, AddressingMode::Rd(Register::E), AddressingMode::Rd(Register::L)),
    inst_ld!(0x5E, AddressingMode::Rd(Register::E), AddressingMode::Ri(Register::HL)),
    inst_ld!(0x5F, AddressingMode::Rd(Register::E), AddressingMode::Rd(Register::A)),
    // 0x6x
    inst_ld!(0x60, AddressingMode::Rd(Register::H), AddressingMode::Rd(Register::B)),
    inst_ld!(0x61, AddressingMode::Rd(Register::H), AddressingMode::Rd(Register::C)),
    inst_ld!(0x62, AddressingMode::Rd(Register::H), AddressingMode::Rd(Register::D)),
    inst_ld!(0x63, AddressingMode::Rd(Register::H), AddressingMode::Rd(Register::E)),
    inst_ld!(0x64, AddressingMode::Rd(Register::H), AddressingMode::Rd(Register::H)),
    inst_ld!(0x65, AddressingMode::Rd(Register::H), AddressingMode::Rd(Register::L)),
    inst_ld!(0x66, AddressingMode::Rd(Register::H), AddressingMode::Ri(Register::HL)),
    inst_ld!(0x67, AddressingMode::Rd(Register::H), AddressingMode::Rd(Register::A)),
    inst_ld!(0x68, AddressingMode::Rd(Register::L), AddressingMode::Rd(Register::B)),
    inst_ld!(0x69, AddressingMode::Rd(Register::L), AddressingMode::Rd(Register::C)),
    inst_ld!(0x6A, AddressingMode::Rd(Register::L), AddressingMode::Rd(Register::D)),
    inst_ld!(0x6B, AddressingMode::Rd(Register::L), AddressingMode::Rd(Register::E)),
    inst_ld!(0x6C, AddressingMode::Rd(Register::L), AddressingMode::Rd(Register::H)),
    inst_ld!(0x6D, AddressingMode::Rd(Register::L), AddressingMode::Rd(Register::L)),
    inst_ld!(0x6E, AddressingMode::Rd(Register::L), AddressingMode::Ri(Register::HL)),
    inst_ld!(0x6F, AddressingMode::Rd(Register::L), AddressingMode::Rd(Register::A)),
    // 0x7x
    inst_ld!(0x70, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::B)),
    inst_ld!(0x71, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::C)),
    inst_ld!(0x72, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::D)),
    inst_ld!(0x73, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::E)),
    inst_ld!(0x74, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::H)),
    inst_ld!(0x75, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::L)),
    inst_ld!(0x77, AddressingMode::Ri(Register::HL), AddressingMode::Rd(Register::A)),
    inst_ld!(0x78, AddressingMode::Rd(Register::A), AddressingMode::Rd(Register::B)),
    inst_ld!(0x79, AddressingMode::Rd(Register::A), AddressingMode::Rd(Register::C)),
    inst_ld!(0x7A, AddressingMode::Rd(Register::A), AddressingMode::Rd(Register::D)),
    inst_ld!(0x7B, AddressingMode::Rd(Register::A), AddressingMode::Rd(Register::E)),
    inst_ld!(0x7C, AddressingMode::Rd(Register::A), AddressingMode::Rd(Register::H)),
    inst_ld!(0x7D, AddressingMode::Rd(Register::A), AddressingMode::Rd(Register::L)),
    inst_ld!(0x7E, AddressingMode::Rd(Register::A), AddressingMode::Ri(Register::HL)),
    inst_ld!(0x7F, AddressingMode::Rd(Register::A), AddressingMode::Rd(Register::A)),
    // 0x8x
    // 0x9x
    // 0xAx
    // 0xBx
    // 0xCx
    inst_jp!(0xC2, AddressingMode::PC2, Condition::NZ),
    inst_jp!(0xC3, AddressingMode::PC2),
    inst_jp!(0xCA, AddressingMode::PC2, Condition::Z),
    // 0xDx
    inst_jp!(0xD2, AddressingMode::PC2, Condition::NZ),
    inst_jp!(0xDA, AddressingMode::PC2, Condition::C),
    // 0xEx
    inst_ld!(0xE0, AddressingMode::PC1, AddressingMode::Rd(Register::A)),
    inst_ld!(0xE2, AddressingMode::Rd(Register::C), AddressingMode::Rd(Register::A)),
    inst_jp!(0xE9, AddressingMode::Ri(Register::HL)),
    inst_ld!(0xEA, AddressingMode::PC2, AddressingMode::Rd(Register::A)),
    // 0xFx
    inst_ld!(0xF0, AddressingMode::Rd(Register::A), AddressingMode::PC1),
    inst_ld!(0xF2, AddressingMode::Rd(Register::A), AddressingMode::Ri(Register::C)),
    inst_ld!(
        0xF8,
        AddressingMode::Rd(Register::HL),
        AddressingMode::Rd(Register::SP) /* SP + r8 */
    ),
    inst_ld!(0xF9, AddressingMode::Rd(Register::SP), AddressingMode::Rd(Register::HL)),
    inst_ld!(0xFA, AddressingMode::Rd(Register::A), AddressingMode::PC2),
];

#[inline]
pub(crate) fn get_instruction(opcode: u8) -> &'static Instruction {
    // TODO index accessing
    INSTRUCTIONS
        .iter()
        .find(|it| it.opcode == opcode)
        .expect(&format!("Expect an instruction whose opcode is 0x{:04X}", opcode))
}
