/**
 * 1 frame = 262 scanlines (1 pre-render, 240 visible, 20 vblank, 1 post-render).
 * 1 scanline = 341 PPU clock cycles (dots)
 * 1 PPU cycle = 1/3 CPU cycle = 1 pixel
 * 1 VBlank = 20 scanlines
 * 1 HBlank = 1 scanline
 * https://wiki.nesdev.com/w/index.php/PPU_rendering
 * https://wiki.nesdev.com/w/index.php/PPU_frame_timing
 * https://wiki.nesdev.com/w/index.php/PPU_memory_map
 * https://wiki.nesdev.com/w/index.php/PPU_attribute_tables
 * https://wiki.nesdev.com/w/index.php/PPU_scrolling
 * https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
 * https://www.reddit.com/r/EmuDev/comments/evu3u2/what_does_the_nes_ppu_actually_do/
 * http://wiki.nesdev.com/w/index.php/Mirroring
 * http://wiki.nesdev.com/w/index.php/PPU_nametables
 */

use crate::bus::Bus;
use crate::cartridge::Mirroring;

pub mod palette;

pub enum CtrlFlag {
    Nametable       = 0b0000_0011,  // Nametable address
    Increment       = 0b0000_0100,  // VRAM address increment per read or write: -32 or +1
    Sprite          = 0b0000_1000,  // Sprite pattern table address for 8x8 sprites
    Background      = 0b0001_0000,  // Background pattern table address
    Height          = 0b0010_0000,  // Sprite size (8x16 or 8x8)
    Master          = 0b0100_0000,  // PPU master/slave select
    Nmi             = 0b1000_0000,  // Enable NMI on V-Blank
}

pub enum MaskFlag {
    Greyscale       = 0b0000_0001,  // Greyscale
    BackgroundLeft  = 0b0000_0010,  // Enable background on leftmost 8 pixels of screen
    SpritesLeft     = 0b0000_0100,  // Enable sprites on leftmost 8 pixels of screen
    Background      = 0b0000_1000,  // Enable background
    Sprites         = 0b0001_0000,  // Enable sprites
    Red             = 0b0010_0000,  // Emphasize red
    Green           = 0b0100_0000,  // Emphasize green
    Blue            = 0b1000_0000,  // Emphasize blue
}

pub enum StatusFlag {
    Overflow        = 0b0010_0000,  // Sprite overflow
    Hit             = 0b0100_0000,  // Sprite hit
    VBlank          = 0b1000_0000,  // Vertical blank
}

pub struct Ppu {
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oam_address: u8,
    pub oam_data: u8,
    pub oam_dma: u8,
    pub scroll: u8,
    pub data: u8,
    pub vram: [u8; 0x2000], // Nametables. 2x1KiB (2 screen states)
    pub dot: u16,
    pub scanline: u16,
    // framebuffer: [u8; 256 * 240], // 512x480 -> 256x240
    
    // Background
    pub address: u16,
    // pattern_shift_hi: u16,
    // pattern_shift_lo: u16,
    // palette_shift_hi: u8,
    // palette_shift_lo: u8,

    // Sprites
    pub oam: [u8; 256], // Sprite RAM: 64 * 4 bytes (Y, tile #, attribute, X)
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_address: 0,
            oam_data: 0,
            oam_dma: 0,
            scroll: 0,
            data: 0,
            vram: [0; 0x2000],
            oam: [0; 256],
            dot: 0,
            scanline: 0,
            // framebuffer: [0; 256 * 240],
            address: 0,
            // pattern_shift_hi: 0,
            // pattern_shift_lo: 0,
            // palette_shift_hi: 0,
            // palette_shift_lo: 0,        
        }
    }

    pub fn cycle (&self, _bus: &Bus) {
        match self.scanline {
            0 ..= 239 => {
                // PPU busy fetching data, so PPU memory should not be accessed during this time (unless rendering is turned off - MaskFlags)
                match self.dot {
                    0 => {}, // Idle
                    1 ..= 256 => {
                        // let offset = (self.ctrl & CtrlFlag::Nametable as u8) as u16 * 0x400;
                        // let tile = self.read(bus, self.address + offset);
                        //  bus.cartridge.unwrap().
                        // vert(v) inc.
                    },
                    257 ..= 320 => {
                        // hori(v) = hori(t)
                    },
                    321 ..= 336 => {},
                    337 ..= 340 => {},
                    _ => {},
                }
            },
            240 => {}, // Post-render
            241 ..= 260 => {}, // V-Blank, NMI interrupt
            261 => {} // Pre-render
            _ => {}
        }
    }

    pub fn read (&self, bus: &Bus, address: u16) -> u8 {
        match address {
            // Pattern tables in cartridge CHR ROM/RAM
            0x0000 ..= 0x1FFF => {
                // TODO specific addressing
                bus.cartridge.as_ref().unwrap().read_chr(address)
            },
            // Name tables (1024 bytes) & their associated attribute tables (64 bytes)
            0x2000 ..= 0x3EFF => {
                let mirroring = bus.cartridge.as_ref().unwrap().get_mirroring();
                let n = address as usize - 0x2000;

                match address {
                    // Top left
                    0x2000 ..= 0x23FF => {
                        match mirroring {
                            Mirroring::Horizontal => self.vram[n],
                            Mirroring::Vertical => self.vram[n],
                            Mirroring::FourScreen => self.vram[n],
                        }
                    },
                    // Top right
                    0x2400 ..= 0x27FF => {
                        match mirroring {
                            Mirroring::Horizontal => self.vram[n - 0x400],
                            Mirroring::Vertical => self.vram[n],
                            Mirroring::FourScreen => self.vram[n],
                        }
                    },
                    // Bottom left
                    0x2800 ..= 0x2BFF => {
                        match mirroring {
                            Mirroring::Horizontal => self.vram[n - 0x400],
                            Mirroring::Vertical => self.vram[n - 0x800],
                            Mirroring::FourScreen => unimplemented!("Four-screen mirroring not implemented"),
                        }
                    },
                    // Bottom right
                    0x2C00 ..= 0x2FFF => {
                        match mirroring {
                            Mirroring::Horizontal => self.vram[n - 0x800],
                            Mirroring::Vertical => self.vram[n - 0x800],
                            Mirroring::FourScreen => unimplemented!("Four-screen mirroring not implemented"),
                        }
                    },
                    _ => self.read(bus, address % 0x3000 + 0x2000),
                }
            },
            0x3F00 ..= 0x3FFF => 0, // Palette
            // 0x4000 ..= 0xFFFF => self.read(nes, address - 0x4000),
            _ => panic!("Invalid address"),
        }
    }

    pub fn write (&mut self, _bus: &mut Bus, _address: u16, _data: u8) {

    }

    // fn read_oam (&self, memory: &Memory) -> [u8; 256] {
    //     // self.vram[address % 0x4000];
    // }

    // pub fn write_address () {}
}

impl Default for Ppu {
    fn default () -> Self {
        Ppu::new()
    }
}
