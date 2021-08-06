extern crate console_error_panic_hook;

use wasm_bindgen::{prelude::*, Clamped};
use log::{info, trace};
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::ppu::CtrlFlag;
use crate::tilemap::Tilemap;

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod instruction;
pub mod ppu;
pub mod tilemap;

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
    pub fn load (&mut self, rom: Vec<u8>) {
        self.bus.load(&rom);
        self.cpu.reset();
    }

    /**
     * Cycle once
     */
    pub fn cycle (&mut self) {
        self.cpu.cycle(&mut self.bus);
        self.cycles += 1;
    }

    /**
     * Cycle until frame is rendered
     */
    pub fn frame (&mut self) -> usize {
        let frame = self.bus.ppu.frame;

        while frame == self.bus.ppu.frame {
            if self.cycles % 3 == 0 {
                self.cpu.cycle(&mut self.bus);
            }
            self.bus.ppu.cycle(&self.bus.cartridge.as_ref().unwrap(), &mut self.cpu);
            // info!("Scanline {}, Dot {}", self.bus.ppu.scanline, self.bus.ppu.dot);
            self.cycles += 1;
        }

        frame + 1
    }

    pub fn get_framebuffer (&self) -> Clamped<Vec<u8>> {
        Clamped(self.bus.ppu.framebuffer.to_vec())
    }

    /**
     * https://wiki.nesdev.com/w/index.php/PPU_attribute_tables
     */
    pub fn get_nametable (&mut self, nth: u16) -> Clamped<Vec<u8>> {
        let cartridge = self.bus.cartridge.as_ref().unwrap();
        let start_address = self.bus.ppu.mirror(cartridge, 0x2000 + nth * 0x400);
        let mut map = Tilemap::new(32, 30);

        for n in 0 .. 960 {
            let (x, y) = (n % 32, n / 32);

            // Get tile
            let address = start_address + n;
            let index = self.bus.ppu.read_vram(cartridge, address) as usize;
            let tile = cartridge.get_tile(index + if (self.bus.ppu.ctrl & CtrlFlag::Background as u8) > 0 { 256 } else { 0 });

            // Get byte from attribute table. See https://github.com/OneLoneCoder/olcNES/blob/master/Part%20%234%20-%20PPU%20Backgrounds/olc2C02.cpp#L802
            let byte = self.bus.ppu.read_vram(cartridge, start_address + 960 + (x / 4) + (y / 4) * 8);

            // Get palette
            let number = match (x % 4 / 2, y % 4 / 2) {
                (0, 0) => (byte >> 0) & 0b11, // Top left
                (1, 0) => (byte >> 2) & 0b11, // Top right
                (0, 1) => (byte >> 4) & 0b11, // Bottom left
                (1, 1) => (byte >> 6) & 0b11, // Bottom right
                _ => panic!("Not possible"),
            };
            let palette = &self.bus.ppu.palettes[4 * number as usize .. 4 * number as usize + 4];

            // Draw tile
            map.write_tile(x as usize, y as usize, tile.as_slice(), palette);
        }

        Clamped(map.buffer)
    }

    /**
     * Get the contents of the CHR-ROM pattern tables.
     * Pattern tables contain background graphics (right) and sprite graphics (left)
     * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
     */
    pub fn get_pattern_tables (&self) -> Clamped<Vec<u8>> {
        let cartridge = self.bus.cartridge.as_ref().unwrap();
        let mut map = Tilemap::new(16, 32);
        let (bg_palette, fg_palette) = (&self.bus.ppu.palettes[..4], &self.bus.ppu.palettes[16..20]);
    
        for n in 0..512 {
            let x = n % 16;
            let y = n / 16;

            let tile = cartridge.get_tile(n);
            map.write_tile(x, y, tile.as_slice(), &(if n >= 256 { bg_palette } else { fg_palette }));
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
        self.bus.cartridge.as_ref().unwrap().sram.to_vec()
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
