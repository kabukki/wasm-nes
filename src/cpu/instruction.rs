use crate::bus::Bus;
use crate::cpu::{Cpu, StatusFlag, Interrupt};

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    ADC,
    ANC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLI,
    CLV,
    CLD,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    KIL,
    LAX,
    LDX,
    LDA,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    RLA,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    SHA,
    SHX,
    SHY,
    SLO,
    STA,
    STX,
    STY,
    TAS,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    ___,
}

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    Implied,        // ∅
    Accumulator,    // = A
    Immediate,      // = CONSTANT[i8]
    Relative,       // = M(PC + OFFSET[i8])
    Absolute,       // = M(ADDR[u16])
    AbsoluteX,      // = M(ADDR[u16]) + X
    AbsoluteY,      // = M(ADDR[u16]) + Y
    ZeroPage,       // = P0(ADDR[u8])
    ZeroPageX,      // = P0(ADDR[u8] + X)
    ZeroPageY,      // = P0(ADDR[u8] + Y)
    Indirect,       // = M(PTR(ADDR[u16]))
    IndirectX,      // = P0(PTR(ADDR[u8] + X))
    IndirectY,      // = P0(PTR(ADDR[u8]) + Y)
}

#[derive(Copy, Clone, PartialEq)]
pub enum Operand {
    None,
    Byte (u8),
    Address (u16),
}

#[derive(Copy, Clone)]
pub struct Instruction {
    pub opcode: u8,
    pub operator: Operator,
    pub mode: AddressingMode,
    pub cycles: usize,
    pub illegal: bool,
    pub extra_on_page_cross: bool,
}

