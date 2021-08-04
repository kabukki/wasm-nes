use log::{warn, trace};
use crate::bus::Bus;
use crate::cpu::{Cpu, StatusFlag, Interrupt};

type InstructionHandler = fn (&mut Cpu, Operand, &mut Bus);

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
            Operand::Address (address) => write!(f, "{:#0x}", address),
        }
    }
}

pub struct Instruction {
    opcode: u8,
    name: &'static str,
    mode: AddressingMode,
    cycles: u8,
    handler: InstructionHandler,
    illegal: bool,
    extra_on_page_cross: bool,
}

impl Instruction {
    pub fn execute (&self, cpu: &mut Cpu, bus: &mut Bus) -> u8 {
        let (operand, page_crossed) = match self.mode {
            AddressingMode::Implied => (Operand::None, false),
            AddressingMode::Accumulator => (Operand::None, false),
            AddressingMode::Immediate => {
                let value = bus.read(cpu.pc);
                cpu.pc += 1;
                (Operand::Byte(value), false)
            },
            AddressingMode::Relative => {
                let address = bus.read(cpu.pc) as i8;
                cpu.pc += 1;
                (Operand::Address(cpu.pc.wrapping_add(address as u16)), false)
            },
            AddressingMode::Absolute => {
                let address = (bus.read(cpu.pc.wrapping_add(1)) as u16) << 8 | bus.read(cpu.pc) as u16;
                cpu.pc += 2;
                (Operand::Address(address), false)
            },
            AddressingMode::AbsoluteX => {
                let address = (bus.read(cpu.pc.wrapping_add(1)) as u16) << 8 | bus.read(cpu.pc) as u16;
                cpu.pc += 2;
                (
                    Operand::Address(address.wrapping_add(cpu.x as u16)),
                    !same_page(address.wrapping_add(cpu.x as u16), address)
                )
            },
            AddressingMode::AbsoluteY => {
                let address = (bus.read(cpu.pc.wrapping_add(1)) as u16) << 8 | bus.read(cpu.pc) as u16;
                cpu.pc += 2;
                (
                    Operand::Address(address.wrapping_add(cpu.y as u16)),
                    !same_page(address.wrapping_add(cpu.y as u16), address)
                )
            },
            AddressingMode::ZeroPage => {
                let address = bus.read(cpu.pc) as u16;
                cpu.pc += 1;
                (Operand::Address(address), false)
            },
            AddressingMode::ZeroPageX => {
                let address = bus.read(cpu.pc).wrapping_add(cpu.x) as u16;
                cpu.pc += 1;
                (Operand::Address(address), false)
            },
            AddressingMode::ZeroPageY => {
                let address = bus.read(cpu.pc).wrapping_add(cpu.y) as u16;
                cpu.pc += 1;
                (Operand::Address(address), false)
            },
            AddressingMode::Indirect => {
                let ptr = (bus.read(cpu.pc.wrapping_add(1)) as u16) << 8 | bus.read(cpu.pc) as u16;

                // Simulate fetch error @ page boundary
                let page = ptr & 0xFF00;
                let address = (bus.read(page | (ptr as u8).wrapping_add(1) as u16) as u16) << 8 | bus.read(ptr) as u16;

                cpu.pc += 2;
                (Operand::Address(address), false)
            },
            AddressingMode::IndirectX => {
                let ptr = bus.read(cpu.pc).wrapping_add(cpu.x);
                let address = (bus.read(ptr.wrapping_add(1) as u16) as u16) << 8 | bus.read(ptr as u16) as u16;

                cpu.pc += 1;
                (Operand::Address(address), false)
            },
            AddressingMode::IndirectY => {
                let ptr = bus.read(cpu.pc);
                let address = (bus.read(ptr.wrapping_add(1) as u16) as u16) << 8 | bus.read(ptr as u16) as u16;

                cpu.pc += 1;
                (
                    Operand::Address(address.wrapping_add(cpu.y as u16)),
                    !same_page(address.wrapping_add(cpu.y as u16), address)
                )
            },
        };

        if self.illegal {
            warn!("Encountered illegal operand: {:02x}", self.opcode);
        }
        
        // trace!("{:02X} [{} {:#?}] {}", self.opcode, self.name, self.mode, operand);
        
        if self.extra_on_page_cross && page_crossed {
            cpu.cycles += 1;
        }

        (self.handler)(cpu, operand, bus);

        self.cycles
    }
}

