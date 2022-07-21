use wasm_bindgen::prelude::*;
use crate::{
    bus::Bus,
    cpu::Cpu,
    clock::Clock,
};

#[wasm_bindgen]
pub struct Emulator {
    pub (crate) cpu: Cpu,
    pub (crate) bus: Bus,
    pub (crate) clock: Clock,
}

#[wasm_bindgen]
impl Emulator {
    pub fn new (rom: Vec<u8>, sample_rate: f64) -> Self {
        let mut emulator = Self {
            cpu: Cpu::new(),
            bus: Bus::new(&rom, sample_rate),
            clock: Clock::new(crate::clock::CLOCK_MASTER_NTSC),
        };

        emulator.cpu.reset();

        emulator
    }

    /**
     * Run one master clock cycle
     */
    pub fn cycle (&mut self) {
        self.cpu.tick(self.clock.time, &mut self.bus);
        self.bus.apu.tick(self.clock.time, &mut self.cpu);
        self.bus.ppu.tick(self.clock.time, &self.bus.cartridge, &mut self.cpu);

        self.clock.tick();
    }

    /**
     * Cycle until frame is rendered
     */
    pub fn cycle_until_frame (&mut self) {
        let frame = self.bus.ppu.frame;

        while frame == self.bus.ppu.frame {
            self.cycle();
        }
    }

    pub fn cycle_until_scanline (&mut self) {
        let cycle = self.bus.ppu.scanline;

        while cycle == self.bus.ppu.scanline {
            self.cycle();
        }
    }

    pub fn cycle_until_ppu (&mut self) {
        let cycle = self.bus.ppu.clock.cycles;

        while cycle == self.bus.ppu.clock.cycles {
            self.cycle();
        }
    }

    pub fn cycle_until_cpu (&mut self) {
        let cycle = self.cpu.clock.cycles;

        while cycle == self.cpu.clock.cycles {
            self.cycle();
        }
    }

    // pub fn set_rate (&mut self) {
    //     self.clock.rate = crate::util::CLOCK_MASTER_PAL;
    // }

    pub fn update_controllers (&mut self, data: &[u8]) {
        self.bus.controllers[0].update(data[0]);
        self.bus.controllers[1].update(data[1]);
    }

    /**
     * https://wiki.nesdev.org/w/index.php/Init_code
     */
    pub fn reset (&mut self) {
        self.cpu.reset();
        self.bus.apu.reset();
        self.clock.reset();
    }

    pub fn read (&mut self, address: u16) -> u8 {
        self.bus.read(address)
    }

    pub fn get_audio (&mut self) -> Vec<f32> {
        self.bus.apu.flush()
    }

    pub fn get_framebuffer (&self) -> js_sys::Uint8ClampedArray {
        unsafe { js_sys::Uint8ClampedArray::view(&self.bus.ppu.framebuffer) }
    }
}
