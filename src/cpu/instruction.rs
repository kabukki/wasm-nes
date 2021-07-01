use crate::cpu::cpu::{Cpu, StatusFlag};
use crate::cpu::memory::Memory;

type InstructionHandler = fn (&mut Cpu, Operand, &mut Memory);

#[derive(Debug, Copy, Clone)]
pub enum AddressingMode {
    /**
     * ∅
     */
    Implied,
    /**
     * = A
     */
    Accumulator,
    /**
     * = CONSTANT[i8]
     */
    Immediate,
    /**
     * = M(PC + OFFSET[i8])
     */
    Relative,
    /**
     * = M(ADDR[u16])
     */
    Absolute,
    /**
     * = M(ADDR[u16]) + X
     */
    AbsoluteX,
    /**
     * = M(ADDR[u16]) + Y
     */
    AbsoluteY,
    /**
     * = P0(ADDR[u8])
     */
    ZeroPage,
    /**
     * = P0(ADDR[u8] + X)
     */
    ZeroPageX,
    /**
     * = P0(ADDR[u8] + Y)
     */
    ZeroPageY,
    /**
     * = M(PTR(ADDR[u16]))
     */
    Indirect,
    /**
     * = P0(PTR(ADDR[u8] + X))
     */
    IndirectX,
    /**
     * = P0(PTR(ADDR[u8]) + Y)
     */
    IndirectY,
}

#[derive(PartialEq)]
pub enum Operand {
    None,
    Byte (u8),
    Address (u16),
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::None => write!(f, "None"),
            Operand::Byte (value) => write!(f, "{} ({:#0x})", value, value),
            Operand::Address (address) => write!(f, "{:#018x}", address),
        }
    }
}

pub struct Instruction {
    pub opcode: u8,
    pub name: &'static str,
    pub mode: AddressingMode,
    pub cycles: u8,
    pub handler: InstructionHandler,
    pub illegal: bool,
}

impl Instruction {
    pub fn execute (&self, cpu: &mut Cpu, memory: &mut Memory) -> u8 {
        let operand = match self.mode {
            AddressingMode::Implied => Operand::None,
            AddressingMode::Accumulator => Operand::None,
            AddressingMode::Immediate => {
                let value = memory.read(cpu.pc);
                cpu.pc += 1;
                Operand::Byte(value)
            },
            AddressingMode::Relative => {
                let value = memory.read(cpu.pc) as i8;
                cpu.pc += 1;
                Operand::Address(cpu.pc.wrapping_add(value as u16))
            },
            AddressingMode::Absolute => {
                let value = (memory.read(cpu.pc + 1) as u16) << 8 | memory.read(cpu.pc) as u16;
                cpu.pc += 2;
                Operand::Address(value)
            },
            AddressingMode::AbsoluteX => {
                let value = (memory.read(cpu.pc + 1) as u16) << 8 | memory.read(cpu.pc) as u16;
                cpu.pc += 2;
                Operand::Address(value + cpu.x as u16)
            },
            AddressingMode::AbsoluteY => {
                let value = (memory.read(cpu.pc + 1) as u16) << 8 | memory.read(cpu.pc) as u16;
                cpu.pc += 2;
                Operand::Address(value + cpu.y as u16)
            },
            AddressingMode::ZeroPage => {
                let value = memory.read(cpu.pc) as u16;
                cpu.pc += 1;
                Operand::Address(value)
            },
            AddressingMode::ZeroPageX => {
                let value = memory.read(cpu.pc);
                cpu.pc += 1;
                Operand::Address(value.wrapping_add(cpu.x) as u16)
            },
            AddressingMode::ZeroPageY => {
                let value = memory.read(cpu.pc);
                cpu.pc += 1;
                Operand::Address(value.wrapping_add(cpu.y) as u16)
            },
            AddressingMode::Indirect => {
                let ptr = (memory.read(cpu.pc + 1) as u16) << 8 | memory.read(cpu.pc) as u16;
                let address = (memory.read(ptr) as u16) << 8 | memory.read(ptr + 1) as u16;

                cpu.pc += 2;
                Operand::Address(address)
            },
            // AddressingMode::IndirectX => {}
            // AddressingMode::IndirectY,
            _ => Operand::None,
        };
        let extra = 0; // TODO extra cycles based on policy: page boundary, branching... returned along operand

        // log (operand, self.mode)
        println!("  {:02X} [{} {:#?}] {}", self.opcode, self.name, self.mode, operand);
        (self.handler)(cpu, operand, memory);

        self.cycles + extra
    }
}