pub const INSTRUCTIONS: [Instruction; 256] = [
    Instruction { opcode: 0x00, name: "BRK", mode: AddressingMode::Implied,     cycles: 7, handler: brk,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x01, name: "ORA", mode: AddressingMode::IndirectX,   cycles: 6, handler: ora,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x02, name: "KIL", mode: AddressingMode::Implied,     cycles: 1, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x03, name: "SLO", mode: AddressingMode::IndirectX,   cycles: 1, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x04, name: "NOP", mode: AddressingMode::ZeroPage,    cycles: 3, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x05, name: "ORA", mode: AddressingMode::ZeroPage,    cycles: 3, handler: ora,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x06, name: "ASL", mode: AddressingMode::ZeroPage,    cycles: 5, handler: asl,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x07, name: "SLO", mode: AddressingMode::ZeroPage,    cycles: 5, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x08, name: "PHP", mode: AddressingMode::Implied,     cycles: 3, handler: php,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x09, name: "ORA", mode: AddressingMode::Immediate,   cycles: 2, handler: ora,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0A, name: "ASL", mode: AddressingMode::Accumulator, cycles: 2, handler: asl,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0B, name: "ANC", mode: AddressingMode::Immediate,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x0C, name: "NOP", mode: AddressingMode::Absolute,    cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x0D, name: "ORA", mode: AddressingMode::Absolute,    cycles: 4, handler: ora,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0E, name: "ASL", mode: AddressingMode::Absolute,    cycles: 6, handler: asl,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x0F, name: "SLO", mode: AddressingMode::Absolute,    cycles: 6, handler: unimplemented,  illegal: false,     extra_on_page_cross: false  },

    Instruction { opcode: 0x10, name: "BPL", mode: AddressingMode::Relative,    cycles: 2, handler: bpl,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x11, name: "ORA", mode: AddressingMode::IndirectY,   cycles: 5, handler: ora,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x12, name: "KIL", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x13, name: "SLO", mode: AddressingMode::IndirectY,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x14, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x15, name: "ORA", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: ora,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x16, name: "ASL", mode: AddressingMode::ZeroPageX,   cycles: 6, handler: asl,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x17, name: "SLO", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x18, name: "CLC", mode: AddressingMode::Implied,     cycles: 2, handler: clc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x19, name: "ORA", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: ora,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x1A, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x1B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x1C, name: "NOP", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x1D, name: "ORA", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: ora,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x1E, name: "ASL", mode: AddressingMode::AbsoluteX,   cycles: 7, handler: asl,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x1F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x20, name: "JSR", mode: AddressingMode::Absolute,    cycles: 6, handler: jsr,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x21, name: "AND", mode: AddressingMode::IndirectX,   cycles: 6, handler: and,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x22, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x23, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x24, name: "BIT", mode: AddressingMode::ZeroPage,    cycles: 3, handler: bit,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x25, name: "AND", mode: AddressingMode::ZeroPage,    cycles: 3, handler: and,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x26, name: "ROL", mode: AddressingMode::ZeroPage,    cycles: 5, handler: rol,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x27, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x28, name: "PLP", mode: AddressingMode::Implied,     cycles: 4, handler: plp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x29, name: "AND", mode: AddressingMode::Immediate,   cycles: 2, handler: and,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2A, name: "ROL", mode: AddressingMode::Accumulator, cycles: 2, handler: rol,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x2C, name: "BIT", mode: AddressingMode::Absolute,    cycles: 4, handler: bit,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2D, name: "AND", mode: AddressingMode::Absolute,    cycles: 4, handler: and,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2E, name: "ROL", mode: AddressingMode::Absolute,    cycles: 6, handler: rol,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x2F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x30, name: "BMI", mode: AddressingMode::Relative,    cycles: 2, handler: bmi,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x31, name: "AND", mode: AddressingMode::IndirectY,   cycles: 5, handler: and,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x32, name: "KIL", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x33, name: "RLA", mode: AddressingMode::IndirectX,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x34, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x35, name: "AND", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: and,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x36, name: "ROL", mode: AddressingMode::ZeroPageX,   cycles: 6, handler: rol,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x37, name: "RLA", mode: AddressingMode::ZeroPageX,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x38, name: "SEC", mode: AddressingMode::Implied,     cycles: 2, handler: sec,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x39, name: "AND", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: and,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x3A, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x3B, name: "RLA", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x3C, name: "NOP", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x3D, name: "AND", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: and,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x3E, name: "ROL", mode: AddressingMode::AbsoluteX,   cycles: 7, handler: rol,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x3F, name: "RLA", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x40, name: "RTI", mode: AddressingMode::Implied,     cycles: 6, handler: rti,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x41, name: "EOR", mode: AddressingMode::IndirectX,   cycles: 6, handler: eor,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x42, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x43, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x44, name: "NOP", mode: AddressingMode::ZeroPage,    cycles: 3, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x45, name: "EOR", mode: AddressingMode::ZeroPage,    cycles: 3, handler: eor,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x46, name: "LSR", mode: AddressingMode::ZeroPage,    cycles: 5, handler: lsr,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x47, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x48, name: "PHA", mode: AddressingMode::Implied,     cycles: 3, handler: pha,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x49, name: "EOR", mode: AddressingMode::Immediate,   cycles: 2, handler: eor,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4A, name: "LSR", mode: AddressingMode::Accumulator, cycles: 2, handler: lsr,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x4C, name: "JMP", mode: AddressingMode::Absolute,    cycles: 3, handler: jmp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4D, name: "EOR", mode: AddressingMode::Absolute,    cycles: 4, handler: eor,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4E, name: "LSR", mode: AddressingMode::Absolute,    cycles: 6, handler: lsr,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x4F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x50, name: "BVC", mode: AddressingMode::Relative,    cycles: 2, handler: bvc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x51, name: "EOR", mode: AddressingMode::IndirectY,   cycles: 5, handler: eor,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x52, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x53, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x54, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x55, name: "EOR", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: eor,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x56, name: "LSR", mode: AddressingMode::ZeroPageX,   cycles: 6, handler: lsr,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x57, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x58, name: "CLI", mode: AddressingMode::Implied,     cycles: 2, handler: unimplemented,  illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x59, name: "EOR", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: eor,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x5A, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x5B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x5C, name: "NOP", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x5D, name: "EOR", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: eor,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x5E, name: "LSR", mode: AddressingMode::AbsoluteX,   cycles: 7, handler: lsr,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x5F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x60, name: "RTS", mode: AddressingMode::Implied,     cycles: 6, handler: rts,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x61, name: "ADC", mode: AddressingMode::IndirectX,   cycles: 6, handler: adc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x62, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x63, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x64, name: "NOP", mode: AddressingMode::ZeroPage,    cycles: 3, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x65, name: "ADC", mode: AddressingMode::ZeroPage,    cycles: 3, handler: adc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x66, name: "ROR", mode: AddressingMode::ZeroPage,    cycles: 5, handler: ror,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x67, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x68, name: "PLA", mode: AddressingMode::Implied,     cycles: 4, handler: pla,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x69, name: "ADC", mode: AddressingMode::Immediate,   cycles: 2, handler: adc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6A, name: "ROR", mode: AddressingMode::Accumulator, cycles: 2, handler: ror,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x6C, name: "JMP", mode: AddressingMode::Indirect,    cycles: 5, handler: jmp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6D, name: "ADC", mode: AddressingMode::Absolute,    cycles: 4, handler: adc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6E, name: "ROR", mode: AddressingMode::Absolute,    cycles: 6, handler: ror,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x6F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x70, name: "BVS", mode: AddressingMode::Relative,    cycles: 2, handler: bvs,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x71, name: "ADC", mode: AddressingMode::IndirectY,   cycles: 5, handler: adc,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x72, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x73, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x74, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x75, name: "ADC", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: adc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x76, name: "ROR", mode: AddressingMode::ZeroPageX,   cycles: 6, handler: ror,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x77, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x78, name: "SEI", mode: AddressingMode::Implied,     cycles: 2, handler: sei,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x79, name: "ADC", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: adc,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x7A, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x7B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x7C, name: "NOP", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x7D, name: "ADC", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: adc,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0x7E, name: "ROR", mode: AddressingMode::AbsoluteX,   cycles: 7, handler: ror,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x7F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x80, name: "NOP", mode: AddressingMode::Immediate,   cycles: 3, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x81, name: "STA", mode: AddressingMode::IndirectX,   cycles: 6, handler: sta,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x82, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x83, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x84, name: "STY", mode: AddressingMode::ZeroPage,    cycles: 3, handler: sty,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x85, name: "STA", mode: AddressingMode::ZeroPage,    cycles: 3, handler: sta,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x86, name: "STX", mode: AddressingMode::ZeroPage,    cycles: 3, handler: stx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x87, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x88, name: "DEY", mode: AddressingMode::Implied,     cycles: 2, handler: dey,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x89, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x8A, name: "TXA", mode: AddressingMode::Implied,     cycles: 2, handler: txa,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8B, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x8C, name: "STY", mode: AddressingMode::Absolute,    cycles: 4, handler: sty,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8D, name: "STA", mode: AddressingMode::Absolute,    cycles: 4, handler: sta,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8E, name: "STX", mode: AddressingMode::Absolute,    cycles: 4, handler: stx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x8F, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0x90, name: "BCC", mode: AddressingMode::Relative,    cycles: 2, handler: bcc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x91, name: "STA", mode: AddressingMode::IndirectY,   cycles: 6, handler: sta,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x92, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x93, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x94, name: "STY", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: sty,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x95, name: "STA", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: sta,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x96, name: "STX", mode: AddressingMode::ZeroPageY,   cycles: 4, handler: stx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x97, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x98, name: "TYA", mode: AddressingMode::Implied,     cycles: 2, handler: tya,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x99, name: "STA", mode: AddressingMode::AbsoluteY,   cycles: 5, handler: sta,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x9A, name: "TXS", mode: AddressingMode::Implied,     cycles: 2, handler: txs,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x9B, name: "TAS", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x9C, name: "SHY", mode: AddressingMode::AbsoluteX,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x9D, name: "STA", mode: AddressingMode::AbsoluteX,   cycles: 5, handler: sta,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0x9E, name: "SHX", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0x9F, name: "SHA", mode: AddressingMode::AbsoluteY,   cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xA0, name: "LDY", mode: AddressingMode::Immediate,   cycles: 2, handler: ldy,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA1, name: "LDA", mode: AddressingMode::IndirectX,   cycles: 6, handler: lda,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA2, name: "LDX", mode: AddressingMode::Immediate,   cycles: 2, handler: ldx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xA4, name: "LDY", mode: AddressingMode::ZeroPage,    cycles: 3, handler: ldy,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA5, name: "LDA", mode: AddressingMode::ZeroPage,    cycles: 3, handler: lda,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA6, name: "LDX", mode: AddressingMode::ZeroPage,    cycles: 3, handler: ldx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xA8, name: "TAY", mode: AddressingMode::Implied,     cycles: 2, handler: tay,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xA9, name: "LDA", mode: AddressingMode::Immediate,   cycles: 2, handler: lda,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAA, name: "TAX", mode: AddressingMode::Implied,     cycles: 2, handler: tax,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xAC, name: "LDY", mode: AddressingMode::Absolute,    cycles: 4, handler: ldy,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAD, name: "LDA", mode: AddressingMode::Absolute,    cycles: 4, handler: lda,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAE, name: "LDX", mode: AddressingMode::Absolute,    cycles: 4, handler: ldx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xAF, name: "LAX", mode: AddressingMode::Absolute,    cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xB0, name: "BCS", mode: AddressingMode::Relative,    cycles: 2, handler: bcs,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB1, name: "LDA", mode: AddressingMode::IndirectY,   cycles: 5, handler: lda,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xB2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xB3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xB4, name: "LDY", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: ldy,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB5, name: "LDA", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: lda,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB6, name: "LDX", mode: AddressingMode::ZeroPageY,   cycles: 4, handler: ldx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xB8, name: "CLV", mode: AddressingMode::Implied,     cycles: 2, handler: clv,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xB9, name: "LDA", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: lda,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBA, name: "TSX", mode: AddressingMode::Implied,     cycles: 2, handler: tsx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xBB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xBC, name: "LDY", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: ldy,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBD, name: "LDA", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: lda,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBE, name: "LDX", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: ldx,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xBF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xC0, name: "CPY", mode: AddressingMode::Immediate,   cycles: 2, handler: cpy,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC1, name: "CMP", mode: AddressingMode::IndirectX,   cycles: 6, handler: cmp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xC3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xC4, name: "CPY", mode: AddressingMode::ZeroPage,    cycles: 3, handler: cpy,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC5, name: "CMP", mode: AddressingMode::ZeroPage,    cycles: 3, handler: cmp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC6, name: "DEC", mode: AddressingMode::ZeroPage,    cycles: 5, handler: dec,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xC8, name: "INY", mode: AddressingMode::Implied,     cycles: 2, handler: iny,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xC9, name: "CMP", mode: AddressingMode::Immediate,   cycles: 2, handler: cmp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCA, name: "DEX", mode: AddressingMode::Implied,     cycles: 2, handler: dex,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xCC, name: "CPY", mode: AddressingMode::Absolute,    cycles: 4, handler: cpy,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCD, name: "CMP", mode: AddressingMode::Absolute,    cycles: 4, handler: cmp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCE, name: "DEC", mode: AddressingMode::Absolute,    cycles: 6, handler: dec,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xCF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xD0, name: "BNE", mode: AddressingMode::Relative,    cycles: 2, handler: bne,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD1, name: "CMP", mode: AddressingMode::IndirectY,   cycles: 5, handler: cmp,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xD2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD4, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD5, name: "CMP", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: cmp,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD6, name: "DEC", mode: AddressingMode::ZeroPageX,   cycles: 6, handler: dec,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xD8, name: "CLD", mode: AddressingMode::Implied,     cycles: 2, handler: cld,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xD9, name: "CMP", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: cmp,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xDA, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xDB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xDC, name: "NOP", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xDD, name: "CMP", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: cmp,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xDE, name: "DEC", mode: AddressingMode::AbsoluteX,   cycles: 7, handler: dec,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xDF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xE0, name: "CPX", mode: AddressingMode::Immediate,   cycles: 2, handler: cpx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE1, name: "SBC", mode: AddressingMode::IndirectX,   cycles: 6, handler: sbc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xE3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xE4, name: "CPX", mode: AddressingMode::ZeroPage,    cycles: 3, handler: cpx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE5, name: "SBC", mode: AddressingMode::ZeroPage,    cycles: 3, handler: sbc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE6, name: "INC", mode: AddressingMode::ZeroPage,    cycles: 5, handler: inc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xE8, name: "INX", mode: AddressingMode::Implied,     cycles: 2, handler: inx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xE9, name: "SBC", mode: AddressingMode::Immediate,   cycles: 2, handler: sbc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEA, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xEC, name: "CPX", mode: AddressingMode::Absolute,    cycles: 4, handler: cpx,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xED, name: "SBC", mode: AddressingMode::Absolute,    cycles: 4, handler: sbc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEE, name: "INC", mode: AddressingMode::Absolute,    cycles: 6, handler: inc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xEF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },

    Instruction { opcode: 0xF0, name: "BEQ", mode: AddressingMode::Relative,    cycles: 2, handler: beq,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF1, name: "SBC", mode: AddressingMode::IndirectY,   cycles: 5, handler: sbc,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xF2, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF3, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF4, name: "NOP", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF5, name: "SBC", mode: AddressingMode::ZeroPageX,   cycles: 4, handler: sbc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF6, name: "INC", mode: AddressingMode::ZeroPageX,   cycles: 6, handler: inc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF7, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xF8, name: "SED", mode: AddressingMode::Implied,     cycles: 2, handler: sed,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xF9, name: "SBC", mode: AddressingMode::AbsoluteY,   cycles: 4, handler: sbc,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xFA, name: "NOP", mode: AddressingMode::Implied,     cycles: 2, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xFB, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xFC, name: "NOP", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: nop,            illegal: true,      extra_on_page_cross: false  },
    Instruction { opcode: 0xFD, name: "SBC", mode: AddressingMode::AbsoluteX,   cycles: 4, handler: sbc,            illegal: false,     extra_on_page_cross: true   },
    Instruction { opcode: 0xFE, name: "INC", mode: AddressingMode::AbsoluteX,   cycles: 7, handler: inc,            illegal: false,     extra_on_page_cross: false  },
    Instruction { opcode: 0xFF, name: "", mode: AddressingMode::Implied,        cycles: 2, handler: unimplemented,  illegal: true,      extra_on_page_cross: false  },
];

