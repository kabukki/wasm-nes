use crate::{
    bus::Bus,
    cpu::{Instruction, Operand, AddressingMode},
};

pub struct Disassembly {
    address: String,
    operator: String,
    bytes: String,
    operand: Option<String>,
}

impl Disassembly {
    pub fn new (instruction: Instruction, operand: Operand, address: u16) -> Self {
        Self {
            address: format!("{:04X}", address),
            operator: format!("{:?}", instruction.operator),
            operand: match (instruction.mode, operand) {
                (AddressingMode::Implied,       Operand::None)              => None,
                (AddressingMode::Accumulator,   Operand::None)              => Some(format!("A")),
                (AddressingMode::Immediate,     Operand::Byte (byte))       => Some(format!("#${:02X}", byte)),
                (AddressingMode::Relative,      Operand::Byte (byte))       => Some(format!("${:02X}", byte)),
                (AddressingMode::ZeroPage,      Operand::Byte (byte))       => Some(format!("${:02X}", byte)),
                (AddressingMode::ZeroPageX,     Operand::Byte (byte))       => Some(format!("${:02X},X", byte)),
                (AddressingMode::ZeroPageY,     Operand::Byte (byte))       => Some(format!("${:02X},Y", byte)),
                (AddressingMode::IndirectX,     Operand::Byte (byte))       => Some(format!("(${:02X}),X", byte)),
                (AddressingMode::IndirectY,     Operand::Byte (byte))       => Some(format!("(${:02X}),Y", byte)),
                (AddressingMode::Absolute,      Operand::Address (address)) => Some(format!("${:04X}", address)),
                (AddressingMode::AbsoluteX,     Operand::Address (address)) => Some(format!("${:04X},X", address)),
                (AddressingMode::AbsoluteY,     Operand::Address (address)) => Some(format!("${:04X},Y", address)),
                (AddressingMode::Indirect,      Operand::Address (address)) => Some(format!("(${:04X})", address)),
                (_,                             _)                          => unreachable!(),
            },
            bytes: match operand {
                Operand::None               => format!("{:02X} -- --", instruction.opcode),
                Operand::Byte (byte)        => format!("{:02X} {:02X} --", instruction.opcode, byte),
                Operand::Address (address)  => format!("{:02X} {:02X} {:02X}", instruction.opcode, (address & 0x00FF) as u8, (address & 0xFF00) >> 8 as u8),
            },
        }
    }
}

impl std::fmt::Display for Disassembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = vec![self.address.clone(), self.bytes.clone(), self.operator.clone()];

        match self.operand.clone() {
            None => {},
            Some (operand) => { parts.push(operand); },
        }

        write!(f, "{}", parts.join(" "))
    }
}

impl Bus {
    pub fn disassemble (&mut self) -> Vec<Disassembly> {
        let mut list = vec![];
        let mut address = 0x8000;

        loop {
            let (instruction, operand, read) = self.fetch_instruction(address);
            let (new_address, overflow) = address.overflowing_add(read);
            let disassembly = Disassembly::new(instruction, operand, address);

            list.push(disassembly);

            if overflow {
                break;
            } else {
                address = new_address;
            }
        }

        list
    }
}
