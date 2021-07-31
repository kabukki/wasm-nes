use log::{warn, info, trace};
use crate::bus::{Bus, MEMORY_RAM_STACK_START};
use crate::cpu::instruction::INSTRUCTIONS;
use crate::cpu::interrupt::Interrupt;

pub mod instruction;
pub mod interrupt;

pub enum StatusFlag {
    Carry               = 0b0000_0001,
    Zero                = 0b0000_0010,
    DisableInterrupt    = 0b0000_0100,
    Decimal             = 0b0000_1000,
    Break               = 0b0001_0000,
    Unused              = 0b0010_0000,
    Overflow            = 0b0100_0000,
    Negative            = 0b1000_0000,
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
     * Reamaining cycles
     */
    pub cycles: usize,

    /**
     * Total cycles run so far
     */
    pub cycles_total: usize,

    /**
     * Pending interrupt
     */
    pub interrupt: Option<Interrupt>,
}

impl Cpu {
    pub fn new () -> Cpu {
        Cpu {
            pc: 0xC000,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            status: (StatusFlag::Unused as u8 | StatusFlag::DisableInterrupt as u8),
            cycles: 0,
            cycles_total: 7,
            interrupt: None,
        }
    }

    /**
     * Run a single clock cycle
     */
    pub fn cycle (&mut self, bus: &mut Bus) {
        if self.cycles == 0 {
            if self.interrupt.is_some() {
                self.interrupt(self.interrupt.unwrap(), bus);
                self.interrupt = None;
            } else {
                let instruction = &INSTRUCTIONS[bus.read(self.pc) as usize];
                // debug!("{:#?}", instruction);
                self.pc += 1;
    
                let cycles = instruction.execute(self, bus);
                self.cycles += cycles as usize;
            }
            
            // trace!("PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X}", self.pc, self.a, self.x, self.y, self.status, self.sp);
        }
        
        self.cycles -= 1;
        self.cycles_total += 1;
    }

    /**
     * Runs cycles until next instruction
     */
    pub fn cycle_full (&mut self, bus: &mut Bus) {
        loop {
            self.cycle(bus);
            if self.cycles == 0 {
                break;
            }
        }
    }

    pub fn push_stack (&mut self, bus: &mut Bus, data: u8) {
        bus.write(MEMORY_RAM_STACK_START + self.sp as u16, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pop_stack (&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.read(MEMORY_RAM_STACK_START + self.sp as u16)
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

    pub fn interrupt (&mut self, interrupt: Interrupt, bus: &mut Bus) {
        // info!("INTERRUPT {:?}", interrupt);

        if self.get_flag(StatusFlag::DisableInterrupt) && interrupt == Interrupt::IRQ {
            warn!("Attempted to IRQ but was disabled");
            return;
        }

        if interrupt != Interrupt::RESET {
            let (hi, lo) = ((self.pc >> 8) as u8, self.pc as u8);
            self.push_stack(bus, hi);
            self.push_stack(bus, lo);
    
            self.push_stack(bus, self.status | (StatusFlag::Unused as u8));
            self.set_flag(StatusFlag::DisableInterrupt, true);
        }

        self.pc = (bus.read(interrupt as u16 + 1) as u16) << 8 | bus.read(interrupt as u16) as u16;
        // info!("PC now at {:x}", self.pc);
        self.cycles = 6;
    }

    pub fn interrupt_request (&mut self, interrupt: Interrupt) {
        self.interrupt = Some(interrupt);
    }
    
    pub fn reset (&mut self) {
        self.interrupt_request(Interrupt::RESET);
    }
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "PC = {:#x}, A = {}, X = {}, Y = {}, Status = {:#010b}", self.pc, self.a, self.x, self.y, self.status)
        write!(f, "{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}", self.pc, self.a, self.x, self.y, self.status, self.sp, self.cycles)
    }
}

impl Default for Cpu {
    fn default () -> Self {
        Cpu::new()
    }
}