/* Helpers */

fn same_page (a: u16, b: u16) -> bool {
    (a & 0xFF00) == (b & 0xFF00)
}

fn branch (cpu: &mut Cpu, address: u16) {
    cpu.cycles += if !same_page(address, cpu.pc) { 2 } else { 1 };
    cpu.pc = address;
}

/* Instruction handlers */

fn nop (_cpu: &mut Cpu, _operand: Operand, _bus: &mut Bus) {}

fn unimplemented (_cpu: &mut Cpu, _operand: Operand, _bus: &mut Bus) {
    unimplemented!();
}

/**
 * Add Memory to Accumulator with Carry
 */
fn adc (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
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
fn and (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a &= value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * Shift Left One Bit
 */
fn asl (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
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
            let new_value = lsr_inner(cpu, bus.read(address));
            bus.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * Branch on Carry Clear
 */
fn bcc (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Carry) {
        branch(cpu, address);
    }
}

/**
 * Branch on Carry Set
 */
fn bcs (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Carry) {
        branch(cpu, address);
    }
}

/**
 * Branch on Result Zero
 */
fn beq (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Zero) {
        branch(cpu, address);
    }
}

/**
 * Test Bits in Memory with Accumulator
 */
fn bit (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Zero, (cpu.a & value) == 0);
    cpu.set_flag(StatusFlag::Overflow, (value & StatusFlag::Overflow as u8) != 0);
    cpu.set_flag(StatusFlag::Negative, (value & StatusFlag::Negative as u8) != 0);
}