const INSTRUCTIONS: [Instruction; 256] = [
    Instruction { opcode: 0x00, operator: Operator::BRK, mode: AddressingMode::Implied,     cycles: 7, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x01, operator: Operator::ORA, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x02, operator: Operator::KIL, mode: AddressingMode::Implied,     cycles: 1, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x03, operator: Operator::SLO, mode: AddressingMode::IndirectX,   cycles: 1, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x04, operator: Operator::NOP, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x05, operator: Operator::ORA, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x06, operator: Operator::ASL, mode: AddressingMode::ZeroPage,    cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x07, operator: Operator::SLO, mode: AddressingMode::ZeroPage,    cycles: 5, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x08, operator: Operator::PHP, mode: AddressingMode::Implied,     cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x09, operator: Operator::ORA, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0A, operator: Operator::ASL, mode: AddressingMode::Accumulator, cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0B, operator: Operator::ANC, mode: AddressingMode::Immediate,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x0C, operator: Operator::NOP, mode: AddressingMode::Absolute,    cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x0D, operator: Operator::ORA, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0E, operator: Operator::ASL, mode: AddressingMode::Absolute,    cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0F, operator: Operator::SLO, mode: AddressingMode::Absolute,    cycles: 6, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x10, operator: Operator::BPL, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x11, operator: Operator::ORA, mode: AddressingMode::IndirectY,   cycles: 5, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x12, operator: Operator::KIL, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x13, operator: Operator::SLO, mode: AddressingMode::IndirectY,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x14, operator: Operator::NOP, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x15, operator: Operator::ORA, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x16, operator: Operator::ASL, mode: AddressingMode::ZeroPageX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x17, operator: Operator::SLO, mode: AddressingMode::ZeroPageX,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x18, operator: Operator::CLC, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x19, operator: Operator::ORA, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x1A, operator: Operator::NOP, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x1B, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x1C, operator: Operator::NOP, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x1D, operator: Operator::ORA, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x1E, operator: Operator::ASL, mode: AddressingMode::AbsoluteX,   cycles: 7, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x1F, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x20, operator: Operator::JSR, mode: AddressingMode::Absolute,    cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x21, operator: Operator::AND, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x22, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x23, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x24, operator: Operator::BIT, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x25, operator: Operator::AND, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x26, operator: Operator::ROL, mode: AddressingMode::ZeroPage,    cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x27, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x28, operator: Operator::PLP, mode: AddressingMode::Implied,     cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x29, operator: Operator::AND, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2A, operator: Operator::ROL, mode: AddressingMode::Accumulator, cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2B, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x2C, operator: Operator::BIT, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2D, operator: Operator::AND, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2E, operator: Operator::ROL, mode: AddressingMode::Absolute,    cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2F, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x30, operator: Operator::BMI, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x31, operator: Operator::AND, mode: AddressingMode::IndirectY,   cycles: 5, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x32, operator: Operator::KIL, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x33, operator: Operator::RLA, mode: AddressingMode::IndirectX,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x34, operator: Operator::NOP, mode: AddressingMode::ZeroPageX,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x35, operator: Operator::AND, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x36, operator: Operator::ROL, mode: AddressingMode::ZeroPageX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x37, operator: Operator::RLA, mode: AddressingMode::ZeroPageX,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x38, operator: Operator::SEC, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x39, operator: Operator::AND, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x3A, operator: Operator::NOP, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x3B, operator: Operator::RLA, mode: AddressingMode::AbsoluteY,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x3C, operator: Operator::NOP, mode: AddressingMode::AbsoluteX,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x3D, operator: Operator::AND, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x3E, operator: Operator::ROL, mode: AddressingMode::AbsoluteX,   cycles: 7, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x3F, operator: Operator::RLA, mode: AddressingMode::AbsoluteX,   cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x40, operator: Operator::RTI, mode: AddressingMode::Implied,     cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x41, operator: Operator::EOR, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x42, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x43, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x44, operator: Operator::NOP, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x45, operator: Operator::EOR, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x46, operator: Operator::LSR, mode: AddressingMode::ZeroPage,    cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x47, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x48, operator: Operator::PHA, mode: AddressingMode::Implied,     cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x49, operator: Operator::EOR, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4A, operator: Operator::LSR, mode: AddressingMode::Accumulator, cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4B, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x4C, operator: Operator::JMP, mode: AddressingMode::Absolute,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4D, operator: Operator::EOR, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4E, operator: Operator::LSR, mode: AddressingMode::Absolute,    cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4F, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x50, operator: Operator::BVC, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x51, operator: Operator::EOR, mode: AddressingMode::IndirectY,   cycles: 5, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x52, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x53, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x54, operator: Operator::NOP, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x55, operator: Operator::EOR, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x56, operator: Operator::LSR, mode: AddressingMode::ZeroPageX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x57, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x58, operator: Operator::CLI, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x59, operator: Operator::EOR, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x5A, operator: Operator::NOP, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x5B, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x5C, operator: Operator::NOP, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x5D, operator: Operator::EOR, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x5E, operator: Operator::LSR, mode: AddressingMode::AbsoluteX,   cycles: 7, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x5F, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x60, operator: Operator::RTS, mode: AddressingMode::Implied,     cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x61, operator: Operator::ADC, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x62, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x63, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x64, operator: Operator::NOP, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x65, operator: Operator::ADC, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x66, operator: Operator::ROR, mode: AddressingMode::ZeroPage,    cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x67, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x68, operator: Operator::PLA, mode: AddressingMode::Implied,     cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x69, operator: Operator::ADC, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6A, operator: Operator::ROR, mode: AddressingMode::Accumulator, cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6B, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x6C, operator: Operator::JMP, mode: AddressingMode::Indirect,    cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6D, operator: Operator::ADC, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6E, operator: Operator::ROR, mode: AddressingMode::Absolute,    cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6F, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x70, operator: Operator::BVS, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x71, operator: Operator::ADC, mode: AddressingMode::IndirectY,   cycles: 5, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x72, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x73, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x74, operator: Operator::NOP, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x75, operator: Operator::ADC, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x76, operator: Operator::ROR, mode: AddressingMode::ZeroPageX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x77, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x78, operator: Operator::SEI, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x79, operator: Operator::ADC, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x7A, operator: Operator::NOP, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x7B, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x7C, operator: Operator::NOP, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x7D, operator: Operator::ADC, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x7E, operator: Operator::ROR, mode: AddressingMode::AbsoluteX,   cycles: 7, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x7F, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x80, operator: Operator::NOP, mode: AddressingMode::Immediate,   cycles: 3, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x81, operator: Operator::STA, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x82, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x83, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x84, operator: Operator::STY, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x85, operator: Operator::STA, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x86, operator: Operator::STX, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x87, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x88, operator: Operator::DEY, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x89, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x8A, operator: Operator::TXA, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8B, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x8C, operator: Operator::STY, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8D, operator: Operator::STA, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8E, operator: Operator::STX, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8F, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x90, operator: Operator::BCC, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x91, operator: Operator::STA, mode: AddressingMode::IndirectY,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x92, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x93, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x94, operator: Operator::STY, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x95, operator: Operator::STA, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x96, operator: Operator::STX, mode: AddressingMode::ZeroPageY,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x97, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x98, operator: Operator::TYA, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x99, operator: Operator::STA, mode: AddressingMode::AbsoluteY,   cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x9A, operator: Operator::TXS, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x9B, operator: Operator::TAS, mode: AddressingMode::AbsoluteY,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x9C, operator: Operator::SHY, mode: AddressingMode::AbsoluteX,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x9D, operator: Operator::STA, mode: AddressingMode::AbsoluteX,   cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x9E, operator: Operator::SHX, mode: AddressingMode::AbsoluteY,   cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x9F, operator: Operator::SHA, mode: AddressingMode::AbsoluteY,   cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xA0, operator: Operator::LDY, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA1, operator: Operator::LDA, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA2, operator: Operator::LDX, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA3, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xA4, operator: Operator::LDY, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA5, operator: Operator::LDA, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA6, operator: Operator::LDX, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA7, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xA8, operator: Operator::TAY, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA9, operator: Operator::LDA, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAA, operator: Operator::TAX, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAB, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xAC, operator: Operator::LDY, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAD, operator: Operator::LDA, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAE, operator: Operator::LDX, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAF, operator: Operator::LAX, mode: AddressingMode::Absolute,    cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xB0, operator: Operator::BCS, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB1, operator: Operator::LDA, mode: AddressingMode::IndirectY,   cycles: 5, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xB2, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xB3, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xB4, operator: Operator::LDY, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB5, operator: Operator::LDA, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB6, operator: Operator::LDX, mode: AddressingMode::ZeroPageY,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB7, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xB8, operator: Operator::CLV, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB9, operator: Operator::LDA, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBA, operator: Operator::TSX, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xBB, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xBC, operator: Operator::LDY, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBD, operator: Operator::LDA, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBE, operator: Operator::LDX, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBF, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xC0, operator: Operator::CPY, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC1, operator: Operator::CMP, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC2, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xC3, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xC4, operator: Operator::CPY, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC5, operator: Operator::CMP, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC6, operator: Operator::DEC, mode: AddressingMode::ZeroPage,    cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC7, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xC8, operator: Operator::INY, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC9, operator: Operator::CMP, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCA, operator: Operator::DEX, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCB, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xCC, operator: Operator::CPY, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCD, operator: Operator::CMP, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCE, operator: Operator::DEC, mode: AddressingMode::Absolute,    cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCF, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xD0, operator: Operator::BNE, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD1, operator: Operator::CMP, mode: AddressingMode::IndirectY,   cycles: 5, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xD2, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD3, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD4, operator: Operator::NOP, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD5, operator: Operator::CMP, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD6, operator: Operator::DEC, mode: AddressingMode::ZeroPageX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD7, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD8, operator: Operator::CLD, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD9, operator: Operator::CMP, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xDA, operator: Operator::NOP, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xDB, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xDC, operator: Operator::NOP, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xDD, operator: Operator::CMP, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xDE, operator: Operator::DEC, mode: AddressingMode::AbsoluteX,   cycles: 7, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xDF, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xE0, operator: Operator::CPX, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE1, operator: Operator::SBC, mode: AddressingMode::IndirectX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE2, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xE3, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xE4, operator: Operator::CPX, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE5, operator: Operator::SBC, mode: AddressingMode::ZeroPage,    cycles: 3, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE6, operator: Operator::INC, mode: AddressingMode::ZeroPage,    cycles: 5, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE7, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xE8, operator: Operator::INX, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE9, operator: Operator::SBC, mode: AddressingMode::Immediate,   cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEA, operator: Operator::NOP, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEB, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xEC, operator: Operator::CPX, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xED, operator: Operator::SBC, mode: AddressingMode::Absolute,    cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEE, operator: Operator::INC, mode: AddressingMode::Absolute,    cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEF, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xF0, operator: Operator::BEQ, mode: AddressingMode::Relative,    cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF1, operator: Operator::SBC, mode: AddressingMode::IndirectY,   cycles: 5, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xF2, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF3, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF4, operator: Operator::NOP, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF5, operator: Operator::SBC, mode: AddressingMode::ZeroPageX,   cycles: 4, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF6, operator: Operator::INC, mode: AddressingMode::ZeroPageX,   cycles: 6, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF7, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF8, operator: Operator::SED, mode: AddressingMode::Implied,     cycles: 2, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF9, operator: Operator::SBC, mode: AddressingMode::AbsoluteY,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xFA, operator: Operator::NOP, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xFB, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xFC, operator: Operator::NOP, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xFD, operator: Operator::SBC, mode: AddressingMode::AbsoluteX,   cycles: 4, illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xFE, operator: Operator::INC, mode: AddressingMode::AbsoluteX,   cycles: 7, illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xFF, operator: Operator::___, mode: AddressingMode::Implied,     cycles: 2, illegal: true,      extra_on_page_cross: false  },
];

