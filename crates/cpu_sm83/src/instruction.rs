#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum AddressingMode {
    Direct_A,
    Direct_B,
    Direct_C,
    Direct_D,
    Direct_E,
    Direct_H,
    Direct_L,
    Direct_AF,
    Direct_BC,
    Direct_DE,
    Direct_HL,
    Direct_SP,
    Indirect_BC,
    Indirect_DE,
    Indirect_HL,
    /// PC-relative 1 byte
    PC1,
    /// PC-relative 2 bytes
    PC2,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum Condition {
    Z,
    NZ,
    C,
    NC,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum Instruction {
    NONE,
    NOP,
    LD(AddressingMode, AddressingMode),
    INC(AddressingMode),
    DEC(AddressingMode),
    JR(Option<Condition>),
    JP(Option<Condition>, AddressingMode),
    CALL(Option<Condition>),
    ADD(AddressingMode, AddressingMode),
    ADC(AddressingMode),
    SUB(AddressingMode),
    SBC(AddressingMode),
    PUSH(AddressingMode),
    POP(AddressingMode),
    RET(Option<Condition>),
    RETI,
    RST,
    AND(AddressingMode),
    OR(AddressingMode),
    XOR(AddressingMode),
    STOP,
    DI,
    EI,
    HALT,
    RLA,
    RRA,
    RLCA,
    RRCA,
    DAA,
    CPL,
    SCF,
    CCF,
    CP(AddressingMode),
    CB,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub(crate) enum CbInstruction {
    RLC,
    RRC,
    RL,
    RR,
    SLA,
    SRA,
    SWAP,
    SRL,
    BIT,
    RES,
    SET,
}

type Am = AddressingMode;
type Inst = Instruction;
const INSTRUCTIONS: [Instruction; 256] = [
    // 0x0x
    Inst::NOP,
    Inst::LD(Am::Direct_BC, Am::PC2),
    Inst::LD(Am::Indirect_BC, Am::Direct_A),
    Inst::INC(Am::Direct_BC),
    Inst::INC(Am::Direct_B),
    Inst::DEC(Am::Direct_B),
    Inst::LD(Am::Direct_B, Am::PC1),
    Inst::RLCA,
    Inst::LD(Am::PC2, Am::Direct_SP),
    Inst::ADD(Am::Direct_HL, Am::Direct_BC),
    Inst::LD(Am::Direct_A, Am::Indirect_BC),
    Inst::DEC(Am::Direct_BC),
    Inst::INC(Am::Direct_C),
    Inst::DEC(Am::Direct_C),
    Inst::LD(Am::Direct_C, Am::PC1),
    Inst::RRCA,
    // 0x1x
    Inst::STOP,
    Inst::LD(Am::Direct_DE, Am::PC2),
    Inst::LD(Am::Indirect_DE, Am::Direct_A),
    Inst::INC(Am::Direct_DE),
    Inst::INC(Am::Direct_D),
    Inst::DEC(Am::Direct_D),
    Inst::LD(Am::Direct_D, Am::PC1),
    Inst::RLA,
    Inst::JR(None),
    Inst::ADD(Am::Direct_HL, Am::Direct_DE),
    Inst::LD(Am::Direct_A, Am::Indirect_DE),
    Inst::DEC(Am::Direct_DE),
    Inst::INC(Am::Direct_E),
    Inst::DEC(Am::Direct_E),
    Inst::LD(Am::Direct_E, Am::PC1),
    Inst::RRA,
    // 0x2x
    Inst::JR(Some(Condition::NZ)),
    Inst::LD(Am::Direct_HL, Am::PC2),
    Inst::LD(Am::Indirect_HL, Am::Direct_A),
    Inst::INC(Am::Direct_HL),
    Inst::INC(Am::Direct_H),
    Inst::DEC(Am::Direct_H),
    Inst::LD(Am::Direct_H, Am::PC1),
    Inst::DAA,
    Inst::JR(Some(Condition::Z)),
    Inst::ADD(Am::Direct_HL, Am::Direct_HL),
    Inst::LD(Am::Direct_A, Am::Indirect_HL),
    Inst::DEC(Am::Direct_HL),
    Inst::INC(Am::Direct_L),
    Inst::DEC(Am::Direct_L),
    Inst::LD(Am::Direct_L, Am::PC1),
    Inst::CPL,
    // 0x3x
    Inst::JR(Some(Condition::NC)),
    Inst::LD(Am::Direct_SP, Am::PC2),
    Inst::LD(Am::Indirect_HL, Am::Direct_A),
    Inst::INC(Am::Direct_SP),
    Inst::INC(Am::Indirect_HL),
    Inst::DEC(Am::Indirect_HL),
    Inst::LD(Am::Indirect_HL, Am::PC1),
    Inst::SCF,
    Inst::JR(Some(Condition::C)),
    Inst::ADD(Am::Direct_HL, Am::Direct_SP),
    Inst::LD(Am::Direct_A, Am::Indirect_HL),
    Inst::DEC(Am::Direct_SP),
    Inst::INC(Am::Direct_A),
    Inst::DEC(Am::Direct_A),
    Inst::LD(Am::Direct_A, Am::PC1),
    Inst::CCF,
    // 0x4x
    Inst::LD(Am::Direct_B, Am::Direct_B),
    Inst::LD(Am::Direct_B, Am::Direct_C),
    Inst::LD(Am::Direct_B, Am::Direct_D),
    Inst::LD(Am::Direct_B, Am::Direct_E),
    Inst::LD(Am::Direct_B, Am::Direct_H),
    Inst::LD(Am::Direct_B, Am::Direct_L),
    Inst::LD(Am::Direct_B, Am::Indirect_HL),
    Inst::LD(Am::Direct_B, Am::Direct_A),
    Inst::LD(Am::Direct_C, Am::Direct_B),
    Inst::LD(Am::Direct_C, Am::Direct_C),
    Inst::LD(Am::Direct_C, Am::Direct_D),
    Inst::LD(Am::Direct_C, Am::Direct_E),
    Inst::LD(Am::Direct_C, Am::Direct_H),
    Inst::LD(Am::Direct_C, Am::Direct_L),
    Inst::LD(Am::Direct_C, Am::Indirect_HL),
    Inst::LD(Am::Direct_C, Am::Direct_A),
    // 0x5x
    Inst::LD(Am::Direct_D, Am::Direct_B),
    Inst::LD(Am::Direct_D, Am::Direct_C),
    Inst::LD(Am::Direct_D, Am::Direct_D),
    Inst::LD(Am::Direct_D, Am::Direct_E),
    Inst::LD(Am::Direct_D, Am::Direct_H),
    Inst::LD(Am::Direct_D, Am::Direct_L),
    Inst::LD(Am::Direct_D, Am::Indirect_HL),
    Inst::LD(Am::Direct_D, Am::Direct_A),
    Inst::LD(Am::Direct_E, Am::Direct_B),
    Inst::LD(Am::Direct_E, Am::Direct_C),
    Inst::LD(Am::Direct_E, Am::Direct_D),
    Inst::LD(Am::Direct_E, Am::Direct_E),
    Inst::LD(Am::Direct_E, Am::Direct_H),
    Inst::LD(Am::Direct_E, Am::Direct_L),
    Inst::LD(Am::Direct_E, Am::Indirect_HL),
    Inst::LD(Am::Direct_E, Am::Direct_A),
    // 0x6x
    Inst::LD(Am::Direct_H, Am::Direct_B),
    Inst::LD(Am::Direct_H, Am::Direct_C),
    Inst::LD(Am::Direct_H, Am::Direct_D),
    Inst::LD(Am::Direct_H, Am::Direct_E),
    Inst::LD(Am::Direct_H, Am::Direct_H),
    Inst::LD(Am::Direct_H, Am::Direct_L),
    Inst::LD(Am::Direct_H, Am::Indirect_HL),
    Inst::LD(Am::Direct_H, Am::Direct_A),
    Inst::LD(Am::Direct_L, Am::Direct_B),
    Inst::LD(Am::Direct_L, Am::Direct_C),
    Inst::LD(Am::Direct_L, Am::Direct_D),
    Inst::LD(Am::Direct_L, Am::Direct_E),
    Inst::LD(Am::Direct_L, Am::Direct_H),
    Inst::LD(Am::Direct_L, Am::Direct_L),
    Inst::LD(Am::Direct_L, Am::Indirect_HL),
    Inst::LD(Am::Direct_L, Am::Direct_A),
    // 0x7x
    Inst::LD(Am::Indirect_HL, Am::Direct_B),
    Inst::LD(Am::Indirect_HL, Am::Direct_C),
    Inst::LD(Am::Indirect_HL, Am::Direct_D),
    Inst::LD(Am::Indirect_HL, Am::Direct_E),
    Inst::LD(Am::Indirect_HL, Am::Direct_H),
    Inst::LD(Am::Indirect_HL, Am::Direct_L),
    Inst::HALT,
    Inst::LD(Am::Indirect_HL, Am::Direct_A),
    Inst::LD(Am::Direct_A, Am::Direct_B),
    Inst::LD(Am::Direct_A, Am::Direct_C),
    Inst::LD(Am::Direct_A, Am::Direct_D),
    Inst::LD(Am::Direct_A, Am::Direct_E),
    Inst::LD(Am::Direct_A, Am::Direct_H),
    Inst::LD(Am::Direct_A, Am::Direct_L),
    Inst::LD(Am::Direct_A, Am::Indirect_HL),
    Inst::LD(Am::Direct_A, Am::Direct_A),
    // 0x8x
    Inst::ADD(Am::Direct_A, Am::Direct_B),
    Inst::ADD(Am::Direct_A, Am::Direct_C),
    Inst::ADD(Am::Direct_A, Am::Direct_D),
    Inst::ADD(Am::Direct_A, Am::Direct_E),
    Inst::ADD(Am::Direct_A, Am::Direct_H),
    Inst::ADD(Am::Direct_A, Am::Direct_L),
    Inst::ADD(Am::Direct_A, Am::Indirect_HL),
    Inst::ADD(Am::Direct_A, Am::Direct_A),
    Inst::ADC(Am::Direct_B),
    Inst::ADC(Am::Direct_C),
    Inst::ADC(Am::Direct_D),
    Inst::ADC(Am::Direct_E),
    Inst::ADC(Am::Direct_H),
    Inst::ADC(Am::Direct_L),
    Inst::ADC(Am::Indirect_HL),
    Inst::ADC(Am::Direct_A),
    // 0x9x
    Inst::SUB(Am::Direct_B),
    Inst::SUB(Am::Direct_C),
    Inst::SUB(Am::Direct_D),
    Inst::SUB(Am::Direct_E),
    Inst::SUB(Am::Direct_H),
    Inst::SUB(Am::Direct_L),
    Inst::SUB(Am::Indirect_HL),
    Inst::SUB(Am::Direct_A),
    Inst::SBC(Am::Direct_B),
    Inst::SBC(Am::Direct_C),
    Inst::SBC(Am::Direct_D),
    Inst::SBC(Am::Direct_E),
    Inst::SBC(Am::Direct_H),
    Inst::SBC(Am::Direct_L),
    Inst::SBC(Am::Indirect_HL),
    Inst::SBC(Am::Direct_A),
    // 0xAx
    Inst::AND(Am::Direct_B),
    Inst::AND(Am::Direct_C),
    Inst::AND(Am::Direct_D),
    Inst::AND(Am::Direct_E),
    Inst::AND(Am::Direct_H),
    Inst::AND(Am::Direct_L),
    Inst::AND(Am::Indirect_HL),
    Inst::AND(Am::Direct_A),
    Inst::XOR(Am::Direct_B),
    Inst::XOR(Am::Direct_C),
    Inst::XOR(Am::Direct_D),
    Inst::XOR(Am::Direct_E),
    Inst::XOR(Am::Direct_H),
    Inst::XOR(Am::Direct_L),
    Inst::XOR(Am::Indirect_HL),
    Inst::XOR(Am::Direct_A),
    // 0xBx
    Inst::OR(Am::Direct_B),
    Inst::OR(Am::Direct_C),
    Inst::OR(Am::Direct_D),
    Inst::OR(Am::Direct_E),
    Inst::OR(Am::Direct_H),
    Inst::OR(Am::Direct_L),
    Inst::OR(Am::Indirect_HL),
    Inst::OR(Am::Direct_A),
    Inst::CP(Am::Direct_B),
    Inst::CP(Am::Direct_C),
    Inst::CP(Am::Direct_D),
    Inst::CP(Am::Direct_E),
    Inst::CP(Am::Direct_H),
    Inst::CP(Am::Direct_L),
    Inst::CP(Am::Indirect_HL),
    Inst::CP(Am::Direct_A),
    // 0xCx
    Inst::RET(Some(Condition::NZ)),
    Inst::POP(Am::Direct_BC),
    Inst::JP(Some(Condition::NZ), Am::PC2),
    Inst::JP(None, Am::PC2),
    Inst::CALL(Some(Condition::NZ)),
    Inst::PUSH(Am::Direct_BC),
    Inst::ADD(Am::Direct_A, Am::PC1),
    Inst::RST,
    Inst::RET(Some(Condition::Z)),
    Inst::RET(None),
    Inst::JP(Some(Condition::Z), Am::PC2),
    Inst::CB,
    Inst::CALL(Some(Condition::Z)),
    Inst::CALL(None),
    Inst::ADC(Am::PC1),
    Inst::RST,
    // 0xDx
    Inst::RET(Some(Condition::NC)),
    Inst::POP(Am::Direct_DE),
    Inst::JP(Some(Condition::NZ), Am::PC2),
    Inst::NONE,
    Inst::CALL(Some(Condition::NC)),
    Inst::PUSH(Am::Direct_DE),
    Inst::SUB(Am::PC1),
    Inst::RST,
    Inst::RET(Some(Condition::C)),
    Inst::RETI,
    Inst::JP(Some(Condition::C), Am::PC2),
    Inst::NONE,
    Inst::CALL(Some(Condition::C)),
    Inst::NONE,
    Inst::SBC(Am::PC1),
    Inst::RST,
    // 0xEx
    Inst::LD(Am::PC1, Am::Direct_A),
    Inst::POP(Am::Direct_HL),
    Inst::LD(Am::Direct_C, Am::Direct_A),
    Inst::NONE,
    Inst::NONE,
    Inst::PUSH(Am::Direct_HL),
    Inst::AND(Am::PC1),
    Inst::RST,
    Inst::ADD(Am::Direct_SP, Am::PC1),
    Inst::JP(None, Am::Indirect_HL),
    Inst::LD(Am::PC2, Am::Direct_A),
    Inst::NONE,
    Inst::NONE,
    Inst::NONE,
    Inst::XOR(Am::PC1),
    Inst::RST,
    // 0xFx
    Inst::LD(Am::Direct_A, Am::PC1),
    Inst::POP(Am::Direct_AF),
    Inst::LD(Am::Direct_A, Am::Direct_C),
    Inst::DI,
    Inst::NONE,
    Inst::PUSH(Am::Direct_AF),
    Inst::OR(Am::PC1),
    Inst::RST,
    Inst::LD(Am::Direct_HL, Am::Direct_SP), /* SP + r8 */
    Inst::LD(Am::Direct_SP, Am::Direct_HL),
    Inst::LD(Am::Direct_A, Am::PC2),
    Inst::EI,
    Inst::NONE,
    Inst::NONE,
    Inst::CP(Am::PC1),
    Inst::RST,
];

type Cycle = (u8, u8);

const CYCLES: [Cycle; 256] = [
    // 0x0x
    (4, 4),
    (12, 12),
    (8, 8),
    (8, 8),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (20, 20),
    (8, 8),
    (8, 8),
    (8, 8),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x1x
    (4, 4),
    (12, 12),
    (8, 8),
    (8, 8),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (12, 12),
    (8, 8),
    (8, 8),
    (8, 8),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x2x
    (12, 8),
    (12, 12),
    (8, 8),
    (8, 8),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (12, 8),
    (8, 8),
    (8, 8),
    (8, 8),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x3x
    (12, 8),
    (12, 12),
    (8, 8),
    (8, 8),
    (12, 12),
    (12, 12),
    (12, 12),
    (4, 4),
    (12, 8),
    (8, 8),
    (8, 8),
    (8, 8),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x4x
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x5x
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x6x
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x7x
    (8, 8),
    (8, 8),
    (8, 8),
    (8, 8),
    (8, 8),
    (8, 8),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x8x
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0x9x
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0xAx
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0xBx
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 4),
    (8, 8),
    (4, 4),
    // 0xCx
    (20, 8),
    (12, 12),
    (16, 12),
    (16, 16),
    (24, 12),
    (16, 16),
    (8, 8),
    (16, 16),
    (20, 8),
    (16, 16),
    (16, 12),
    (4, 4),
    (24, 12),
    (24, 24),
    (8, 8),
    (16, 16),
    // 0xDx
    (20, 8),
    (12, 12),
    (16, 12),
    (0, 0),
    (24, 12),
    (16, 16),
    (8, 8),
    (16, 16),
    (20, 8),
    (16, 16),
    (16, 12),
    (0, 0),
    (24, 12),
    (0, 0),
    (8, 8),
    (16, 16),
    // 0xEx
    (12, 12),
    (12, 12),
    (8, 8),
    (0, 0),
    (0, 0),
    (16, 16),
    (8, 8),
    (16, 16),
    (16, 16),
    (4, 4),
    (16, 16),
    (0, 0),
    (0, 0),
    (0, 0),
    (8, 8),
    (16, 16),
    // 0xFx
    (12, 12),
    (12, 12),
    (8, 8),
    (4, 4),
    (0, 0),
    (16, 16),
    (8, 8),
    (16, 16),
    (12, 12),
    (8, 8),
    (16, 16),
    (4, 4),
    (0, 0),
    (0, 0),
    (8, 8),
    (16, 16),
];

pub(crate) fn get_instruction(opcode: u8) -> &'static Instruction {
    INSTRUCTIONS.get(opcode as usize).unwrap()
}

pub(crate) fn get_cycles(opcode: u8) -> &'static Cycle {
    CYCLES.get(opcode as usize).unwrap()
}
