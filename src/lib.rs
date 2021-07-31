extern crate console_error_panic_hook;

use wasm_bindgen::{prelude::*, Clamped};
use log::{info, trace};
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::ppu::CtrlFlag;
use crate::tilemap::Tilemap;

pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod bus;
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

    pub fn cycle (&mut self) -> usize {
        let frame = self.bus.ppu.frame;

        // Cycle until frame is rendered
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
        // let buffer32 = &self.bus.ppu.framebuffer;
        // let mut buffer8: Vec<u8> = Vec::with_capacity(buffer32.len() * 4);
        
        // for n in buffer32.iter() {
        //     buffer8.extend_from_slice(&n.to_be_bytes());
        // }

        // Clamped(buffer8)

        Clamped(
            unsafe {
                self.bus.ppu.framebuffer.align_to::<u8>().1.to_vec()
            }
        )
    }

    /**
     * https://wiki.nesdev.com/w/index.php/PPU_attribute_tables
     */
    pub fn get_nametable (&self, nth: u16) -> Clamped<Vec<u8>> {
        let cartridge = self.bus.cartridge.as_ref().unwrap();
        let start_address = self.bus.ppu.mirror(cartridge, 0x2000 + nth * 0x400);
        let mut map = Tilemap::new(32, 30);

        for n in 0 .. 960 {
            let x = n % 32;
            let y = n / 32;

            // Get tile
            let index = self.bus.ppu.read_vram(cartridge, start_address + n) as usize;
            let tile = cartridge.get_tile(index + if (self.bus.ppu.ctrl & CtrlFlag::Background as u8) > 0 { 256 } else { 0 });

            // Get byte from attribute table
            // let byte = self.bus.ppu.read(&self.bus.cartridge.as_ref().unwrap(), 0x23C0 | (addr & 0xC00) | ((addr >> 4) & 0x38) | ((addr >> 2) & 0x07));
            // let byte = self.bus.ppu.nametables[(nth as usize * 960) + (n as usize / 4)];

            // let (topleft, topright, bottomleft, bottomright) = (
            //     byte & 0b11,
            //     (byte >> 2) & 0b11,
            //     (byte >> 4) & 0b11,
            //     (byte >> 6) & 0b11,
            // );
            
            // Get palette
            // let quadrant = match (x % 2, y % 2) {
            //     (0, 0) => topleft,
            //     (1, 0) => topright,
            //     (0, 1) => bottomleft,
            //     (1, 1) => bottomright,
            //     _ => panic!("Not possible"),
            // };

            // let palette = &self.bus.ppu.palettes[4 * quadrant as usize .. 4 * quadrant as usize + 4];
            let palette = &self.bus.ppu.palettes[..4]; // Use first palette

            // Draw tile
            map.write_tile(x as usize, y as usize, tile.as_slice(), palette);
        }

        Clamped(map.buffer)
    }

    pub fn get_nametable_ram (&self) -> Vec<u8> {
        self.bus.ppu.nametables.to_vec()
    }

    /**
     * Get the contents of the CHR-ROM pattern tables.
     * Pattern tables contain background graphics (right) and sprite graphics (left)
     * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
     */
    pub fn get_pattern_tables (&self) -> Clamped<Vec<u8>> {
        let cartridge = self.bus.cartridge.as_ref().unwrap();
        let mut map = Tilemap::new(16, 32);
        let palette = [0x0D, 0x00, 0x10, 0x20]; // Greyscale palette
    
        for n in 0..512 {
            let x = n % 16;
            let y = n / 16;

            let tile = cartridge.get_tile(n);
            map.write_tile(x, y, tile.as_slice(), &palette);
        }
    
        Clamped(map.buffer)
    }

    /**
     * Get the palettes in use
     */
    pub fn get_palettes (&self) -> Clamped<Vec<u8>> {
        let mut map = Tilemap::new(16, 2);

        for n in 0..8 {
            let palette = &self.bus.ppu.palettes[n * 4..n * 4 + 4];
            let x = n * 4 % 16;
            let y = n * 4 / 16;
             
            for index in 0..4 {
                let tile = vec![index as u8; 8 * 8];
                map.write_tile(x + index, y, tile.as_slice(), palette);
            }
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

    // pub fn get_ram (&self) -> Vec<u8> {
    //     self.bus.wram.to_vec()
    // }
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
