/**
 * 1 frame = 262 scanlines (1 pre-render, 240 visible, 20 vblank, 1 post-render).
 * 1 scanline = 341 PPU clock cycles (dots)
 * 1 PPU cycle = 1/3 CPU cycle = 1 pixel
 * 1 VBlank = 20 scanlines
 * 1 HBlank = 1 scanline
 * 
 * https://wiki.nesdev.com/w/index.php/PPU_frame_timing
 * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
 * https://wiki.nesdev.com/w/index.php/PPU_scrolling
 * https://www.reddit.com/r/EmuDev/comments/evu3u2/what_does_the_nes_ppu_actually_do/
 * http://wiki.nesdev.com/w/index.php/Mirroring
 * http://wiki.nesdev.com/w/index.php/PPU_nametables
 */

use log::{info, debug};
use crate::cpu::{Cpu, interrupt::Interrupt};
use crate::cartridge::{Cartridge, Mirroring};

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
    Hit             = 0b0100_0000,  // Sprite 0 hit
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
    pub nametables: [u8; 0x800], // Nametables. 2x1KiB (2 screen states)
    pub palettes: [u8; 0x20], // Palettes. 4x4 background + 4x4 sprite
    pub dot: u16,
    pub scanline: u16,
    pub framebuffer: Box<[u32; 256 * 240]>, // 512x480 -> 256x240 (32x30 = 960 tiles)
    write_latch: bool,
    read_buffer: u8,
    pub frame: usize,
    
    // Background
    pub address: u16,
    pattern_latch_hi: u8,
    pattern_latch_lo: u8,
    pattern_shift_hi: u16,
    pattern_shift_lo: u16,
    // palette_shift_hi: u8,
    // palette_shift_lo: u8,


    // Sprites
    pub oam: [u8; 256], // Sprite RAM: 64 * 4 bytes (Y, tile #, attribute, X)
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            ctrl: 0b0000_0000,
            mask: 0,
            status: StatusFlag::VBlank as u8,
            oam_address: 0,
            oam_data: 0,
            oam_dma: 0,
            scroll: 0,
            data: 0,
            nametables: [0; 0x800],
            palettes: [0; 0x20],
            oam: [0; 256],
            dot: 0,
            scanline: 261, // start @ pre-render
            framebuffer: Box::new([0b00000000_00000000_00000000_11111111; 256 * 240]),
            write_latch: false,
            read_buffer: 0,
            frame: 0,
            address: 0,
            pattern_latch_hi: 0,
            pattern_latch_lo: 0,
            pattern_shift_hi: 0,
            pattern_shift_lo: 0,
            // palette_shift_hi: 0,
            // palette_shift_lo: 0,        
        }
    }

    /**
     * https://wiki.nesdev.com/w/index.php/PPU_rendering
     * https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
     */
    pub fn cycle (&mut self, cartridge: &Cartridge, cpu: &mut Cpu) {
        match self.scanline {
            // Fill shift registers with data for next scanline
            0 ..= 239 | 261 => {
                // PPU busy fetching data, so PPU memory should not be accessed during this time (unless rendering is turned off - MaskFlags)
                match self.dot {
                    0 => {}, // Idle
                    // Draw pixels for scanline
                    1 ..= 256 | 321 ..= 336 => {
                        // // let offset = (self.ctrl & CtrlFlag::Nametable as u8) as u16 * 0x400;
                        // // let tile = self.read(bus, self.address + offset);
                        // //  bus.cartridge.unwrap().
                        // if self.mask & MaskFlag::Background as u8 != 0 {
                        //     self.pattern_shift_hi <<= 1;
                        //     self.pattern_shift_lo <<= 1;
                        // }

                        // Pre-render. Clear VBlank and Sprite 0 hit bits
                        if self.dot == 1 && self.scanline == 261 {
                            self.status &= !(StatusFlag::VBlank as u8 | StatusFlag::Hit as u8);
                        }

                        // // Draw pixel on visible scanlines
                        // if self.dot <= 256 && self.scanline != 261 {
                        //     // self.framebuffer[self.scanline as usize * 256 + self.dot as usize] = 0b11111111_00000000_00000000_11111111;
                        //     // self.framebuffer[self.scanline as usize * 256 + self.dot as usize] = self.pattern_shift_hi | self.pattern_shift_lo;
                        // }

                        // // Load data into background shifters
                        // // Each memory access takes 2 PPU cycles to complete, and 4 must be performed per tile
                        // match self.dot % 8 {
                        //     // vert(v) inc.
                        //     0 => {},
                        //     // NT byte
                        //     1 => {
                        //         // Reload shifters
                        //         self.pattern_shift_hi = (self.pattern_shift_hi & 0b11111111_00000000) | self.pattern_latch_hi as u16;
                        //         self.pattern_shift_lo = (self.pattern_shift_lo & 0b11111111_00000000) | self.pattern_latch_lo as u16;
                        //     },
                        //     // AT byte
                        //     3 => {},
                        //     // PT tile byte lo
                        //     5 => {
                        //         self.pattern_latch_lo = cartridge.chr_rom[self.dot as usize / 8];
                        //     },
                        //     // PT tile byte hi
                        //     7 => {
                        //         self.pattern_latch_hi = cartridge.chr_rom[self.dot as usize / 8 + 8];
                        //     },
                        //     _ => {},
                        // }
                    },
                    257 ..= 320 => {
                        // Sprite
                        // hori(v) = hori(t)
                    },
                    337 ..= 340 => {},
                    _ => {},
                }
            },
            240 => {}, // Post-render
            // V-Blank and NMI
            241 => {
                if self.dot == 1 {
                    // info!("Scanline 241. Setting VBlank");
                    self.status |= StatusFlag::VBlank as u8;
                    // self.status |= StatusFlag::Hit as u8; // Test 0 sprite hit TODO REMOVE
                    // Trigger NMI if enabled
                    if self.ctrl & CtrlFlag::Nmi as u8 > 0 {
                        cpu.interrupt_request(Interrupt::NMI);
                    }
                }
            },
            // The PPU makes no memory accesses during these scanlines, so PPU memory can be freely accessed by the program.
            242 ..= 260 => {},
            _ => {}
        }

        self.dot += 1;

        if self.dot > 340 {
            // info!("End of line {}", self.scanline);
            self.dot = 0;
            self.scanline += 1;

            if self.scanline > 261 {
                self.scanline = 0;
                self.frame += 1;

                // // Skip first dot on odd frames
                // if self.frame % 2 == 1 {
                //     self.dot += 1;
                // }
            }            
        }
    }

    /**
     * Read registers
     */
    pub fn read (&mut self, cartridge: &Cartridge, address: u16) -> u8 {
        match (address % 8) + 0x2000 {
            // PPUSTATUS
            0x2002 => {
                // Residual data on bottom 5 bits
                let status = (self.status & 0b1110_0000) | (self.read_buffer & 0b0001_1111);
                // Clear vblank bit on read
                self.status &= !(StatusFlag::VBlank as u8);
                self.write_latch = false;
                status
            },
            // OAMDATA
            0x2004 => self.oam_data,
            // PPUDATA
            0x2007 => {
                let mut dummy = self.read_buffer;

                self.read_buffer = cartridge.read_chr(self.address);

                // Palette read
                if self.address >= 0x3F00 {
                    dummy = self.read_buffer;
                }

                self.address += if (self.ctrl & CtrlFlag::Increment as u8) > 0 { 32 } else { 1 };

                dummy
            },
            _ => panic!("Invalid I/O read @ {:#x}", address),
        }
    }

    /**
     * Write to registers
     */
    pub fn write (&mut self, cartridge: &Cartridge, address: u16, data: u8) {
        match (address % 8) + 0x2000 {
            // PPUCTRL
            0x2000 => { self.ctrl = data; },
            // PPUMASK
            0x2001 => { self.mask = data; },
            // OAMADDR
            0x2003 => {
                // debug!("Write OAMADDR");
                self.oam_address = data;
            },
            // OAMDATA
            0x2004 => {
                // debug!("Write OAMDATA");
                self.oam_data = data;
            },
            // PPUSCROLL
            0x2005 => {
                self.scroll = data;
                self.write_latch = !self.write_latch;
            },
            // PPUADDR
            0x2006 => {
                if self.write_latch {
                    // Low byte
                    self.address = (self.address & 0b11111111_00000000) | (data as u16);
                } else {
                    // High byte
                    self.address = (self.address & 0b00000000_11111111) | ((data as u16) << 8);
                }
                
                // debug!("Write address {:#x}, PPU address is now {:#x}", data, self.address);
        
                self.write_latch = !self.write_latch;
            },
            // PPUDATA
            0x2007 => {
                self.write_vram(cartridge, self.address, data);
                self.address += if (self.ctrl & CtrlFlag::Increment as u8) > 0 { 32 } else { 1 };
            },
            _ => panic!("Invalid I/O write @ {:#x}", address),
        }
    }

    /**
     * Read memory
     * https://wiki.nesdev.com/w/index.php/PPU_memory_map
     */
    pub fn read_vram (&self, cartridge: &Cartridge, address: u16) -> u8 {
        match address {
            // Pattern tables in cartridge CHR ROM/RAM
            0x0000 ..= 0x1FFF => {
                // TODO specific addressing
                cartridge.read_chr(address)
            },
            // Name tables (1024 bytes each), containing tiles (32x30 = 960 bytes) & the attribute table (64 bytes)
            0x2000 ..= 0x3EFF => {
                self.nametables[self.mirror(cartridge, address) as usize - 0x2000]
            },
            // Palette
            0x3F00 ..= 0x3FFF => self.palettes[(address as usize - 0x3F00) % 0x20],
            // 0x4000 ..= 0xFFFF => self.read(nes, address - 0x4000),
            _ => panic!("Invalid read @ {:#x}", address),
        }
    }

    /**
     * Write to memory
     */
    fn write_vram (&mut self, cartridge: &Cartridge, address: u16, data: u8) {
        match address {
            // Name tables
            0x2000 ..= 0x3EFF => {
                // info!("Write NT {:#x} (idx {:#x}) <- {:#x}", address, self.mirror(cartridge, address) as usize - 0x2000, data);
                self.nametables[self.mirror(cartridge, address) as usize - 0x2000] = data;
            },
            // Palettes
            0x3F00 ..= 0x3FFF => {
                // info!("Write Palette {:#x} <- {:#x}", address, data);
                self.palettes[(address as usize - 0x3F00) % 0x20] = data;
            },
            _ => panic!("Invalid write @ {:#x}", address),
        }
    }

    /**
     * Mirror a nametable address
     */
    pub fn mirror (&self, cartridge: &Cartridge, address: u16) -> u16 {
        let mirroring = cartridge.get_mirroring();

        match mirroring {
            Mirroring::Horizontal => match address {
                0x2000 ..= 0x23FF => address,
                0x2400 ..= 0x27FF => address - 0x400,
                0x2800 ..= 0x2BFF => address - 0x400,
                0x2C00 ..= 0x2FFF => address - 0x800,
                _ => self.mirror(cartridge, address % 0x3000 + 0x2000),
            },
            Mirroring::Vertical => match address {
                0x2000 ..= 0x23FF => address,
                0x2400 ..= 0x27FF => address,
                0x2800 ..= 0x2BFF => address - 0x800,
                0x2C00 ..= 0x2FFF => address - 0x800,
                _ => self.mirror(cartridge, address % 0x3000 + 0x2000),        
            },
            Mirroring::FourScreen => unimplemented!("Four-screen mirroring not implemented"),
        }
    }

    // fn read_oam (&self, memory: &Memory) -> [u8; 256] {
    //     // self.vram[address % 0x4000];
    // }
}

impl Default for Ppu {
    fn default () -> Self {
        Ppu::new()
    }
}