impl Bus {
    pub fn fetch_instruction (&mut self, address: u16) -> (Instruction, Operand, u16) {
        let opcode = self.read(address);
        let instruction = INSTRUCTIONS[opcode as usize];
        let operand = match instruction.mode {
            AddressingMode::Implied     |
            AddressingMode::Accumulator => Operand::None,
            AddressingMode::Immediate   |
            AddressingMode::Relative    |
            AddressingMode::ZeroPage    |
            AddressingMode::ZeroPageX   |
            AddressingMode::ZeroPageY   |
            AddressingMode::IndirectX   |
            AddressingMode::IndirectY   => Operand::Byte(self.read(address.wrapping_add(1))),
            AddressingMode::Absolute    |
            AddressingMode::AbsoluteX   |
            AddressingMode::AbsoluteY   |
            AddressingMode::Indirect    => Operand::Address((self.read(address.wrapping_add(2)) as u16) << 8 | self.read(address.wrapping_add(1)) as u16),
        };
        let read = match operand {
            Operand::None               => 1,
            Operand::Byte (_)           => 2,
            Operand::Address (_)        => 3,
        };

        (instruction, operand, read)
    }
}

/* Helpers */

fn same_page (a: u16, b: u16) -> bool {
    (a & 0xFF00) == (b & 0xFF00)
}

