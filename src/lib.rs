extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;
use log::debug;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cartridge::Cartridge;

pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod bus;

/**
 * CPU state representation
 */
#[derive(PartialEq)]
pub struct State {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub sp: u8,
}

impl State {
    pub fn from_str (string: &String) -> State {
        let pc = &string[..4];
        let registers = &string[48..73];
        let (a, x, y, status, sp) = (&registers[2..4], &registers[7..9], &registers[12..14], &registers[17..19], &registers[23..25]);

        State {
            pc: u16::from_str_radix(pc, 16).unwrap(),
            a: u8::from_str_radix(a, 16).unwrap(),
            x: u8::from_str_radix(x, 16).unwrap(),
            y: u8::from_str_radix(y, 16).unwrap(),
            status: u8::from_str_radix(status, 16).unwrap(),
            sp: u8::from_str_radix(sp, 16).unwrap(),
        }
    }
    // pub fn to_str
}

// trait MemoryMap {
//     fn read (&self, a: u16) -> u8;
//     // pub fn write (a: u16) -> u8 {}
// }

#[wasm_bindgen]
pub struct Nes {
    cpu: Cpu,
    bus: Bus,
    cycles: usize,
}

#[wasm_bindgen]
impl Nes {
    pub fn new () -> Self {
        Nes {
            cpu: Cpu::new(),
            bus: Bus::new(),
            cycles: 0,
        }
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
            debug!("PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X}", self.cpu.pc, self.cpu.a, self.cpu.x, self.cpu.y, self.cpu.status, self.cpu.sp);
            self.cpu.cycle(&mut self.bus);
        }
        self.bus.ppu.cycle(&self.bus);
        self.cycles += 1;
    }
}

impl Default for Nes {
    fn default () -> Self {
        Nes::new()
    }
}

#[wasm_bindgen]
pub fn set_panic_hook () {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn set_log () {
    console_log::init_with_level(log::Level::Trace).expect("Could not set up logger");
}
