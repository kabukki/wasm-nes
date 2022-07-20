use wasm_bindgen::prelude::*;
use crate::{cpu, Emulator};

#[derive(serde::Serialize)]
pub struct Disassembly {
    address: u16,
    bank: u8,
    operator: String,
    bytes: String,
    operand: Option<String>,
}

impl Disassembly {
    pub fn new (instruction: cpu::Instruction, operand: cpu::Operand, address: u16, bank: u8) -> Self {
        Self {
            address,
            bank,
            operator: format!("{:?}", instruction.operator),
            operand: match (instruction.mode, operand) {
                (cpu::AddressingMode::Implied,       cpu::Operand::None)              => None,
                (cpu::AddressingMode::Accumulator,   cpu::Operand::None)              => Some(format!("A")),
                (cpu::AddressingMode::Immediate,     cpu::Operand::Byte (byte))       => Some(format!("#${:02X}", byte)),
                (cpu::AddressingMode::Relative,      cpu::Operand::Byte (byte))       => Some(format!("${:02X}", byte)),
                (cpu::AddressingMode::ZeroPage,      cpu::Operand::Byte (byte))       => Some(format!("${:02X}", byte)),
                (cpu::AddressingMode::ZeroPageX,     cpu::Operand::Byte (byte))       => Some(format!("${:02X},X", byte)),
                (cpu::AddressingMode::ZeroPageY,     cpu::Operand::Byte (byte))       => Some(format!("${:02X},Y", byte)),
                (cpu::AddressingMode::IndirectX,     cpu::Operand::Byte (byte))       => Some(format!("(${:02X}),X", byte)),
                (cpu::AddressingMode::IndirectY,     cpu::Operand::Byte (byte))       => Some(format!("(${:02X}),Y", byte)),
                (cpu::AddressingMode::Absolute,      cpu::Operand::Address (address)) => Some(format!("${:04X}", address)),
                (cpu::AddressingMode::AbsoluteX,     cpu::Operand::Address (address)) => Some(format!("${:04X},X", address)),
                (cpu::AddressingMode::AbsoluteY,     cpu::Operand::Address (address)) => Some(format!("${:04X},Y", address)),
                (cpu::AddressingMode::Indirect,      cpu::Operand::Address (address)) => Some(format!("(${:04X})", address)),
                (_,                             _)                          => unreachable!(),
            },
            bytes: match operand {
                cpu::Operand::None               => format!("{:02X} -- --", instruction.opcode),
                cpu::Operand::Byte (byte)        => format!("{:02X} {:02X} --", instruction.opcode, byte),
                cpu::Operand::Address (address)  => format!("{:02X} {:02X} {:02X}", instruction.opcode, (address & 0x00FF) as u8, (address & 0xFF00) >> 8 as u8),
            },
        }
    }
}

impl std::fmt::Display for Disassembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = vec![self.operator.clone()];

        match self.operand.clone() {
            None => {},
            Some (operand) => { parts.push(operand); },
        }

        write!(f, "{}", parts.join(" "))
    }
}

#[wasm_bindgen]
impl Emulator {
    pub fn debug_disassembly_at (&mut self, address: u16) -> JsValue {
        let (instruction, operand, _) = self.bus.fetch_instruction(address);
        let disassembly = Disassembly::new(instruction, operand, address, self.bus.cartridge.mapper.get_bank_at(&self.bus.cartridge.prg_rom, address));

        JsValue::from_serde(&disassembly).unwrap()
    }

    pub fn debug_disassembly_index_to_address (&mut self, offset: u16) -> u16 {
        let mut address = 0x8000;

        for _ in 0..offset {
            let (_, _, read) = self.bus.fetch_instruction(address);
            address += read;
        }

        address
    }

    pub fn debug_disassembly_address_to_index (&mut self, target: u16) -> u16 {
        let mut total = 0;
        let mut address = 0x8000;

        while address < target {
            let (_, _, read) = self.bus.fetch_instruction(address);
            address = address.saturating_add(read);
            total += 1 ;
        }

        total
    }

    pub fn debug_disassembly_total (&mut self) -> u16 {
        self.debug_disassembly_address_to_index(0xFFFF)
    }
}
