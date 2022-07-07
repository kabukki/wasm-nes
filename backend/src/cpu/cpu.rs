use wasm_bindgen::prelude::wasm_bindgen;
use crate::{
    cpu::{Interrupt, INTERRUPT_LATENCY},
    bus::Bus,
    clock::ClockDivider,
};

pub const MEMORY_RAM_STACK_START: u16 = 0x100;

#[wasm_bindgen(js_name = CpuStatusFlag)]
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
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub cycles: usize,
    pub interrupt: Option<Interrupt>,
    pub clock: ClockDivider,
}

impl Cpu {
    pub fn new () -> Cpu {
        Cpu {
            pc: 0x34,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            status: StatusFlag::Unused as u8,
            cycles: 0,
            interrupt: None,
            clock: ClockDivider::new(crate::clock::CLOCK_CPU_NTSC),
        }
    }

    pub fn tick (&mut self, time: f64, bus: &mut Bus) {
        if self.clock.tick(time) {
            let mut dma = bus.dma;
    
            match dma {
                Some (ref mut status) => {
                    if status.wait {
                        if self.clock.cycles % 2 == 1 {
                            status.wait = false;
                        }
                    } else {
                        // Copy 256 bytes over 512 cycles
                        if self.clock.cycles % 2 == 0 {
                            let address = ((status.page as u16) << 8) + status.count as u16;
                            status.read_buffer = bus.read(address);
                        } else {
                            bus.ppu.write_oam(status.read_buffer);
    
                            if status.count < u8::MAX {
                                status.count += 1;
                            } else {
                                dma = None;
                            }
                        }
                    }
    
                    bus.dma = dma;
                },
                None => {
                    self.cycle(bus);
                },
            }
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
                self.execute(bus);
            }
        }

        self.cycles -= 1;
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
        // println!("INTERRUPT {:?}", interrupt);
        match interrupt {
            Interrupt::NMI | Interrupt::IRQ => {
                if self.get_flag(StatusFlag::DisableInterrupt) && interrupt == Interrupt::IRQ {
                    log::warn!("Attempted to IRQ but was disabled");
                    return;
                }

                let (hi, lo) = ((self.pc >> 8) as u8, self.pc as u8);
                self.push_stack(bus, hi);
                self.push_stack(bus, lo);
                self.push_stack(bus, self.status | (StatusFlag::Unused as u8));
                self.set_flag(StatusFlag::DisableInterrupt, true);
                self.cycles = INTERRUPT_LATENCY;
            },
            Interrupt::RESET => {
                self.sp = self.sp.wrapping_sub(3);
                self.set_flag(StatusFlag::DisableInterrupt, true);
                self.cycles = INTERRUPT_LATENCY;
            },
        };

        self.pc = (bus.read(interrupt as u16 + 1) as u16) << 8 | bus.read(interrupt as u16) as u16;
    }

    pub fn interrupt_request (&mut self, interrupt: Interrupt) {
        self.interrupt = Some(interrupt);
    }

    pub fn reset (&mut self) {
        self.interrupt_request(Interrupt::RESET);
    }
}