impl Cpu {
    /**
     * Interpret the encoded operand, notably read effective addresses
     */
    fn decode (&mut self, bus: &mut Bus, instruction: Instruction, operand: Operand) -> Operand {
        let (operand_decoded, page_crossed) = match (instruction.mode, operand) {
            (AddressingMode::Implied,       Operand::None)              => (Operand::None, false),
            (AddressingMode::Accumulator,   Operand::None)              => (Operand::Byte(self.a), false),
            (AddressingMode::Immediate,     Operand::Byte (byte))       => (Operand::Byte(byte), false),
            (AddressingMode::Relative,      Operand::Byte (byte))       => (Operand::Address(self.pc.wrapping_add(byte as u16)), false),
            (AddressingMode::Absolute,      Operand::Address (address)) => (Operand::Address(address), false),
            (AddressingMode::AbsoluteX,     Operand::Address (address)) => (
                Operand::Address(address.wrapping_add(self.x as u16)),
                !same_page(address.wrapping_add(self.x as u16), address)
            ),
            (AddressingMode::AbsoluteY,     Operand::Address (address)) => (
                Operand::Address(address.wrapping_add(self.y as u16)),
                !same_page(address.wrapping_add(self.y as u16), address)
            ),
            (AddressingMode::ZeroPage,      Operand::Byte (address))    => (Operand::Address(address as u16), false),
            (AddressingMode::ZeroPageX,     Operand::Byte (address))    => (Operand::Address(address.wrapping_add(self.x) as u16), false),
            (AddressingMode::ZeroPageY,     Operand::Byte (address))    => (Operand::Address(address.wrapping_add(self.y) as u16), false),
            (AddressingMode::Indirect,      Operand::Address (ptr))     => {
                // Simulate fetch error @ page boundary
                let page = ptr & 0xFF00;
                let address = (bus.read(page | (ptr as u8).wrapping_add(1) as u16) as u16) << 8 | bus.read(ptr) as u16;

                (Operand::Address(address), false)
            },
            (AddressingMode::IndirectX,     Operand::Byte (ptr))        => {
                let address = (bus.read(ptr.wrapping_add(self.x + 1) as u16) as u16) << 8 | bus.read(ptr.wrapping_add(self.x) as u16) as u16;
                (Operand::Address(address), false)
            },
            (AddressingMode::IndirectY,     Operand::Byte (ptr))        => {
                let address = (bus.read(ptr.wrapping_add(1) as u16) as u16) << 8 | bus.read(ptr as u16) as u16;

                (
                    Operand::Address(address.wrapping_add(self.y as u16)),
                    !same_page(address.wrapping_add(self.y as u16), address)
                )
            },
            (_, _) => unreachable!(),
        };

        if instruction.illegal {
            log::warn!("Encountered illegal instruction: {:02X} ({:?} {:?})", instruction.opcode, instruction.operator, instruction.mode);
        }

        if instruction.extra_on_page_cross && page_crossed {
            self.cycles += 1;
        }

        operand_decoded
    }