pub const INSTRUCTIONS: [Instruction; 256] = [
    Instruction { opcode: 0x00, name: "BRK", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0x01, name: "ORA", mode: AddressingMode::IndirectX,   cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x02, name: "KIL", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x03, name: "SLO", mode: AddressingMode::IndirectX,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x04, name: "NOP", mode: AddressingMode::ZeroPage,    cycles: 2, handler: nop,            illegal: true   },
    Instruction { opcode: 0x05, name: "ORA", mode: AddressingMode::ZeroPage,    cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x06, name: "ASL", mode: AddressingMode::ZeroPage,    cycles: 2, handler: asl,            illegal: false  },
    Instruction { opcode: 0x07, name: "SLO", mode: AddressingMode::ZeroPage,    cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x08, name: "PHP", mode: AddressingMode::Implied,     cycles: 2, handler: php,            illegal: false  },
    Instruction { opcode: 0x09, name: "ORA", mode: AddressingMode::Immediate,   cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x0A, name: "ASL", mode: AddressingMode::Accumulator, cycles: 2, handler: asl,            illegal: false  },
    Instruction { opcode: 0x0B, name: "ANC", mode: AddressingMode::Immediate,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x0C, name: "NOP", mode: AddressingMode::Absolute,    cycles: 2, handler: nop,            illegal: true   },
    Instruction { opcode: 0x0D, name: "ORA", mode: AddressingMode::Absolute,    cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x0E, name: "ASL", mode: AddressingMode::Absolute,    cycles: 2, handler: asl,            illegal: false  },
    Instruction { opcode: 0x0F, name: "SLO", mode: AddressingMode::Absolute,    cycles: 2, handler: unimplemented,  illegal: false  },

    Instruction { opcode: 0x10, name: "BPL", mode: AddressingMode::Relative,    cycles: 2, handler: bpl,            illegal: false  },
    Instruction { opcode: 0x11, name: "ORA", mode: AddressingMode::IndirectY,   cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x12, name: "KIL", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x13, name: "SLO", mode: AddressingMode::IndirectY,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x14, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: nop,            illegal: true   },
    Instruction { opcode: 0x15, name: "ORA", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x16, name: "ASL", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: asl,            illegal: false  },
    Instruction { opcode: 0x17, name: "SLO", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x18, name: "CLC", mode: AddressingMode::Implied,     cycles: 2, handler: clc,            illegal: false  },
    Instruction { opcode: 0x19, name: "ORA", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x1A, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true   },
    Instruction { opcode: 0x1B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x1C, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x1D, name: "ORA", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: ora,            illegal: false  },
    Instruction { opcode: 0x1E, name: "ASL", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: asl,            illegal: false  },
    Instruction { opcode: 0x1F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x20, name: "JSR", mode: AddressingMode::Absolute,    cycles: 2, handler: jsr,            illegal: false  },
    Instruction { opcode: 0x21, name: "AND", mode: AddressingMode::IndirectX,   cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x22, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x23, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x24, name: "BIT", mode: AddressingMode::ZeroPage,    cycles: 2, handler: bit,            illegal: false  },
    Instruction { opcode: 0x25, name: "AND", mode: AddressingMode::ZeroPage,    cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x26, name: "ROL", mode: AddressingMode::ZeroPage,    cycles: 2, handler: rol,            illegal: false  },
    Instruction { opcode: 0x27, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x28, name: "PLP", mode: AddressingMode::Implied,     cycles: 2, handler: plp,            illegal: false  },
    Instruction { opcode: 0x29, name: "AND", mode: AddressingMode::Immediate,   cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x2A, name: "ROL", mode: AddressingMode::Accumulator, cycles: 2, handler: rol,            illegal: false  },
    Instruction { opcode: 0x2B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x2C, name: "BIT", mode: AddressingMode::Absolute,    cycles: 2, handler: bit,            illegal: false  },
    Instruction { opcode: 0x2D, name: "AND", mode: AddressingMode::Absolute,    cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x2E, name: "ROL", mode: AddressingMode::Absolute,    cycles: 2, handler: rol,            illegal: false  },
    Instruction { opcode: 0x2F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x30, name: "BMI", mode: AddressingMode::Relative,    cycles: 2, handler: bmi,            illegal: false  },
    Instruction { opcode: 0x31, name: "AND", mode: AddressingMode::IndirectY,   cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x32, name: "KIL", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x33, name: "RLA", mode: AddressingMode::IndirectX,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x34, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: nop,            illegal: true   },
    Instruction { opcode: 0x35, name: "AND", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x36, name: "ROL", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: rol,            illegal: false  },
    Instruction { opcode: 0x37, name: "RLA", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x38, name: "SEC", mode: AddressingMode::Implied,     cycles: 2, handler: sec,            illegal: false  },
    Instruction { opcode: 0x39, name: "AND", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x3A, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true   },
    Instruction { opcode: 0x3B, name: "RLA", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x3C, name: "NOP", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: nop,            illegal: true   },
    Instruction { opcode: 0x3D, name: "AND", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: and,            illegal: false  },
    Instruction { opcode: 0x3E, name: "ROL", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: rol,            illegal: false  },
    Instruction { opcode: 0x3F, name: "RLA", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x40, name: "RTI", mode: AddressingMode::Implied,     cycles: 6, handler: rti,            illegal: false  },
    Instruction { opcode: 0x41, name: "EOR", mode: AddressingMode::IndirectX,   cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x42, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x43, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x44, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x45, name: "EOR", mode: AddressingMode::ZeroPage,    cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x46, name: "LSR", mode: AddressingMode::ZeroPage,    cycles: 2, handler: lsr,            illegal: false  },
    Instruction { opcode: 0x47, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x48, name: "PHA", mode: AddressingMode::Implied,     cycles: 2, handler: pha,            illegal: false  },
    Instruction { opcode: 0x49, name: "EOR", mode: AddressingMode::Immediate,   cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x4A, name: "LSR", mode: AddressingMode::Accumulator, cycles: 2, handler: lsr,            illegal: false  },
    Instruction { opcode: 0x4B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x4C, name: "JMP", mode: AddressingMode::Absolute,    cycles: 3, handler: jmp,            illegal: false  },
    Instruction { opcode: 0x4D, name: "EOR", mode: AddressingMode::Absolute,    cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x4E, name: "LSR", mode: AddressingMode::Absolute,    cycles: 2, handler: lsr,            illegal: false  },
    Instruction { opcode: 0x4F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x50, name: "BVC", mode: AddressingMode::Relative,    cycles: 2, handler: bvc,            illegal: false  },
    Instruction { opcode: 0x51, name: "EOR", mode: AddressingMode::IndirectY,   cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x52, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x53, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x54, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x55, name: "EOR", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x56, name: "LSR", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: lsr,            illegal: false  },
    Instruction { opcode: 0x57, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x58, name: "CLI", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0x59, name: "EOR", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x5A, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x5B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x5C, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x5D, name: "EOR", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: eor,            illegal: false  },
    Instruction { opcode: 0x5E, name: "LSR", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: lsr,            illegal: false  },
    Instruction { opcode: 0x5F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x60, name: "RTS", mode: AddressingMode::Implied,     cycles: 2, handler: rts,            illegal: false  },
    Instruction { opcode: 0x61, name: "ADC", mode: AddressingMode::IndirectX,   cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x62, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x63, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x64, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x65, name: "ADC", mode: AddressingMode::ZeroPage,    cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x66, name: "ROR", mode: AddressingMode::ZeroPage,    cycles: 2, handler: ror,            illegal: false  },
    Instruction { opcode: 0x67, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x68, name: "PLA", mode: AddressingMode::Implied,     cycles: 4, handler: pla,            illegal: false  },
    Instruction { opcode: 0x69, name: "ADC", mode: AddressingMode::Immediate,   cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x6A, name: "ROR", mode: AddressingMode::Accumulator, cycles: 2, handler: ror,            illegal: false  },
    Instruction { opcode: 0x6B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x6C, name: "JMP", mode: AddressingMode::Indirect,    cycles: 3, handler: jmp,            illegal: false  },
    Instruction { opcode: 0x6D, name: "ADC", mode: AddressingMode::Absolute,    cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x6E, name: "ROR", mode: AddressingMode::Absolute,    cycles: 2, handler: ror,            illegal: false  },
    Instruction { opcode: 0x6F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x70, name: "BVS", mode: AddressingMode::Relative,    cycles: 2, handler: bvs,            illegal: false  },
    Instruction { opcode: 0x71, name: "ADC", mode: AddressingMode::IndirectY,   cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x72, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x73, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x74, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x75, name: "ADC", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x76, name: "ROR", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: ror,            illegal: false  },
    Instruction { opcode: 0x77, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x78, name: "SEI", mode: AddressingMode::Implied,     cycles: 2, handler: sei,            illegal: false  },
    Instruction { opcode: 0x79, name: "ADC", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x7A, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x7B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x7C, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x7D, name: "ADC", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: adc,            illegal: false  },
    Instruction { opcode: 0x7E, name: "ROR", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: ror,            illegal: false  },
    Instruction { opcode: 0x7F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x80, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x81, name: "STA", mode: AddressingMode::IndirectX,   cycles: 2, handler: sta,            illegal: false  },
    Instruction { opcode: 0x82, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x83, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x84, name: "STY", mode: AddressingMode::ZeroPage,    cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0x85, name: "STA", mode: AddressingMode::ZeroPage,    cycles: 2, handler: sta,            illegal: false  },
    Instruction { opcode: 0x86, name: "STX", mode: AddressingMode::ZeroPage,    cycles: 2, handler: stx,            illegal: false  },
    Instruction { opcode: 0x87, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x88, name: "DEY", mode: AddressingMode::Implied,     cycles: 2, handler: dey,            illegal: false  },
    Instruction { opcode: 0x89, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x8A, name: "TXA", mode: AddressingMode::Implied,     cycles: 2, handler: txa,            illegal: false  },
    Instruction { opcode: 0x8B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x8C, name: "STY", mode: AddressingMode::Absolute,    cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0x8D, name: "STA", mode: AddressingMode::Absolute,    cycles: 2, handler: sta,            illegal: false  },
    Instruction { opcode: 0x8E, name: "STX", mode: AddressingMode::Absolute,    cycles: 2, handler: stx,            illegal: false  },
    Instruction { opcode: 0x8F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0x90, name: "BCC", mode: AddressingMode::Relative,    cycles: 2, handler: bcc,            illegal: false  },
    Instruction { opcode: 0x91, name: "STA", mode: AddressingMode::IndirectY,   cycles: 2, handler: sta,            illegal: false  },
    Instruction { opcode: 0x92, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x93, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x94, name: "STY", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0x95, name: "STA", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: sta,            illegal: false  },
    Instruction { opcode: 0x96, name: "STX", mode: AddressingMode::ZeroPageY,   cycles: 2, handler: stx,            illegal: false  },
    Instruction { opcode: 0x97, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x98, name: "TYA", mode: AddressingMode::Implied,     cycles: 2, handler: tya,            illegal: false  },
    Instruction { opcode: 0x99, name: "STA", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: sta,            illegal: false  },
    Instruction { opcode: 0x9A, name: "TXS", mode: AddressingMode::Implied,     cycles: 2, handler: txs,            illegal: false  },
    Instruction { opcode: 0x9B, name: "TAS", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x9C, name: "SHY", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x9D, name: "STA", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: sta,            illegal: false  },
    Instruction { opcode: 0x9E, name: "SHX", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0x9F, name: "SHA", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0xA0, name: "LDY", mode: AddressingMode::Immediate,   cycles: 2, handler: ldy,            illegal: false  },
    Instruction { opcode: 0xA1, name: "LDA", mode: AddressingMode::IndirectX,   cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xA2, name: "LDX", mode: AddressingMode::Immediate,   cycles: 2, handler: ldx,            illegal: false  },
    Instruction { opcode: 0xA3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xA4, name: "LDY", mode: AddressingMode::ZeroPage,    cycles: 2, handler: ldy,            illegal: false  },
    Instruction { opcode: 0xA5, name: "LDA", mode: AddressingMode::ZeroPage,    cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xA6, name: "LDX", mode: AddressingMode::ZeroPage,    cycles: 2, handler: ldx,            illegal: false  },
    Instruction { opcode: 0xA7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xA8, name: "TAY", mode: AddressingMode::Implied,     cycles: 2, handler: tay,            illegal: false  },
    Instruction { opcode: 0xA9, name: "LDA", mode: AddressingMode::Immediate,   cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xAA, name: "TAX", mode: AddressingMode::Implied,     cycles: 2, handler: tax,            illegal: false  },
    Instruction { opcode: 0xAB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xAC, name: "LDY", mode: AddressingMode::Absolute,    cycles: 2, handler: ldy,            illegal: false  },
    Instruction { opcode: 0xAD, name: "LDA", mode: AddressingMode::Absolute,    cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xAE, name: "LDX", mode: AddressingMode::Absolute,    cycles: 2, handler: ldx,            illegal: false  },
    Instruction { opcode: 0xAF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0xB0, name: "BCS", mode: AddressingMode::Relative,    cycles: 2, handler: bcs,            illegal: false  },
    Instruction { opcode: 0xB1, name: "LDA", mode: AddressingMode::IndirectY,   cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xB2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xB3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xB4, name: "LDY", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: ldy,            illegal: false  },
    Instruction { opcode: 0xB5, name: "LDA", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xB6, name: "LDX", mode: AddressingMode::ZeroPageY,   cycles: 2, handler: ldx,            illegal: false  },
    Instruction { opcode: 0xB7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xB8, name: "CLV", mode: AddressingMode::Implied,     cycles: 2, handler: clv,            illegal: false  },
    Instruction { opcode: 0xB9, name: "LDA", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xBA, name: "TSX", mode: AddressingMode::Implied,     cycles: 2, handler: tsx,            illegal: false  },
    Instruction { opcode: 0xBB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xBC, name: "LDY", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: ldy,            illegal: false  },
    Instruction { opcode: 0xBD, name: "LDA", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: lda,            illegal: false  },
    Instruction { opcode: 0xBE, name: "LDX", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: ldx,            illegal: false  },
    Instruction { opcode: 0xBF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0xC0, name: "CPY", mode: AddressingMode::Immediate,   cycles: 2, handler: cpy,            illegal: false  },
    Instruction { opcode: 0xC1, name: "CMP", mode: AddressingMode::IndirectX,   cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xC2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xC3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xC4, name: "CPY", mode: AddressingMode::ZeroPage,    cycles: 2, handler: cpy,            illegal: false  },
    Instruction { opcode: 0xC5, name: "CMP", mode: AddressingMode::ZeroPage,    cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xC6, name: "DEC", mode: AddressingMode::ZeroPage,    cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0xC7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xC8, name: "INY", mode: AddressingMode::Implied,     cycles: 2, handler: iny,            illegal: false  },
    Instruction { opcode: 0xC9, name: "CMP", mode: AddressingMode::Immediate,   cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xCA, name: "DEX", mode: AddressingMode::Implied,     cycles: 2, handler: dex,            illegal: false  },
    Instruction { opcode: 0xCB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xCC, name: "CPY", mode: AddressingMode::Absolute,    cycles: 2, handler: cpy,            illegal: false  },
    Instruction { opcode: 0xCD, name: "CMP", mode: AddressingMode::Absolute,    cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xCE, name: "DEC", mode: AddressingMode::Absolute,    cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0xCF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0xD0, name: "BNE", mode: AddressingMode::Relative,    cycles: 2, handler: bne,            illegal: false  },
    Instruction { opcode: 0xD1, name: "CMP", mode: AddressingMode::IndirectY,   cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xD2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xD3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xD4, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xD5, name: "CMP", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xD6, name: "DEC", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0xD7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xD8, name: "CLD", mode: AddressingMode::Implied,     cycles: 2, handler: cld,            illegal: false  },
    Instruction { opcode: 0xD9, name: "CMP", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xDA, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xDB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xDC, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xDD, name: "CMP", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: cmp,            illegal: false  },
    Instruction { opcode: 0xDE, name: "DEC", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: unimplemented,  illegal: false  },
    Instruction { opcode: 0xDF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0xE0, name: "CPX", mode: AddressingMode::Immediate,   cycles: 2, handler: cpx,            illegal: false  },
    Instruction { opcode: 0xE1, name: "SBC", mode: AddressingMode::IndirectX,   cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xE2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xE3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xE4, name: "CPX", mode: AddressingMode::ZeroPage,    cycles: 2, handler: cpx,            illegal: false  },
    Instruction { opcode: 0xE5, name: "SBC", mode: AddressingMode::ZeroPage,    cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xE6, name: "INC", mode: AddressingMode::ZeroPage,    cycles: 2, handler: inc,            illegal: false  },
    Instruction { opcode: 0xE7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xE8, name: "INX", mode: AddressingMode::Implied,     cycles: 2, handler: inx,            illegal: false  },
    Instruction { opcode: 0xE9, name: "SBC", mode: AddressingMode::Immediate,   cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xEA, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: false  },
    Instruction { opcode: 0xEB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xEC, name: "CPX", mode: AddressingMode::Absolute,    cycles: 2, handler: cpx,            illegal: false  },
    Instruction { opcode: 0xED, name: "SBC", mode: AddressingMode::Absolute,    cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xEE, name: "INC", mode: AddressingMode::Absolute,    cycles: 2, handler: inc,            illegal: false  },
    Instruction { opcode: 0xEF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },

    Instruction { opcode: 0xF0, name: "BEQ", mode: AddressingMode::Relative,    cycles: 2, handler: beq,            illegal: false  },
    Instruction { opcode: 0xF1, name: "SBC", mode: AddressingMode::IndirectY,   cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xF2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xF3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xF4, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xF5, name: "SBC", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xF6, name: "INC", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: inc,            illegal: false  },
    Instruction { opcode: 0xF7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xF8, name: "SED", mode: AddressingMode::Implied,     cycles: 2, handler: sed,            illegal: false  },
    Instruction { opcode: 0xF9, name: "SBC", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xFA, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xFB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xFC, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
    Instruction { opcode: 0xFD, name: "SBC", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: sbc,            illegal: false  },
    Instruction { opcode: 0xFE, name: "INC", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: inc,            illegal: false  },
    Instruction { opcode: 0xFF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true   },
];

fn nop (_cpu: &mut Cpu, _operand: Operand, _memory: &mut Memory) {}

fn unimplemented (_cpu: &mut Cpu, _operand: Operand, _memory: &mut Memory) {
    unimplemented!();
}

/**
 * Add Memory to Accumulator with Carry
 */
fn adc (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    let res = cpu.a as u16 + value as u16 + cpu.get_flag(StatusFlag::Carry) as u16;
    let overflow = (!(cpu.a ^ value) & (cpu.a ^ res as u8) & StatusFlag::Negative as u8) != 0;
    
    cpu.a = res as u8;
    
    cpu.set_flag(StatusFlag::Carry, res >> 8 != 0);
    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Overflow, overflow);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * AND Memory with Accumulator
 */
fn and (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a &= value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * Shift Left One Bit
 */
fn asl (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    fn lsr_inner (cpu: &mut Cpu, value: u8) -> u8 {
        let (new_value, carry) = (value << 1, value & StatusFlag::Negative as u8);
        cpu.set_flag(StatusFlag::Carry, carry != 0);
        cpu.set_flag(StatusFlag::Zero, new_value == 0);
        cpu.set_flag(StatusFlag::Negative, (new_value as i8) < 0);

        new_value
    }

    match operand {
        Operand::None => {
            let new_value = lsr_inner(cpu, cpu.a);
            cpu.a = new_value;
        },
        Operand::Address (address) => {
            let new_value = lsr_inner(cpu, memory.read(address));
            memory.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * Branch on Carry Set
 */
fn bcs (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Carry) {
        cpu.pc = address;
    }
}

/**
 * Branch on Carry Clear
 */
fn bcc (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Carry) {
        cpu.pc = address;
    }
}

/**
 * Branch on Result Zero
 */
fn beq (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Zero) {
        cpu.pc = address;
    }
}

/**
 * Test Bits in Memory with Accumulator
 */
fn bit (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Zero, (cpu.a & value) == 0);
    cpu.set_flag(StatusFlag::Overflow, (value & StatusFlag::Overflow as u8) != 0);
    cpu.set_flag(StatusFlag::Negative, (value & StatusFlag::Negative as u8) != 0);
}

/**
 * Branch on Result Minus
 */
fn bmi (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Negative) {
        cpu.pc = address;
    }
}

