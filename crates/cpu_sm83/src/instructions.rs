type Opcode = u16;

pub(crate) struct Instruction {
    pub(crate) opcode: Opcode,
    pub(crate) instr_ty: InstructionType,
    pub(crate) cond: Option<Condition>,
    pub(crate) operand1: Option<Address>,
    pub(crate) operand2: Option<Address>,
}

pub(crate) enum InstructionType {
    NOP,
    LD,
    INC,
    DEC,
    ADD,
    JR,
    JP,
    // TODO
}

pub(crate) enum Address {
    R(Register),
    RM(Register),
    PC1, // 1 byte
    PC2, // 2 bytes
}

pub(crate) enum Condition {
    Z,
    NZ,
    C,
    NC,
}

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
    SP_R8,
}

macro_rules! instr_ld {
    ($opcode:expr, $op1:expr, $op2:expr) => {
        Instruction {
            opcode: $opcode,
            instr_ty: InstructionType::LD,
            cond: None,
            operand1: Some($op1),
            operand2: Some($op2),
        }
    };
}

macro_rules! instr_inc {
    ($opcode:expr, $op1:expr) => {
        Instruction {
            opcode: $opcode,
            instr_ty: InstructionType::INC,
            cond: None,
            operand1: Some($op1),
            operand2: None,
        }
    };
}

macro_rules! instr_dec {
    ($opcode:expr, $op1:expr) => {
        Instruction {
            opcode: $opcode,
            instr_ty: InstructionType::DEC,
            cond: None,
            operand1: Some($op1),
            operand2: None,
        }
    };
}

macro_rules! instr_jp {
    ($opcode:expr, $op1:expr, $cond:expr) => {
        Instruction {
            opcode: $opcode,
            instr_ty: InstructionType::JP,
            cond: Some($cond),
            operand1: Some($op1),
            operand2: None,
        }
    };
    ($opcode:expr, $op1:expr) => {
        Instruction {
            opcode: $opcode,
            instr_ty: InstructionType::JP,
            cond: None,
            operand1: Some($op1),
            operand2: None,
        }
    };
}

macro_rules! instr_jr {
    ($opcode:expr, $op1:expr, $cond:expr) => {
        Instruction {
            opcode: $opcode,
            instr_ty: InstructionType::JR,
            cond: Some($cond),
            operand1: Some($op1),
            operand2: None,
        }
    };
    ($opcode:expr, $op1:expr) => {
        Instruction {
            opcode: $opcode,
            instr_ty: InstructionType::JR,
            cond: None,
            operand1: Some($op1),
            operand2: None,
        }
    };
}