    /**
     * Fetch, decode and execue=te next instruction
     */
    pub fn execute (&mut self, bus: &mut Bus) {
        let (instruction, operand, read) = bus.fetch_instruction(self.pc);
        self.pc += read;

        let operand = self.decode(bus, instruction, operand);
        match instruction.operator {
            Operator::ADC => { self.adc(operand, bus); },
            Operator::AND => { self.and(operand, bus); },
            Operator::ASL => { self.asl(operand, bus); },
            Operator::BCC => { self.bcc(operand, bus); },
            Operator::BCS => { self.bcs(operand, bus); },
            Operator::BEQ => { self.beq(operand, bus); },
            Operator::BIT => { self.bit(operand, bus); },
            Operator::BMI => { self.bmi(operand, bus); },
            Operator::BNE => { self.bne(operand, bus); },
            Operator::BPL => { self.bpl(operand, bus); },
            Operator::BRK => { self.brk(operand, bus); },
            Operator::BVC => { self.bvc(operand, bus); },
            Operator::BVS => { self.bvs(operand, bus); },
            Operator::CLC => { self.clc(operand, bus); },
            Operator::CLI => { self.cli(operand, bus); },
            Operator::CLV => { self.clv(operand, bus); },
            Operator::CLD => { self.cld(operand, bus); },
            Operator::CMP => { self.cmp(operand, bus); },
            Operator::CPX => { self.cpx(operand, bus); },
            Operator::CPY => { self.cpy(operand, bus); },
            Operator::DEC => { self.dec(operand, bus); },
            Operator::DEX => { self.dex(operand, bus); },
            Operator::DEY => { self.dey(operand, bus); },
            Operator::EOR => { self.eor(operand, bus); },
            Operator::INC => { self.inc(operand, bus); },
            Operator::INX => { self.inx(operand, bus); },
            Operator::INY => { self.iny(operand, bus); },
            Operator::JMP => { self.jmp(operand, bus); },
            Operator::JSR => { self.jsr(operand, bus); },
            Operator::LDX => { self.ldx(operand, bus); },
            Operator::LDA => { self.lda(operand, bus); },
            Operator::LDY => { self.ldy(operand, bus); },
            Operator::LSR => { self.lsr(operand, bus); },
            Operator::NOP => { self.nop(operand, bus); },
            Operator::ORA => { self.ora(operand, bus); },
            Operator::PHA => { self.pha(operand, bus); },
            Operator::PHP => { self.php(operand, bus); },
            Operator::PLA => { self.pla(operand, bus); },
            Operator::PLP => { self.plp(operand, bus); },
            Operator::ROL => { self.rol(operand, bus); },
            Operator::ROR => { self.ror(operand, bus); },
            Operator::RTI => { self.rti(operand, bus); },
            Operator::RTS => { self.rts(operand, bus); },
            Operator::SBC => { self.sbc(operand, bus); },
            Operator::SEC => { self.sec(operand, bus); },
            Operator::SED => { self.sed(operand, bus); },
            Operator::SEI => { self.sei(operand, bus); },
            Operator::STA => { self.sta(operand, bus); },
            Operator::STX => { self.stx(operand, bus); },
            Operator::STY => { self.sty(operand, bus); },
            Operator::TAX => { self.tax(operand, bus); },
            Operator::TAY => { self.tay(operand, bus); },
            Operator::TSX => { self.tsx(operand, bus); },
            Operator::TXA => { self.txa(operand, bus); },
            Operator::TXS => { self.txs(operand, bus); },
            Operator::TYA => { self.tya(operand, bus); },
            _ => unimplemented!(),
        };

        self.cycles += instruction.cycles;
    }

    fn branch (&mut self, address: u16) {
        self.cycles += if !same_page(address, self.pc) { 2 } else { 1 };
        self.pc = address;
    }

    fn nop (&mut self, _operand: Operand, _bus: &mut Bus) {}

