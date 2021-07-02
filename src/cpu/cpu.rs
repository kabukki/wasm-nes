use crate::cpu::memory::{Memory, MEMORY_RAM_STACK_START};
use crate::cpu::instruction::INSTRUCTIONS;

pub enum StatusFlag {
    Carry       = 0b0000_0001,
    Zero        = 0b0000_0010,
    Interrupt   = 0b0000_0100,
    Decimal     = 0b0000_1000,
    Break       = 0b0001_0000,
    Unused      = 0b0010_0000,
    Overflow    = 0b0100_0000,
    Negative    = 0b1000_0000,
}

/**
 * MOS 6502 CPU
 */
pub struct Cpu {
    /**
     * Program counter
     */
    pub pc: u16,

    /**
     * Stack pointer
     */
    pub sp: u8,

    /**
     * A register
     */
    pub a: u8,

    /**
     * X register
     */
    pub x: u8,
    
    /**
     * Y register
     */
    pub y: u8,

    /**
     * Status register
     */
    pub status: u8,

    /**
     * Elapsed cycles
     */
    pub cycles: usize,
}

impl Cpu {
    pub fn new () -> Cpu {
        return Cpu {
            pc: 0,
            sp: 0xFD, // nestest
            a: 0,
            x: 0,
            y: 0,
            status: 0b00100100,
            cycles: 0,
        };
    }

    pub fn cycle (&mut self, memory: &mut Memory) {
        let instruction = &INSTRUCTIONS[memory.read(self.pc) as usize];
        self.pc += 1;
    
        let cycles = instruction.execute(self, memory);
        self.cycles += cycles as usize;
    }

    pub fn push_stack (&mut self, memory: &mut Memory, data: u8) {
        memory.write(MEMORY_RAM_STACK_START + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop_stack (&mut self, memory: &mut Memory) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        memory.read(MEMORY_RAM_STACK_START + self.sp as u16)
    }

    pub fn set_flag (&mut self, flag: StatusFlag, condition: bool) {
        if condition {
            self.status |= flag as u8;
        } else {
            self.status &= !(flag as u8);
        }
    }

    pub fn get_flag (&mut self, flag: StatusFlag) -> bool {
        (self.status & flag as u8) != 0
    }
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "PC = {:#x}, A = {}, X = {}, Y = {}, Status = {:#010b}", self.pc, self.a, self.x, self.y, self.status)
        write!(f, "{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}", self.pc, self.a, self.x, self.y, self.status, self.sp, self.cycles)
    }
}