/**
 * Branch on Result Minus
 */
fn bmi (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Negative) {
        branch(cpu, address);
    }
}

/**
 * Branch on Result not Zero
 */
fn bne (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Zero) {
        branch(cpu, address);
    }
}

/**
 * Branch on Result Plus
 */
fn bpl (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Negative) {
        branch(cpu, address);
    }
}

/**
 * Force Break
 */
fn brk (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    }

    let (hi, lo) = (((cpu.pc + 2) >> 8) as u8, (cpu.pc + 2) as u8);
    cpu.push_stack(bus, hi);
    cpu.push_stack(bus, lo);

    cpu.push_stack(bus, cpu.status | (StatusFlag::Break as u8) | (StatusFlag::Unused as u8));
    cpu.set_flag(StatusFlag::DisableInterrupt, true);

    let address = (bus.read(Interrupt::IRQ as u16 + 1) as u16) << 8 | bus.read(Interrupt::IRQ as u16) as u16;
    cpu.pc = address;
}

/**
 * Branch on Overflow Clear
 */
fn bvc (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if !cpu.get_flag(StatusFlag::Overflow) {
        branch(cpu, address);
    }
}

/**
 * Branch on Overflow Set
 */
fn bvs (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    if cpu.get_flag(StatusFlag::Overflow) {
        branch(cpu, address);
    }
}