    /**
     * Add Memory to Accumulator with Carry
     */
    fn adc (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        let res = self.a as u16 + value as u16 + self.get_flag(StatusFlag::Carry) as u16;
        let overflow = (!(self.a ^ value) & (self.a ^ res as u8) & StatusFlag::Negative as u8) != 0;
        
        self.a = res as u8;
        
        self.set_flag(StatusFlag::Carry, res >> 8 != 0);
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Overflow, overflow);
        self.set_flag(StatusFlag::Negative, (self.a as i8) < 0);
    }
    
    /**
     * AND Memory with Accumulator
     */
    fn and (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.a &= value;
    
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a as i8) < 0);
    }
    
    /**
     * Shift Left One Bit
     */
    fn asl (&mut self, operand: Operand, bus: &mut Bus) {
        fn asl_inner (cpu: &mut Cpu, value: u8) -> u8 {
            let (new_value, carry) = (value << 1, value & StatusFlag::Negative as u8);
            cpu.set_flag(StatusFlag::Carry, carry != 0);
            cpu.set_flag(StatusFlag::Zero, new_value == 0);
            cpu.set_flag(StatusFlag::Negative, (new_value as i8) < 0);
    
            new_value
        }
    
        match operand {
            Operand::Byte (byte) => {
                let new_value = asl_inner(self, byte);
                self.a = new_value;
            },
            Operand::Address (address) => {
                let new_value = asl_inner(self, bus.read(address));
                bus.write(address, new_value);
            },
            _ => panic!("Invalid addressing mode"),
        };
    }
    
    /**
     * Branch on Carry Clear
     */
    fn bcc (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if !self.get_flag(StatusFlag::Carry) {
            self.branch(address);
        }
    }
    
    /**
     * Branch on Carry Set
     */
    fn bcs (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if self.get_flag(StatusFlag::Carry) {
            self.branch(address);
        }
    }
    
    /**
     * Branch on Result Zero
     */
    fn beq (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if self.get_flag(StatusFlag::Zero) {
            self.branch(address);
        }
    }
    
    /**
     * Test Bits in Memory with Accumulator
     */
    fn bit (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.set_flag(StatusFlag::Zero, (self.a & value) == 0);
        self.set_flag(StatusFlag::Overflow, (value & StatusFlag::Overflow as u8) != 0);
        self.set_flag(StatusFlag::Negative, (value & StatusFlag::Negative as u8) != 0);
    }
    
    /**
     * Branch on Result Minus
     */
    fn bmi (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if self.get_flag(StatusFlag::Negative) {
            self.branch(address);
        }
    }
    
    /**
     * Branch on Result not Zero
     */
    fn bne (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if !self.get_flag(StatusFlag::Zero) {
            self.branch(address);
        }
    }
    
    /**
     * Branch on Result Plus
     */
    fn bpl (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if !self.get_flag(StatusFlag::Negative) {
            self.branch(address);
        }
    }
    
    /**
     * Force Break
     */
    fn brk (&mut self, operand: Operand, bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        }
    
        let (hi, lo) = (((self.pc + 2) >> 8) as u8, (self.pc + 2) as u8);
        self.push_stack(bus, hi);
        self.push_stack(bus, lo);
    
        self.push_stack(bus, self.status | (StatusFlag::Break as u8) | (StatusFlag::Unused as u8));
        self.set_flag(StatusFlag::DisableInterrupt, true);
    
        let address = (bus.read(Interrupt::IRQ as u16 + 1) as u16) << 8 | bus.read(Interrupt::IRQ as u16) as u16;
        self.pc = address;
    }
    
    /**
     * Branch on Overflow Clear
     */
    fn bvc (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if !self.get_flag(StatusFlag::Overflow) {
            self.branch(address);
        }
    }
    
    /**
     * Branch on Overflow Set
     */
    fn bvs (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        if self.get_flag(StatusFlag::Overflow) {
            self.branch(address);
        }
    }
    
    /**
     * Clear Carry Flag
     */
    fn clc (&mut self, operand: Operand, _bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        self.set_flag(StatusFlag::Carry, false);
    }
    
    /**
     * Clear Interrupt Disable Bit
     */
    fn cli (&mut self, operand: Operand, _bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        self.set_flag(StatusFlag::DisableInterrupt, false);
    }
    
    /**
     * Clear Overflow Flag
     */
    fn clv (&mut self, operand: Operand, _bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        self.set_flag(StatusFlag::Overflow, false);
    }
    
    /**
     * Clear Decimal Mode
     */
    fn cld (&mut self, operand: Operand, _bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        self.set_flag(StatusFlag::Decimal, false);
    }
    
    /**
     * Compare Memory with Accumulator
     */
    fn cmp (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.set_flag(StatusFlag::Carry, self.a >= value);
        self.set_flag(StatusFlag::Zero, self.a == value);
        self.set_flag(StatusFlag::Negative, (self.a.wrapping_sub(value) as i8) < 0);
    }
    
    /**
     * Compare Memory and Index X
     */
    fn cpx (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.set_flag(StatusFlag::Carry, self.x >= value);
        self.set_flag(StatusFlag::Zero, self.x == value);
        self.set_flag(StatusFlag::Negative, (self.x.wrapping_sub(value) as i8) < 0);
    }
    
    /**
     * Compare Memory and Index Y
     */
    fn cpy (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.set_flag(StatusFlag::Carry, self.y >= value);
        self.set_flag(StatusFlag::Zero, self.y == value);
        self.set_flag(StatusFlag::Negative, (self.y.wrapping_sub(value) as i8) < 0);
    }
    
    /**
     * Decrement Memory by One
     */
    fn dec (&mut self, operand: Operand, bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        let value = bus.read(address).wrapping_sub(1);
        bus.write(address, value);
    
        self.set_flag(StatusFlag::Zero, value == 0);
        self.set_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    
    /**
     * Decrement Index X by One
     */
    fn dex (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.x,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.x = value.wrapping_sub(1);
    
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x as i8) < 0);
    }
    
    /**
     * Decrement Index Y by One
     */
    fn dey (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.y,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.y = value.wrapping_sub(1);
    
        self.set_flag(StatusFlag::Zero, self.y == 0);
        self.set_flag(StatusFlag::Negative, (self.y as i8) < 0);
    }
    
    /**
     * Exclusive-OR Memory with Accumulator
     */
    fn eor (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.a ^= value;
    
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a as i8) < 0);
    }
    
    /**
     * Increment Memory by One
     */
    fn inc (&mut self, operand: Operand, bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        let value = bus.read(address).wrapping_add(1);
        bus.write(address, value);
    
        self.set_flag(StatusFlag::Zero, value == 0);
        self.set_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    
    /**
     * Increment Index X by One
     */
    fn inx (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.x,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.x = value.wrapping_add(1);
    
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x as i8) < 0);
    }
    
    /**
     * Increment Index Y by One
     */
    fn iny (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.y,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.y = value.wrapping_add(1);
    
        self.set_flag(StatusFlag::Zero, self.y == 0);
        self.set_flag(StatusFlag::Negative, (self.y as i8) < 0);
    }
    
    /**
     * Jump to New Location
     */
    fn jmp (&mut self, operand: Operand, _bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.pc = address;
    }
    
    /**
     * Jump to New Location Saving Return Address
     */
    fn jsr (&mut self, operand: Operand, bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        let (hi, lo) = (((self.pc - 1) >> 8) as u8, (self.pc - 1) as u8);
        self.push_stack(bus, hi);
        self.push_stack(bus, lo);
        self.pc = address;
    }
    
    /**
     * Loads a byte of memory into the X register setting the zero and negative flags as appropriate.
     */
    fn ldx (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.x = value;
    
        self.set_flag(StatusFlag::Zero, value == 0);
        self.set_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    
    /**
     * Load Accumulator with Memory
     */
    fn lda (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.a = value;
    
        self.set_flag(StatusFlag::Zero, value == 0);
        self.set_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    
    /**
     * Load Index Y with Memory
     */
    fn ldy (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.y = value;
    
        self.set_flag(StatusFlag::Zero, value == 0);
        self.set_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    
    /**
     * Shift One Bit Right
     */
    fn lsr (&mut self, operand: Operand, bus: &mut Bus) {
        fn lsr_inner (cpu: &mut Cpu, value: u8) -> u8 {
            let (new_value, carry) = (value >> 1, value & 1);
            cpu.set_flag(StatusFlag::Carry, carry != 0);
            cpu.set_flag(StatusFlag::Zero, new_value == 0);
            cpu.set_flag(StatusFlag::Negative, false);
    
            new_value
        }
    
        match operand {
            Operand::Byte (byte) => {
                let new_value = lsr_inner(self, byte);
                self.a = new_value;
            },
            Operand::Address (address) => {
                let new_value = lsr_inner(self, bus.read(address));
                bus.write(address, new_value);
            },
            _ => panic!("Invalid addressing mode"),
        };
    }
    
    /**
     * OR Memory with Accumulator
     */
    fn ora (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.a |= value;
    
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a as i8) < 0);
    }
    
    /**
     * Push Accumulator on Stack
     */
    fn pha (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.a,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.push_stack(bus, value);
    }
    
    /**
     * Push Processor Status on Stack
     */
    fn php (&mut self, operand: Operand, bus: &mut Bus) {
        let status = match operand {
            Operand::None => self.status,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.push_stack(bus, status | (StatusFlag::Break as u8) | (StatusFlag::Unused as u8));
    }
    
    /**
     * Pull Accumulator from Stack
     */
    fn pla (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.pop_stack(bus),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.a = value;
    
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a & StatusFlag::Negative as u8) > 0);
    }
    
    /**
     * Pull Processor Status from Stack
     */
    fn plp (&mut self, operand: Operand, bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        // Ignored flags, stay as-is
        let mask = (StatusFlag::Break as u8) | (StatusFlag::Unused as u8);
        let status = self.pop_stack(bus);
        self.status = (status & !mask) | (self.status & mask);
    }

    /**
     * Rotate One Bit Left
     */
    fn rol (&mut self, operand: Operand, bus: &mut Bus) {
        fn ror_inner (cpu: &mut Cpu, value: u8) -> u8 {
            let (new_value, carry) = (value << 1 | cpu.status & StatusFlag::Carry as u8, value & StatusFlag::Negative as u8);
            cpu.set_flag(StatusFlag::Carry, carry != 0);
            cpu.set_flag(StatusFlag::Zero, new_value == 0);
            cpu.set_flag(StatusFlag::Negative, (new_value as i8) < 0);
    
            new_value
        }

        match operand {
            Operand::Byte (byte) => {
                let new_value = ror_inner(self, byte);
                self.a = new_value;
            },
            Operand::Address (address) => {
                let new_value = ror_inner(self, bus.read(address));
                bus.write(address, new_value);
            },
            _ => panic!("Invalid addressing mode"),
        };
    }

    /**
     * Rotate One Bit Right
     */
    fn ror (&mut self, operand: Operand, bus: &mut Bus) {
        fn ror_inner (cpu: &mut Cpu, value: u8) -> u8 {
            let (new_value, carry) = (value >> 1 | (cpu.status & StatusFlag::Carry as u8) << 7, value & 1);
            cpu.set_flag(StatusFlag::Carry, carry != 0);
            cpu.set_flag(StatusFlag::Zero, new_value == 0);
            cpu.set_flag(StatusFlag::Negative, (new_value as i8) < 0);
    
            new_value
        }
    
        match operand {
            Operand::Byte (byte) => {
                let new_value = ror_inner(self, byte);
                self.a = new_value;
            },
            Operand::Address (address) => {
                let new_value = ror_inner(self, bus.read(address));
                bus.write(address, new_value);
            },
            _ => panic!("Invalid addressing mode"),
        };
    }

    /**
     * Return from Interrupt
     */
    fn rti (&mut self, operand: Operand, bus: &mut Bus) {
        self.plp(operand, bus);
        let (lo, hi) = (self.pop_stack(bus), self.pop_stack(bus));
        let address = (hi as u16) << 8 | lo as u16;
        
        self.pc = address;
    }

    /**
     * Return from Subroutine
     */
    fn rts (&mut self, operand: Operand, bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        let (lo, hi) = (self.pop_stack(bus), self.pop_stack(bus));
        let address = (hi as u16) << 8 | lo as u16;
    
        self.pc = address + 1;
    }
    
    /**
     * Subtract Memory from Accumulator with Borrow
     */
    fn sbc (&mut self, operand: Operand, bus: &mut Bus) {
        let value = match operand {
            Operand::Byte (byte) => byte,
            Operand::Address (address) => bus.read(address),
            _ => panic!("Invalid addressing mode"),
        };
    
        self.adc(Operand::Byte(!value), bus);
    }
    
    /**
     * Set Carry Flag
     */
    fn sec (&mut self, operand: Operand, _bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        self.set_flag(StatusFlag::Carry, true);
    }
    
    /**
     * Set Decimal Flag
     */
    fn sed (&mut self, operand: Operand, _bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        self.set_flag(StatusFlag::Decimal, true);
    }
    
    /**
     * Set Interrupt Disable Status
     */
    fn sei (&mut self, operand: Operand, _bus: &mut Bus) {
        if operand != Operand::None {
            panic!("Invalid addressing mode");
        };
    
        self.set_flag(StatusFlag::DisableInterrupt, true);
    }
    
    /**
     * Store Accumulator in Memory
     */
    fn sta (&mut self, operand: Operand, bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        bus.write(address, self.a);
    }
    
    /**
     * Store Index X in Memory
     */
    fn stx (&mut self, operand: Operand, bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        bus.write(address, self.x);
    }
    
    /**
     * Store Index Y in Memory
     */
    fn sty (&mut self, operand: Operand, bus: &mut Bus) {
        let address = match operand {
            Operand::Address (address) => address,
            _ => panic!("Invalid addressing mode"),
        };
    
        bus.write(address, self.y);
    }
    
    /**
     * Transfer Accumulator to Index X
     */
    fn tax (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.a,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.x = value;
    
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x as i8) < 0);
    }
    
    /**
     * Transfer Accumulator to Index Y
     */
    fn tay (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.a,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.y = value;
    
        self.set_flag(StatusFlag::Zero, self.y == 0);
        self.set_flag(StatusFlag::Negative, (self.y as i8) < 0);
    }
    
    /**
     * Transfer Stack Pointer to Index X
     */
    fn tsx (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.sp,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.x = value;
    
        self.set_flag(StatusFlag::Zero, self.x == 0);
        self.set_flag(StatusFlag::Negative, (self.x as i8) < 0);
    }
    
    /**
     * Transfer Index X to Accumulator
     */
    fn txa (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.x,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.a = value;
    
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a as i8) < 0);
    }
    
    /**
     * Transfer Index X to Stack Pointer
     */
    fn txs (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.x,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.sp = value;
    }
    
    /**
     * Transfer Index Y to Accumulator
     */
    fn tya (&mut self, operand: Operand, _bus: &mut Bus) {
        let value = match operand {
            Operand::None => self.y,
            _ => panic!("Invalid addressing mode"),
        };
    
        self.a = value;
    
        self.set_flag(StatusFlag::Zero, self.a == 0);
        self.set_flag(StatusFlag::Negative, (self.a as i8) < 0);
    }
}
