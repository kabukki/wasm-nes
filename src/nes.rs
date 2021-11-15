use wasm_bindgen::prelude::*;
use crate::{
    debug::Probe,
    bus::Bus,
    cpu::Cpu,
    clock::{Clock, ClockDivider},
    cartridge::CartridgeDebug,
    ppu::PpuDebug,
};

#[wasm_bindgen]
pub struct Nes {
    cpu: Cpu,
    bus: Bus,
    clock: Clock,
    clock_apu: ClockDivider,
    clock_apu_sample: ClockDivider,
    clock_cpu: ClockDivider,
    clock_ppu: ClockDivider,
}

#[wasm_bindgen]
impl Nes {
    pub fn new (rom: Vec<u8>, sample_rate: f64) -> Self {
        let mut emulator = Nes {
            cpu: Cpu::new(),
            bus: Bus::new(),
            clock: Clock::new(crate::clock::NTSC_CLOCK_MASTER),
            clock_apu: ClockDivider::new(crate::clock::NTSC_CLOCK_APU),
            clock_apu_sample: ClockDivider::new(sample_rate),
            clock_cpu: ClockDivider::new(crate::clock::NTSC_CLOCK_CPU),
            clock_ppu: ClockDivider::new(crate::clock::NTSC_CLOCK_PPU),
        };

        emulator.bus.load(&rom);
        emulator.cpu.reset();

        emulator
    }

    /**
     * Cycle once
     */
    pub fn cycle (&mut self) {
        // CPU
        if self.clock_cpu.tick(self.clock.time) {
            let mut dma = self.bus.dma;
    
            match dma {
                Some (ref mut status) => {
                    if status.wait {
                        if self.clock_cpu.cycles % 2 == 1 {
                            status.wait = false;
                        }
                    } else {
                        // Copy 256 bytes over 512 cycles
                        if self.clock_cpu.cycles % 2 == 0 {
                            let address = ((status.page as u16) << 8) + status.count as u16;
                            status.read_buffer = self.bus.read(address);
                        } else {
                            self.bus.ppu.write_oam(status.read_buffer);
    
                            if status.count < u8::MAX {
                                status.count += 1;
                            } else {
                                dma = None;
                            }
                        }
                    }
    
                    self.bus.dma = dma;
                },    
                None => {
                    self.cpu.cycle(&mut self.bus);
                },
            }
        }

        // APU
        if self.clock_apu.tick(self.clock.time) {
            self.bus.apu.cycle(&mut self.cpu);
        }

        if self.clock_apu_sample.tick(self.clock.time) {
            self.bus.apu.sample();
        }

        // PPU
        if self.clock_ppu.tick(self.clock.time) {
            self.bus.ppu.cycle(&self.bus.cartridge.as_ref().unwrap(), &mut self.cpu);
        }

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
                cpu_cycles: self.clock_cpu.cycles,
                ppu_cycles: self.clock_ppu.cycles,
                apu_cycles: self.clock_apu.cycles,
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
