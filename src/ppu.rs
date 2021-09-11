/**
 * 1 frame = 262 scanlines (1 pre-render, 240 visible, 20 vblank, 1 post-render).
 * 1 scanline = 341 PPU clock cycles (dots)
 * 1 PPU cycle = 1/3 CPU cycle = 1 pixel
 * 1 VBlank = 20 scanlines
 * 1 HBlank = 1 scanline
 * 
 * https://wiki.nesdev.com/w/index.php/PPU_frame_timing
 * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
 * https://www.reddit.com/r/EmuDev/comments/evu3u2/what_does_the_nes_ppu_actually_do/
 * http://wiki.nesdev.com/w/index.php/Mirroring
 * http://wiki.nesdev.com/w/index.php/PPU_nametables
 */

use log::{info, debug};
use crate::cpu::{Cpu, Interrupt};
use crate::cartridge::{Cartridge, Mirroring};

pub enum CtrlFlag {
    Nametable       = 0b0000_0011,  // Nametable select
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

pub enum LoopyRegister {
    CoarseX         = 0b00000000_00011111, // Coarse X offset (0-31)
    CoarseY         = 0b00000011_11100000, // Coarse Y offset (0-31)
    Nametable       = 0b00001100_00000000, // Nametable select
    FineY           = 0b01110000_00000000, // Fine Y offset (0-7)
    Unused          = 0b10000000_00000000,
}

pub const NAMETABLE_X_MASK: u16 = 0b00000100_00000000;
pub const NAMETABLE_Y_MASK: u16 = 0b00001000_00000000;

pub struct Ppu {
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oam_address: u8,
    pub oam_data: u8,
    pub oam_dma: u8,
    pub data: u8,
    pub nametables: [u8; 0x800], // Nametables. 2x1KiB (2 screen states)
    pub palettes: [u8; 0x20], // Palettes. 4x4 background + 4x4 sprite
    pub dot: u16,
    pub scanline: u16,
    pub framebuffer: Box<[u8; 256 * 240 * 4]>, // 512x480 -> 256x240 (32x30 = 960 tiles)
    write_latch: bool,
    read_buffer: u8,
    pub frame: usize,
    
    // Background
    pub cur_address: u16, // loopy_v
    pub tmp_address: u16, // loopy_t
    pattern_tile_id: u8,
    pattern_latch_hi: u8,
    pattern_latch_lo: u8,
    pattern_shift_hi: u16,
    pattern_shift_lo: u16,
    palette_latch: u8,
    palette_shift_hi: u16,
    palette_shift_lo: u16,

