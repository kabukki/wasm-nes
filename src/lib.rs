extern crate console_error_panic_hook;
extern crate wee_alloc;

use wasm_bindgen::{prelude::*, Clamped};
use wee_alloc::WeeAlloc;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::tilemap::Tilemap;

pub mod bus;
pub mod cartridge;
pub mod mapper;
pub mod cpu;
pub mod instruction;
pub mod ppu;
pub mod tilemap;
pub mod controller;

#[global_allocator]
static GLOBAL: WeeAlloc = WeeAlloc::INIT;

#[wasm_bindgen]
pub struct Nes {
    cpu: Cpu,
    bus: Bus,
    cycles: usize,
}

#[wasm_bindgen]
impl Nes {
    pub fn new (rom: Vec<u8>) -> Self {
        let mut emulator = Nes {
            cpu: Cpu::new(),
            bus: Bus::new(),
            cycles: 0,
        };

        emulator.bus.load(&rom);
        emulator.cpu.reset();

        emulator
    }

    /**
     * Cycle once
     */
    pub fn cycle (&mut self) {
        if self.cycles % 3 == 0 {
            let mut dma = self.bus.dma;

            match dma {
                Some (ref mut status) => {
                    if status.wait {
                        if self.cycles % 2 == 1 {
                            status.wait = false;
                        }
                    } else {
                        let address = (status.page as usize) << 8;
                        self.bus.ppu.write_oam_dma(&self.bus.wram[address .. address + 256]);
                        self.cpu.cycles = 512;
                        dma = None;
                    }

                    self.bus.dma = dma;
                },    
                None => {
                    self.cpu.cycle(&mut self.bus);
                },    
            }
        }

        self.bus.ppu.cycle(&self.bus.cartridge.as_ref().unwrap(), &mut self.cpu);
        self.cycles += 1;
    }

    /**
     * Cycle until frame is rendered
     */
    pub fn frame (&mut self) {
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
    }

    pub fn get_framebuffer (&self) -> Clamped<Vec<u8>> {
        Clamped(self.bus.ppu.framebuffer.to_vec())
    }

    /**
     * Get the contents of the CHR-ROM pattern tables.
     * Pattern tables contain background graphics (right) and sprite graphics (left)
     * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
     */
    pub fn get_pattern_tables (&self) -> Clamped<Vec<u8>> {
        let cartridge = self.bus.cartridge.as_ref().unwrap();
        let mut map = Tilemap::new(16, 32);
        let palette = &self.bus.ppu.palettes[..4];
    
        for n in 0..512 {
            let x = n % 16;
            let y = n / 16;

            let tile = cartridge.get_tile(n);
            map.write_tile(x, y, tile.as_slice(), palette);
        }
    
        Clamped(map.buffer)
    }

    /**
     * Get the palettes in use
     */
    pub fn get_palettes (&self) -> Clamped<Vec<u8>> {
        let mut map = Tilemap::new(16, 2);

        for n in 0..32 {
            let color = self.bus.ppu.palettes[n];
            let x = n % 16;
            let y = n / 16;
            let tile = vec![0; 8 * 8];
            map.write_tile(x, y, tile.as_slice(), &[color]);
        }

        Clamped(map.buffer)
    }

    /**
     * Get the system palette
     */
    pub fn get_palette (&self) -> Clamped<Vec<u8>> {
        let mut map = Tilemap::new(16, 4);

        for color in 0..64 {
            let tile = vec![0; 8 * 8];
            map.write_tile(color % 16, color / 16, tile.as_slice(), &[color as u8]);
        }

        Clamped(map.buffer)
    }

    pub fn get_ram (&self) -> Vec<u8> {
        self.bus.wram.to_vec()
    }

    pub fn get_nametable_ram (&self) -> Vec<u8> {
        self.bus.ppu.nametables.to_vec()
    }

    pub fn get_cartridge_ram (&self) -> Vec<u8> {
        self.bus.cartridge.as_ref().unwrap().prg_ram.to_vec()
    }

    pub fn set_cartridge_ram (&mut self, prg_ram: Vec<u8>) {
        self.bus.cartridge.as_mut().unwrap().prg_ram.copy_from_slice(&prg_ram);
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

#[wasm_bindgen]
pub fn fingerprint (data: Vec<u8>) -> String {
    use std::hash::Hasher;

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hasher.write(&data);
    format!("{:x}", hasher.finish())
}