/**
 * Branch on Result not Zero
 */
fn bne (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Zero) {
        cpu.pc = address;
    }
}

/**
 * Branch on Result Plus
 */
fn bpl (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Negative) {
        cpu.pc = address;
    }
}

/**
 * Branch on Overflow Clear
 */
fn bvc (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Overflow) {
        cpu.pc = address;
    }
}

/**
 * Branch on Overflow Set
 */
fn bvs (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Overflow) {
        cpu.pc = address;
    }
}

/**
 * Clear Carry Flag
 */
fn clc (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Carry, false);
}

/**
 * Clear Overflow Flag
 */
fn clv (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Overflow, false);
}

/**
 * Clear Decimal Mode
 */
fn cld (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Decimal, false);
}

/**
 * Compare Memory with Accumulator
 */
fn cmp (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Carry, cpu.a >= value);
    cpu.set_flag(StatusFlag::Zero, cpu.a == value);
    cpu.set_flag(StatusFlag::Negative, (cpu.a.wrapping_sub(value) as i8) < 0);
}

/**
 * Compare Memory and Index X
 */
fn cpx (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Carry, cpu.x >= value);
    cpu.set_flag(StatusFlag::Zero, cpu.x == value);
    cpu.set_flag(StatusFlag::Negative, (cpu.x.wrapping_sub(value) as i8) < 0);
}