    // Scrolling
    scroll_x_fine: u8, // Fine X offset (0-7)

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
            data: 0,
            nametables: [0; 0x800],
            palettes: [0; 0x20],
            oam: [0; 256],
            dot: 0,
            scanline: 261, // start @ pre-render
            framebuffer: Box::new([0; 256 * 240 * 4]),
            write_latch: false,
            read_buffer: 0,
            frame: 0,
            cur_address: 0,
            tmp_address: 0,
            pattern_tile_id: 0,
            pattern_latch_hi: 0,
            pattern_latch_lo: 0,
            pattern_shift_hi: 0,
            pattern_shift_lo: 0,
            palette_latch: 0,
            palette_shift_hi: 0,
            palette_shift_lo: 0,
            scroll_x_fine: 0,
        }
    }

    /**
     * https://wiki.nesdev.com/w/index.php/PPU_rendering
     * https://wiki.nesdev.com/w/index.php/PPU_scrolling
     * https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
     */
    pub fn cycle (&mut self, cartridge: &Cartridge, cpu: &mut Cpu) {
        match self.scanline {
            0 ..= 239 | 261 => {
                // PPU busy fetching data, so PPU memory should not be accessed during this time (unless rendering is turned off - MaskFlags)
                match self.dot {
                    0 => {}, // Idle
                    // Draw pixels for scanline
                    1 ..= 256 | 321 ..= 336 => {
                        if (self.mask & MaskFlag::Background as u8) > 0 {
                            self.pattern_shift_hi <<= 1;
                            self.pattern_shift_lo <<= 1;
                            self.palette_shift_hi <<= 1;
                            self.palette_shift_lo <<= 1;
                        }

                        // Load data for next background tile. Each memory access takes 2 PPU cycles to complete, and 4 must be performed per tile
                        match self.dot % 8 {
                            // Horizontal increment (coarse because by-tile basis)
                            0 => {
                                if (self.mask & MaskFlag::Background as u8) > 0 {
                                    if (self.cur_address & LoopyRegister::CoarseX as u16) < 31 {
                                        // Increment coarse X
                                        self.cur_address += (1 << 0) as u16;
                                    } else {
                                        // Nametable edge, go to other horizontal nametable and reset coarse X to 0
                                        self.cur_address = (self.cur_address & !(LoopyRegister::CoarseX as u16)) | ((self.cur_address & LoopyRegister::Nametable as u16) ^ NAMETABLE_X_MASK);
                                    }
                                }
                            },
                            // Nametable byte
                            1 => {
                                // Load next pixels into the shifters
                                self.pattern_shift_hi = (self.pattern_shift_hi & 0b11111111_00000000) | self.pattern_latch_hi as u16;
                                self.pattern_shift_lo = (self.pattern_shift_lo & 0b11111111_00000000) | self.pattern_latch_lo as u16;
                                // Load next palette into the shifters
                                self.palette_shift_hi = (self.palette_shift_hi & 0b11111111_00000000) | if (self.palette_latch & 0b10 as u8) != 0 { 0b11111111 } else { 0b00000000 };
                                self.palette_shift_lo = (self.palette_shift_lo & 0b11111111_00000000) | if (self.palette_latch & 0b01 as u8) != 0 { 0b11111111 } else { 0b00000000 };

                                self.pattern_tile_id = self.read_vram(
                                    cartridge,
                                    0x2000
                                    | (self.cur_address & (LoopyRegister::Nametable as u16 | LoopyRegister::CoarseX as u16 | LoopyRegister::CoarseY as u16))
                                );
                            },
                            // Attribute table byte. Address: NN 1111 YYY XXX
                            // See https://github.com/OneLoneCoder/olcNES/blob/master/Part%20%234%20-%20PPU%20Backgrounds/olc2C02.cpp#L802
                            // and https://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching
                            3 => {
                                let byte = self.read_vram(
                                    cartridge,
                                    0x23C0
                                    | (self.cur_address & CtrlFlag::Nametable as u16)
                                    | (((self.cur_address & LoopyRegister::CoarseX as u16) >> 2) & 0b000111) // Top 3 bits of coarse X
                                    | (((self.cur_address & LoopyRegister::CoarseY as u16) >> 4) & 0b111000) // Top 3 bits of coarse Y
                                );
                                self.palette_latch = match ((self.cur_address & LoopyRegister::CoarseX as u16) % 4 / 2, ((self.cur_address & LoopyRegister::CoarseY as u16) >> 5) % 4 / 2) {
                                    (0, 0) => (byte >> 0), // Top left
                                    (1, 0) => (byte >> 2), // Top right
                                    (0, 1) => (byte >> 4), // Bottom left
                                    (1, 1) => (byte >> 6), // Bottom right
                                    _ => panic!("Not possible"),
                                } & 0b11;
                            },
                            // Pattern table tile low byte
                            5 => {
                                self.pattern_latch_lo = self.read_vram(
                                    cartridge,
                                    if (self.ctrl & CtrlFlag::Background as u8) > 0 { 0x1000 } else { 0 }
                                    | (self.pattern_tile_id as u16 * 16)
                                    | ((self.cur_address & LoopyRegister::FineY as u16) >> 12)
                                );
                            },
                            // Pattern table tile high byte
                            7 => {
                                self.pattern_latch_hi = self.read_vram(
                                    cartridge,
                                    if (self.ctrl & CtrlFlag::Background as u8) > 0 { 0x1000 } else { 0 }
                                    | (self.pattern_tile_id as u16 * 16)
                                    | ((self.cur_address & LoopyRegister::FineY as u16) >> 12)
                                    + 8
                                );
                            },
                            _ => {},
                        }

                        // Draw pixel on visible scanlines
                        if self.dot <= 256 && self.scanline != 261 && (self.mask & MaskFlag::Background as u8) > 0 {
                            let (hi, lo) = ((self.pattern_shift_hi >> 8) as u8 >> (7 - self.scroll_x_fine), (self.pattern_shift_lo >> 8) as u8 >> (7 - self.scroll_x_fine));
                            let pixel = (hi & 1) << 1 | (lo & 1);
                            
                            let (hi, lo) = ((self.palette_shift_hi >> 8) as u8 >> (7 - self.scroll_x_fine), (self.palette_shift_lo >> 8) as u8 >> (7 - self.scroll_x_fine));
                            let palette = (hi & 1) << 1 | (lo & 1);
                            
                            let color = self.read_vram(cartridge, 0x3F00 | (4 * palette as u16 + pixel as u16));
                            
                            let n = (self.dot - 1) as usize + (256 * self.scanline as usize);
                            let (r, g, b) = PALETTE[color as usize];
                            self.framebuffer[4 * n .. 4 * n + 4].copy_from_slice(&[r, g, b, 255]);
                        }

                        // Pre-render. Clear VBlank and Sprite 0 hit bits
                        if self.dot == 1 && self.scanline == 261 {
                            self.status &= !(StatusFlag::VBlank as u8 | StatusFlag::Hit as u8);
                        }
                        
                        // Vertical increment (fine because by-scanline basis)
                        if self.dot == 256 && (self.mask & MaskFlag::Background as u8) > 0 {
                            if ((self.cur_address & LoopyRegister::FineY as u16) >> 12) < 7 {
                                // Increment fine Y
                                self.cur_address += (1 << 12) as u16;
                            } else {
                                // Reset fine Y to 0
                                self.cur_address &= !(LoopyRegister::FineY as u16);

                                if ((self.cur_address & LoopyRegister::CoarseY as u16) >> 5) < 29 {
                                    // Increment coarse Y
                                    self.cur_address += (1 << 5) as u16;
                                } else {
                                    // Nametable edge, go to other vertical nametable and reset coarse Y to 0
                                    if ((self.cur_address & LoopyRegister::CoarseY as u16) >> 5) != 31 {
                                        self.cur_address ^= NAMETABLE_Y_MASK;
                                    }
                                    self.cur_address &= !(LoopyRegister::CoarseY as u16);
                                }
                            }
                        }
                    },
                    // Sprite
                    257 ..= 320 => {
                        if (self.mask & MaskFlag::Background as u8) > 0 {
                            // Load X info from temporary address
                            if self.dot == 257 {
                                let mask = LoopyRegister::CoarseX as u16 | NAMETABLE_X_MASK;
                                self.cur_address = (self.cur_address & !mask) | (self.tmp_address & mask);
                            }
    
                            // Load Y info from temporary address
                            if self.scanline == 261 && self.dot >= 280 && self.dot <= 304 {
                                let mask = LoopyRegister::CoarseY as u16 | LoopyRegister::FineY as u16 | NAMETABLE_Y_MASK;
                                self.cur_address = (self.cur_address & !mask) | (self.tmp_address & mask);
                            }
                        }
                    },
                    // Unused NT byte fetches
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
     * https://wiki.nesdev.com/w/index.php/PPU_scrolling
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

                self.read_buffer = cartridge.read_chr(self.cur_address);

                // Palette read
                if self.cur_address >= 0x3F00 {
                    dummy = self.read_buffer;
                }

                self.cur_address += if (self.ctrl & CtrlFlag::Increment as u8) > 0 { 32 } else { 1 };

                dummy
            },
            _ => panic!("Invalid I/O read @ {:#x}", address),
        }
    }

    /**
     * Write to registers
     * https://wiki.nesdev.com/w/index.php/PPU_scrolling
     */
    pub fn write (&mut self, cartridge: &mut Cartridge, address: u16, data: u8) {
        match (address % 8) + 0x2000 {
            // PPUCTRL
            0x2000 => {
                self.ctrl = data;
                self.tmp_address = (self.tmp_address & !(LoopyRegister::Nametable as u16)) | ((self.ctrl as u16 & CtrlFlag::Nametable as u16) << 10);
            },
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
                if self.write_latch {
                    // Y scroll
                    self.tmp_address = (self.tmp_address & !(LoopyRegister::FineY as u16)) | ((data as u16 & 0b0000_0111) << 12);
                    self.tmp_address = (self.tmp_address & !(LoopyRegister::CoarseY as u16)) | ((data as u16 >> 3) << 5);
                } else {
                    // X scroll
                    self.scroll_x_fine = data & 0b0000_0111;
                    self.tmp_address = (self.tmp_address & !(LoopyRegister::CoarseX as u16)) | (data >> 3) as u16;
                }

                self.write_latch = !self.write_latch;
            },
            // PPUADDR
            0x2006 => {
                if self.write_latch {
                    // Low byte
                    self.tmp_address = (self.tmp_address & 0b11111111_00000000) | (data as u16);
                    self.cur_address = self.tmp_address;
                } else {
                    // High byte
                    self.tmp_address = (self.tmp_address & 0b00000000_11111111) | ((data as u16) << 8);
                }

                self.write_latch = !self.write_latch;
            },
            // PPUDATA
            0x2007 => {
                self.write_vram(cartridge, self.cur_address, data);
                self.cur_address += if (self.ctrl & CtrlFlag::Increment as u8) > 0 { 32 } else { 1 };
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
            0x3F00 ..= 0x3FFF => {
                self.palettes[self.mirror_palette(address) as usize - 0x3F00]
            },
            // 0x4000 ..= 0xFFFF => self.read(nes, address - 0x4000),
            _ => panic!("Invalid read @ {:#x}", address),
        }
    }

    /**
     * Write to memory
     * https://wiki.nesdev.com/w/index.php/PPU_memory_map
     */
    fn write_vram (&mut self, cartridge: &mut Cartridge, address: u16, data: u8) {
        match address {
            // Pattern tables
            0x0000 ..= 0x1FFF => {
                cartridge.write_chr(address, data);
            },
            // Name tables
            0x2000 ..= 0x3EFF => {
                // info!("Write NT {:#x} (idx {:#x}) <- {:#x}", address, self.mirror(cartridge, address) as usize - 0x2000, data);
                self.nametables[self.mirror(cartridge, address) as usize - 0x2000] = data;
            },
            // Palettes
            0x3F00 ..= 0x3FFF => {
                // info!("Write Palette {:#x} <- {:#x}", address, data);
                self.palettes[self.mirror_palette(address) as usize - 0x3F00] = data;
            },
            _ => panic!("Invalid write @ {:#x}", address),
        }
    }

    /**
     * Mirror a nametable address
     * https://wiki.nesdev.com/w/index.php/Mirroring
     */
    pub fn mirror (&self, cartridge: &Cartridge, address: u16) -> u16 {
        match cartridge.mirroring {
            Mirroring::Horizontal => match address {
                0x2000 ..= 0x23FF => address,
                0x2400 ..= 0x27FF => address - 0x400,
                0x2800 ..= 0x2BFF => address - 0x400,
                0x2C00 ..= 0x2FFF => address - 0x800,
                _ => self.mirror(cartridge, address % 0x1000 + 0x2000),
            },
            Mirroring::Vertical => match address {
                0x2000 ..= 0x23FF => address,
                0x2400 ..= 0x27FF => address,
                0x2800 ..= 0x2BFF => address - 0x800,
                0x2C00 ..= 0x2FFF => address - 0x800,
                _ => self.mirror(cartridge, address % 0x1000 + 0x2000),        
            },
            Mirroring::FourScreen => unimplemented!("Four-screen mirroring not implemented"),
        }
    }

    /**
     * https://wiki.nesdev.com/w/index.php/PPU_palettes
     */
    pub fn mirror_palette (&self, address: u16) -> u16 {
        return match address % 4 {
            0 => address % 0x10,
            _ => address % 0x20,
        } + 0x3F00;
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

/**
 * http://wiki.nesdev.com/w/index.php/PPU_palettes
 * https://moddingwiki.shikadi.net/wiki/VGA_Palette
 * TODO .pal parser
 */
pub const PALETTE: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11),
];


#[test]
fn palette () {
    let ppu = Ppu::new();

    assert_eq!(ppu.mirror_palette(0x3F00), 0x3F00);
    assert_eq!(ppu.mirror_palette(0x3F04), 0x3F04);
    assert_eq!(ppu.mirror_palette(0x3F08), 0x3F08);
    assert_eq!(ppu.mirror_palette(0x3F0C), 0x3F0C);
    assert_eq!(ppu.mirror_palette(0x3F10), 0x3F00);
    assert_eq!(ppu.mirror_palette(0x3F14), 0x3F04);
    assert_eq!(ppu.mirror_palette(0x3F18), 0x3F08);
    assert_eq!(ppu.mirror_palette(0x3F1C), 0x3F0C);
    assert_eq!(ppu.mirror_palette(0x3F20), 0x3F00);
}
