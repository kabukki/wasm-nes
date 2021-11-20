use wasm_bindgen::prelude::*;
use crate::{
    debug::Probe,
    bus::Bus,
    cpu::Cpu,
    clock::Clock,
    cartridge::CartridgeDebug,
    ppu::PpuDebug,
};

#[wasm_bindgen]
pub struct Nes {
    cpu: Cpu,
    bus: Bus,
    clock: Clock,
}

#[wasm_bindgen]
impl Nes {
    pub fn new (rom: Vec<u8>, sample_rate: f64) -> Self {
        let mut emulator = Nes {
            cpu: Cpu::new(),
            bus: Bus::new(sample_rate),
            clock: Clock::new(crate::clock::NTSC_CLOCK_MASTER),
        };

        emulator.bus.load(&rom);
        emulator.cpu.reset();

        emulator
    }

    /**
     * Run one master clock cycle
     */
    pub fn cycle (&mut self) {
        self.cpu.tick(self.clock.time, &mut self.bus);
        self.bus.apu.tick(self.clock.time, &mut self.cpu);
        self.bus.ppu.tick(self.clock.time, &self.bus.cartridge.as_ref().unwrap(), &mut self.cpu);

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

    pub fn get_framebuffer (&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.bus.ppu.framebuffer.to_vec());
    }

    pub fn get_cartridge_ram (&self) -> Vec<u8> {
        self.bus.cartridge.as_ref().unwrap().prg_ram.to_vec()
    }

    pub fn set_cartridge_ram (&mut self, prg_ram: Vec<u8>) {
        self.bus.cartridge.as_mut().unwrap().prg_ram.copy_from_slice(&prg_ram);
    }

    pub fn get_audio (&mut self) -> Vec<f32> {
        self.bus.apu.flush()
    }

    /**
     * Not implemented through Probe trait because impls are not yet supported by wasm-bindgen.
     * https://github.com/rustwasm/wasm-bindgen/issues/2073
     */
    pub fn get_debug (&mut self) -> NesDebug {
        let cartridge = self.bus.cartridge.as_ref().unwrap();

        NesDebug {
            ram: self.bus.wram.to_vec(),
            time: TimeDebug {
                time: self.clock.time,
                cpu_cycles: self.cpu.clock.cycles,
                ppu_cycles: self.bus.ppu.clock.cycles,
                apu_cycles: self.bus.apu.clock.cycles,
            },
            ppu: self.bus.ppu.get_debug(cartridge),
            cartridge: cartridge.get_debug(cartridge),
        }
    }
}

#[wasm_bindgen]
pub struct NesDebug {
    ram: Vec<u8>,
    time: TimeDebug,
    ppu: PpuDebug,
    cartridge: CartridgeDebug,
}

#[wasm_bindgen]
impl NesDebug {
    #[wasm_bindgen(getter)]
    pub fn ram (&self) -> Vec<u8> {
        self.ram.to_owned()
    }

    #[wasm_bindgen(getter)]
    pub fn time (&self) -> TimeDebug {
        self.time.to_owned()
    }

    #[wasm_bindgen(getter)]
    pub fn ppu (&self) -> PpuDebug {
        self.ppu.to_owned()
    }

    #[wasm_bindgen(getter)]
    pub fn cartridge (&self) -> CartridgeDebug {
        self.cartridge.to_owned()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct TimeDebug {
    time: f64,
    cpu_cycles: usize,
    ppu_cycles: usize,
    apu_cycles: usize,
}

#[wasm_bindgen]
impl TimeDebug {
    #[wasm_bindgen(getter)]
    pub fn clock (&self) -> usize {
        (self.time * 1000.0) as usize
    }

    #[wasm_bindgen(getter = cpuCycles)]
    pub fn cpu_cycles (&self) -> usize {
        self.cpu_cycles
    }

    #[wasm_bindgen(getter = ppuCycles)]
    pub fn ppu_cycles (&self) -> usize {
        self.ppu_cycles
    }

    #[wasm_bindgen(getter = apuCycles)]
    pub fn apu_cycles (&self) -> usize {
        self.apu_cycles
    }
}