/**
 * Compare Memory and Index Y
 */
fn cpy (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Carry, cpu.y >= value);
    cpu.set_flag(StatusFlag::Zero, cpu.y == value);
    cpu.set_flag(StatusFlag::Negative, (cpu.y.wrapping_sub(value) as i8) < 0);
}

/**
 * Decrement Index X by One
 */
fn dex (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.x,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.x = value.wrapping_sub(1);

    cpu.set_flag(StatusFlag::Zero, cpu.x == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.x as i8) < 0);
}

/**
 * Decrement Index Y by One
 */
fn dey (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.y,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.y = value.wrapping_sub(1);

    cpu.set_flag(StatusFlag::Zero, cpu.y == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.y as i8) < 0);
}

/**
 * Exclusive-OR Memory with Accumulator
 */
fn eor (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a ^= value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * Increment Memory by One
 */
fn inc (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    let value = memory.read(address).wrapping_add(1);
    memory.write(address, value);

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Increment Index X by One
 */
fn inx (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.x,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.x = value.wrapping_add(1);

    cpu.set_flag(StatusFlag::Zero, cpu.x == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.x as i8) < 0);
}

/**
 * Increment Index Y by One
 */
fn iny (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.y,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.y = value.wrapping_add(1);

    cpu.set_flag(StatusFlag::Zero, cpu.y == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.y as i8) < 0);
}