pub(crate) const INSTRUCTIONS: [Instruction; 126] = [
    // 0x0x
    Instruction {
        opcode: 0x00,
        instr_ty: InstructionType::NOP,
        cond: None,
        operand1: None,
        operand2: None,
    },
    instr_ld!(0x01, Address::R(Register::BC), Address::PC2),
    instr_ld!(0x02, Address::RM(Register::BC), Address::R(Register::A)),
    instr_inc!(0x03, Address::R(Register::BC)),
    instr_inc!(0x04, Address::R(Register::B)),
    instr_dec!(0x05, Address::R(Register::B)),
    instr_ld!(0x06, Address::R(Register::B), Address::PC1),
    instr_ld!(0x08, Address::PC2, Address::R(Register::SP)),
    instr_ld!(0x0A, Address::R(Register::A), Address::RM(Register::BC)),
    instr_dec!(0x0B, Address::R(Register::BC)),
    instr_ld!(0x0E, Address::R(Register::C), Address::PC1),
    instr_inc!(0x0C, Address::R(Register::C)),
    instr_dec!(0x0D, Address::R(Register::C)),
    // 0x1x
    instr_ld!(0x11, Address::R(Register::DE), Address::PC2),
    instr_ld!(0x12, Address::RM(Register::DE), Address::R(Register::A)),
    instr_inc!(0x13, Address::R(Register::DE)),
    instr_inc!(0x14, Address::R(Register::D)),
    instr_dec!(0x15, Address::R(Register::D)),
    instr_ld!(0x16, Address::R(Register::D), Address::PC1),
    instr_jr!(0x18, Address::PC1),
    instr_ld!(0x1A, Address::R(Register::A), Address::RM(Register::DE)),
    instr_dec!(0x1B, Address::R(Register::DE)),
    instr_inc!(0x1C, Address::R(Register::E)),
    instr_dec!(0x1D, Address::R(Register::E)),
    instr_ld!(0x1E, Address::R(Register::E), Address::PC1),
    // 0x2x
    instr_jr!(0x20, Address::PC1, Condition::NZ),
    instr_ld!(0x21, Address::R(Register::HL), Address::PC2),
    instr_ld!(0x22, Address::RM(Register::HL), Address::R(Register::A)),
    instr_inc!(0x23, Address::R(Register::HL)),
    instr_inc!(0x24, Address::R(Register::H)),
    instr_dec!(0x25, Address::R(Register::H)),
    instr_ld!(0x26, Address::R(Register::H), Address::PC1),
    instr_jr!(0x28, Address::PC1, Condition::Z),
    instr_ld!(0x2A, Address::R(Register::A), Address::RM(Register::HL)),
    instr_dec!(0x2B, Address::R(Register::HL)),
    instr_inc!(0x2C, Address::R(Register::L)),
    instr_dec!(0x2D, Address::R(Register::L)),
    instr_ld!(0x2E, Address::R(Register::L), Address::PC1),
    // 0x3x
    instr_jr!(0x20, Address::PC1, Condition::NC),
    instr_ld!(0x31, Address::R(Register::SP), Address::PC2),
    instr_ld!(0x32, Address::RM(Register::HL), Address::R(Register::A)),
    instr_inc!(0x33, Address::R(Register::SP)),
    instr_inc!(0x34, Address::RM(Register::HL)),
    instr_dec!(0x35, Address::RM(Register::HL)),
    instr_ld!(0x36, Address::RM(Register::HL), Address::PC1),
    instr_jr!(0x28, Address::PC1, Condition::C),
    instr_ld!(0x3A, Address::R(Register::A), Address::RM(Register::HL)),
    instr_dec!(0x3B, Address::R(Register::SP)),
    instr_inc!(0x3C, Address::R(Register::A)),
    instr_dec!(0x3D, Address::R(Register::A)),
    instr_ld!(0x3E, Address::R(Register::A), Address::PC1),
    // 0x4x
    instr_ld!(0x40, Address::R(Register::B), Address::R(Register::B)),
    instr_ld!(0x41, Address::R(Register::B), Address::R(Register::C)),
    instr_ld!(0x42, Address::R(Register::B), Address::R(Register::D)),
    instr_ld!(0x43, Address::R(Register::B), Address::R(Register::E)),
    instr_ld!(0x44, Address::R(Register::B), Address::R(Register::H)),
    instr_ld!(0x45, Address::R(Register::B), Address::R(Register::L)),
    instr_ld!(0x46, Address::R(Register::B), Address::RM(Register::HL)),
    instr_ld!(0x47, Address::R(Register::B), Address::R(Register::A)),
    instr_ld!(0x48, Address::R(Register::C), Address::R(Register::B)),
    instr_ld!(0x49, Address::R(Register::C), Address::R(Register::C)),
    instr_ld!(0x4A, Address::R(Register::C), Address::R(Register::D)),
    instr_ld!(0x4B, Address::R(Register::C), Address::R(Register::E)),
    instr_ld!(0x4C, Address::R(Register::C), Address::R(Register::H)),
    instr_ld!(0x4D, Address::R(Register::C), Address::R(Register::L)),
    instr_ld!(0x4E, Address::R(Register::C), Address::RM(Register::HL)),
    instr_ld!(0x4F, Address::R(Register::C), Address::R(Register::A)),
    // 0x5x
    instr_ld!(0x50, Address::R(Register::D), Address::R(Register::B)),
    instr_ld!(0x51, Address::R(Register::D), Address::R(Register::C)),
    instr_ld!(0x52, Address::R(Register::D), Address::R(Register::D)),
    instr_ld!(0x53, Address::R(Register::D), Address::R(Register::E)),
    instr_ld!(0x54, Address::R(Register::D), Address::R(Register::H)),
    instr_ld!(0x55, Address::R(Register::D), Address::R(Register::L)),
    instr_ld!(0x56, Address::R(Register::D), Address::RM(Register::HL)),
    instr_ld!(0x57, Address::R(Register::D), Address::R(Register::A)),
    instr_ld!(0x58, Address::R(Register::E), Address::R(Register::B)),
    instr_ld!(0x59, Address::R(Register::E), Address::R(Register::C)),
    instr_ld!(0x5A, Address::R(Register::E), Address::R(Register::D)),
    instr_ld!(0x5B, Address::R(Register::E), Address::R(Register::E)),
    instr_ld!(0x5C, Address::R(Register::E), Address::R(Register::H)),
    instr_ld!(0x5D, Address::R(Register::E), Address::R(Register::L)),
    instr_ld!(0x5E, Address::R(Register::E), Address::RM(Register::HL)),
    instr_ld!(0x5F, Address::R(Register::E), Address::R(Register::A)),
    // 0x6x
    instr_ld!(0x60, Address::R(Register::H), Address::R(Register::B)),
    instr_ld!(0x61, Address::R(Register::H), Address::R(Register::C)),
    instr_ld!(0x62, Address::R(Register::H), Address::R(Register::D)),
    instr_ld!(0x63, Address::R(Register::H), Address::R(Register::E)),
    instr_ld!(0x64, Address::R(Register::H), Address::R(Register::H)),
    instr_ld!(0x65, Address::R(Register::H), Address::R(Register::L)),
    instr_ld!(0x66, Address::R(Register::H), Address::RM(Register::HL)),
    instr_ld!(0x67, Address::R(Register::H), Address::R(Register::A)),
    instr_ld!(0x68, Address::R(Register::L), Address::R(Register::B)),
    instr_ld!(0x69, Address::R(Register::L), Address::R(Register::C)),
    instr_ld!(0x6A, Address::R(Register::L), Address::R(Register::D)),
    instr_ld!(0x6B, Address::R(Register::L), Address::R(Register::E)),
    instr_ld!(0x6C, Address::R(Register::L), Address::R(Register::H)),
    instr_ld!(0x6D, Address::R(Register::L), Address::R(Register::L)),
    instr_ld!(0x6E, Address::R(Register::L), Address::RM(Register::HL)),
    instr_ld!(0x6F, Address::R(Register::L), Address::R(Register::A)),
    // 0x7x
    instr_ld!(0x70, Address::RM(Register::HL), Address::R(Register::B)),
    instr_ld!(0x71, Address::RM(Register::HL), Address::R(Register::C)),
    instr_ld!(0x72, Address::RM(Register::HL), Address::R(Register::D)),
    instr_ld!(0x73, Address::RM(Register::HL), Address::R(Register::E)),
    instr_ld!(0x74, Address::RM(Register::HL), Address::R(Register::H)),
    instr_ld!(0x75, Address::RM(Register::HL), Address::R(Register::L)),
    instr_ld!(0x77, Address::RM(Register::HL), Address::R(Register::A)),
    instr_ld!(0x78, Address::R(Register::A), Address::R(Register::B)),
    instr_ld!(0x79, Address::R(Register::A), Address::R(Register::C)),
    instr_ld!(0x7A, Address::R(Register::A), Address::R(Register::D)),
    instr_ld!(0x7B, Address::R(Register::A), Address::R(Register::E)),
    instr_ld!(0x7C, Address::R(Register::A), Address::R(Register::H)),
    instr_ld!(0x7D, Address::R(Register::A), Address::R(Register::L)),
    instr_ld!(0x7E, Address::R(Register::A), Address::RM(Register::HL)),
    instr_ld!(0x7F, Address::R(Register::A), Address::R(Register::A)),
    // 0x8x
    // 0x9x
    // 0xAx
    // 0xBx
    // 0xCx
    instr_jp!(0xC2, Address::PC2, Condition::NZ),
    instr_jp!(0xC3, Address::PC2),
    instr_jp!(0xCA, Address::PC2, Condition::Z),
    // 0xDx
    instr_jp!(0xD2, Address::PC2, Condition::NZ),
    instr_jp!(0xDA, Address::PC2, Condition::C),
    // 0xEx
    instr_ld!(0xE2, Address::R(Register::C), Address::R(Register::A)),
    instr_jp!(0xE9, Address::RM(Register::HL)),
    instr_ld!(0xEA, Address::PC2, Address::R(Register::A)),
    // 0xFx
    instr_ld!(0xF2, Address::R(Register::A), Address::RM(Register::C)),
    instr_ld!(0xF8, Address::R(Register::HL), Address::R(Register::SP_R8)),
    instr_ld!(0xF9, Address::R(Register::SP), Address::R(Register::HL)),
    instr_ld!(0xFA, Address::R(Register::A), Address::PC2),
];