/**
 * Clear Carry Flag
 */
fn clc (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Carry, false);
}

/**
 * Clear Overflow Flag
 */
fn clv (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Overflow, false);
}

/**
 * Clear Decimal Mode
 */
fn cld (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Decimal, false);
}

/**
 * Compare Memory with Accumulator
 */
fn cmp (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Carry, cpu.a >= value);
    cpu.set_flag(StatusFlag::Zero, cpu.a == value);
    cpu.set_flag(StatusFlag::Negative, (cpu.a.wrapping_sub(value) as i8) < 0);
}

/**
 * Compare Memory and Index X
 */
fn cpx (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Carry, cpu.x >= value);
    cpu.set_flag(StatusFlag::Zero, cpu.x == value);
    cpu.set_flag(StatusFlag::Negative, (cpu.x.wrapping_sub(value) as i8) < 0);
}

/**
 * Compare Memory and Index Y
 */
fn cpy (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.set_flag(StatusFlag::Carry, cpu.y >= value);
    cpu.set_flag(StatusFlag::Zero, cpu.y == value);
    cpu.set_flag(StatusFlag::Negative, (cpu.y.wrapping_sub(value) as i8) < 0);
}

/**
 * Decrement Memory by One
 */
fn dec (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    let value = bus.read(address).wrapping_sub(1);
    bus.write(address, value);

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Decrement Index X by One
 */
fn dex (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn dey (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn eor (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a ^= value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * Increment Memory by One
 */
fn inc (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    let value = bus.read(address).wrapping_add(1);
    bus.write(address, value);

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Increment Index X by One
 */
fn inx (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn iny (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn jmp (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.pc = address;
}

/**
 * Jump to New Location Saving Return Address
 */
fn jsr (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    let (hi, lo) = (((cpu.pc - 1) >> 8) as u8, (cpu.pc - 1) as u8);
    cpu.push_stack(bus, hi);
    cpu.push_stack(bus, lo);
    cpu.pc = address;
}

/**
 * Loads a byte of memory into the X register setting the zero and negative flags as appropriate.
 */
fn ldx (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.x = value;

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Load Accumulator with Memory
 */
fn lda (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a = value;

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Load Index Y with Memory
 */
fn ldy (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.y = value;

    cpu.set_flag(StatusFlag::Zero, value == 0);
    cpu.set_flag(StatusFlag::Negative, (value as i8) < 0);
}

/**
 * Shift One Bit Right
 */
fn lsr (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
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
            let new_value = lsr_inner(cpu, bus.read(address));
            bus.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * OR Memory with Accumulator
 */
fn ora (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a |= value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}

/**
 * Push Accumulator on Stack
 */
fn pha (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::None => cpu.a,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.push_stack(bus, value);
}

/**
 * Push Processor Status on Stack
 */
fn php (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let status = match operand {
        Operand::None => cpu.status,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.push_stack(bus, status | (StatusFlag::Break as u8) | (StatusFlag::Unused as u8));
}

/**
 * Pull Accumulator from Stack
 */
fn pla (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::None => cpu.pop_stack(bus),
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a = value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a & StatusFlag::Negative as u8) > 0);
}

/**
 * Pull Processor Status from Stack
 */
fn plp (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    // Ignored flags, stay as-is
    let mask = (StatusFlag::Break as u8) | (StatusFlag::Unused as u8);
    let status = cpu.pop_stack(bus);
    cpu.status = (status & !mask) | (cpu.status & mask);
}

/**
 * Rotate One Bit Left
 */
fn rol (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
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
            let new_value = ror_inner(cpu, bus.read(address));
            bus.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * Rotate One Bit Right
 */
fn ror (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
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
            let new_value = ror_inner(cpu, bus.read(address));
            bus.write(address, new_value);
        },
        _ => panic!("Invalid addressing mode"),
    };
}

/**
 * Return from Interrupt
 */
fn rti (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    plp(cpu, operand, bus);
    let (lo, hi) = (cpu.pop_stack(bus), cpu.pop_stack(bus));
    let address = (hi as u16) << 8 | lo as u16;
    
    cpu.pc = address;
}

/**
 * Return from Subroutine
 */
fn rts (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    let (lo, hi) = (cpu.pop_stack(bus), cpu.pop_stack(bus));
    let address = (hi as u16) << 8 | lo as u16;

    cpu.pc = address + 1;
}

/**
 * Subtract Memory from Accumulator with Borrow
 */
fn sbc (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let value = match operand {
        Operand::Byte (byte) => byte,
        Operand::Address (address) => bus.read(address),
        _ => panic!("Invalid addressing mode"),
    };

    adc(cpu, Operand::Byte(!value), bus);
}

/**
 * Set Carry Flag
 */
fn sec (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Carry, true);
}

/**
 * Set Decimal Flag
 */
fn sed (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::Decimal, true);
}

/**
 * Set Interrupt Disable Status
 */
fn sei (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    if operand != Operand::None {
        panic!("Invalid addressing mode");
    };

    cpu.set_flag(StatusFlag::DisableInterrupt, true);
}

/**
 * Store Accumulator in Memory
 */
fn sta (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    bus.write(address, cpu.a);
}

/**
 * Store Index X in Memory
 */
fn stx (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    bus.write(address, cpu.x);
}

/**
 * Store Index Y in Memory
 */
fn sty (cpu: &mut Cpu, operand: Operand, bus: &mut Bus) {
    let address = match operand {
        Operand::Address (address) => address,
        _ => panic!("Invalid addressing mode"),
    };

    bus.write(address, cpu.y);
}

/**
 * Transfer Accumulator to Index X
 */
fn tax (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn tay (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn tsx (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn txa (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
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
fn txs (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let value = match operand {
        Operand::None => cpu.x,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.sp = value;
}

/**
 * Transfer Index Y to Accumulator
 */
fn tya (cpu: &mut Cpu, operand: Operand, _bus: &mut Bus) {
    let value = match operand {
        Operand::None => cpu.y,
        _ => panic!("Invalid addressing mode"),
    };

    cpu.a = value;

    cpu.set_flag(StatusFlag::Zero, cpu.a == 0);
    cpu.set_flag(StatusFlag::Negative, (cpu.a as i8) < 0);
}