/**
 * Jump to New Location
 */
fn jmp (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.pc = address;
}

/**
 * Jump to New Location Saving Return Address
 */
fn jsr (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    let (hi, lo) = (((cpu.pc - 1) >> 8) as u8, (cpu.pc - 1) as u8);
    cpu.push_stack(memory, hi);
    cpu.push_stack(memory, lo);
    cpu.pc = address;
}

/**
 * Loads a byte of memory into the X register setting the zero and negative flags as appropriate.
 */
fn ldx (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.x = value;

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Load Accumulator with Memory
 */
fn lda (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a = value;

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Load Index Y with Memory
 */
fn ldy (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.y = value;

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Shift One Bit Right
 */
fn lsr (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    fn lsr_inner (cpu: &mut Cpu, value: u8) -> u8 {
        let (new_value, carry) = (value >> 1, value & 1);
        cpu.set_flag(StatusFlag::Carry, carry != 0);
        cpu.set_flag(StatusFlag::Zero, new_value == 0);
        cpu.set_flag(StatusFlag::Negative, false);

        new_value
    }

    match operand {
        Operand::None => {
            let new_value = lsr_inner(cpu, cpu.a);
            cpu.a = new_value;
        },
        Operand::Address (address) => {
            let new_value = lsr_inner(cpu, memory.read(address));
            memory.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * OR Memory with Accumulator
 */
fn ora (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a |= value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * Push Accumulator on Stack
 */
fn pha (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.a,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.push_stack(memory, value);
}

/**
 * Push Processor Status on Stack
 */
fn php (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let status = match operand {
        Operand::None => cpu.status,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.push_stack(memory, status | (StatusFlag::Break as u8) | (StatusFlag::Unused as u8));
}

/**
 * Pull Accumulator from Stack
 */
fn pla (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.pop_stack(memory),
        _ => panic!("Invalid addressing mode"),
    };

    println!("{:08b} {} {}", value, value, value as i8);
    cpu.a = value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a & StatusFlag::Negative as u8) > 0);
}

/**
 * Pull Processor Status from Stack
 */
fn plp (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let status = match operand {
        Operand::None => cpu.pop_stack(memory),
        _ => panic!("Invalid addressing mode"),
    };

    // Ignored flags
    let mask = (StatusFlag::Break as u8) | (StatusFlag::Unused as u8);
    cpu.status = (status & !mask) | (cpu.status & mask);
}

/**
 * Rotate One Bit Left
 */
fn rol (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    fn ror_inner (cpu: &mut Cpu, value: u8) -> u8 {
        let (new_value, carry) = (value << 1 | cpu.status & StatusFlag::Carry as u8, value & StatusFlag::Negative as u8);
        cpu.set_flag(StatusFlag::Carry, carry != 0);
        cpu.set_flag(StatusFlag::Zero, new_value == 0);
        cpu.set_flag(StatusFlag::Negative, (new_value as i8) < 0);

        new_value
    }

    match operand {
        Operand::None => {
            let new_value = ror_inner(cpu, cpu.a);
            cpu.a = new_value;
        },
        Operand::Address (address) => {
            let new_value = ror_inner(cpu, memory.read(address));
            memory.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * Rotate One Bit Right
 */
fn ror (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    fn ror_inner (cpu: &mut Cpu, value: u8) -> u8 {
        let (new_value, carry) = (value >> 1 | (cpu.status & StatusFlag::Carry as u8) << 7, value & 1);
        cpu.set_flag(StatusFlag::Carry, carry != 0);
        cpu.set_flag(StatusFlag::Zero, new_value == 0);
        cpu.set_flag(StatusFlag::Negative, (new_value as i8) < 0);

        new_value
    }

    match operand {
        Operand::None => {
            let new_value = ror_inner(cpu, cpu.a);
            cpu.a = new_value;
        },
        Operand::Address (address) => {
            let new_value = ror_inner(cpu, memory.read(address));
            memory.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * Return from Interrupt
 */
fn rti (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    // Ignored flags
    let mask = (StatusFlag::Break as u8) | (StatusFlag::Unused as u8);
    let status = cpu.pop_stack(memory);
    let (lo, hi) = (cpu.pop_stack(memory), cpu.pop_stack(memory));
    let address = (hi as u16) << 8 | lo as u16;
    
    cpu.status = (status & !mask) | (cpu.status & mask);
    cpu.pc = address;
}

/**
 * Return from Subroutine
 */
fn rts (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    let (lo, hi) = (cpu.pop_stack(memory), cpu.pop_stack(memory));
    let address = (hi as u16) << 8 | lo as u16;

    cpu.pc = address + 1;
}

/**
 * Subtract Memory from Accumulator with Borrow
 */
fn sbc (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => memory.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    adc(cpu, Operand::Byte(!value), memory);
}

/**
 * Set Carry Flag
 */
fn sec (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Carry, true);
}

/**
 * Set Decimal Flag
 */
fn sed (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Decimal, true);
}

/**
 * Set Interrupt Disable Status
 */
fn sei (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Interrupt, true);
}

/**
 * Store Accumulator in Memory
 */
fn sta (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    memory.write(address, cpu.a);
}

/**
 * Store Index X in Memory
 */
fn stx (cpu: &mut Cpu, operand: Operand, memory: &mut Memory) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    memory.write(address, cpu.x);
}

/**
 * Transfer Accumulator to Index X
 */
fn tax (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.a,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.x = value;

    cpu.set_flag(StatusFlag::Zero, cpu.x == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.x as i8) < 0);
}

/**
 * Transfer Accumulator to Index Y
 */
fn tay (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.a,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.y = value;

    cpu.set_flag(StatusFlag::Zero, cpu.y == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.y as i8) < 0);
}

/**
 * Transfer Stack Pointer to Index X
 */
fn tsx (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.sp,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.x = value;

    cpu.set_flag(StatusFlag::Zero, cpu.x == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.x as i8) < 0);
}

/**
 * Transfer Index X to Accumulator
 */
fn txa (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.x,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a = value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * Transfer Index X to Stack Pointer
 */
fn txs (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.x,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.sp = value;
}

/**
 * Transfer Index Y to Accumulator
 */
fn tya (cpu: &mut Cpu, operand: Operand, _memory: &mut Memory) {
    let value = match operand {
        Operand::None => cpu.y,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a = value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

pub fn coverage () -> (usize, usize) {
    let legal = INSTRUCTIONS.iter().filter(|ins| !ins.illegal).count();
    let implemented = INSTRUCTIONS.iter().filter(|ins| !ins.illegal && ins.handler as usize != unimplemented as usize).count();

    (legal, implemented)
}
