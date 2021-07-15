extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cartridge::Cartridge;

pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod bus;

trait MemoryMap {
    fn read (&self, a: u16) -> u8;
    // pub fn write (a: u16) -> u8 {}
}

#[wasm_bindgen]
pub struct Nes {
    cpu: Cpu,
    bus: Bus,
    cycles: usize,
}

#[wasm_bindgen]
impl Nes {
    pub fn new () -> Nes {
        return Nes {
            cpu: Cpu::new(),
            bus: Bus::new(),
            cycles: 0,
        };
    }

    /**
     * Load a ROM
     */
    pub fn load (&mut self, rom: &[u8]) {
        self.bus.cartridge = Some(Cartridge::new(rom));
        self.cpu.reset(&self.bus);
    }

    pub fn cycle (&mut self) {
        // 1/3
        if self.cycles % 3 == 0 {
            self.cpu.cycle(&mut self.bus);
        }
        self.bus.ppu.cycle(&self.bus);
        self.cycles += 1;
    }
}

#[wasm_bindgen]
pub fn set_panic_hook () {
    console_error_panic_hook::set_once();
}
