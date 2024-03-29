/**
 * 1 frame = 262 scanlines (1 pre-render, 240 visible, 1 post-render, 20 vblank).
 * 1 scanline = 341 PPU clock cycles (dots)
 * 1 PPU cycle = 1/3 CPU cycle = 1 pixel
 * 1 VBlank = 20 scanlines
 * 1 HBlank = 1 scanline
 * 
 * https://wiki.nesdev.com/w/index.php/PPU_frame_timing
 * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
 * http://wiki.nesdev.com/w/index.php/Mirroring
 * http://wiki.nesdev.com/w/index.php/PPU_nametables
 */

use wasm_bindgen::prelude::*;
use crate::{
    cpu::{Cpu, Interrupt},
    cartridge::{Cartridge, Mirroring},
    ppu::PALETTE,
    clock::ClockDivider,
};

#[wasm_bindgen(js_name = PpuCtrlFlag)]
pub enum CtrlFlag {
    Nametable       = 0b0000_0011,  // Nametable select
    Increment       = 0b0000_0100,  // VRAM address increment per read or write: -32 or +1
    Sprite          = 0b0000_1000,  // Sprite pattern table address for 8x8 sprites
    Background      = 0b0001_0000,  // Background pattern table address
    SpriteHeight    = 0b0010_0000,  // Sprite size (8x16 or 8x8)
    _Master         = 0b0100_0000,  // PPU master/slave select
    Nmi             = 0b1000_0000,  // Enable NMI on V-Blank
}

#[wasm_bindgen(js_name = PpuMaskFlag)]
pub enum MaskFlag {
    _Greyscale      = 0b0000_0001,  // Greyscale
    BackgroundLeft  = 0b0000_0010,  // Enable background on leftmost 8 pixels of screen
    SpritesLeft     = 0b0000_0100,  // Enable sprites on leftmost 8 pixels of screen
    Background      = 0b0000_1000,  // Enable background
    Foreground      = 0b0001_0000,  // Enable sprites
    _Red            = 0b0010_0000,  // Emphasize red
    _Green          = 0b0100_0000,  // Emphasize green
    _Blue           = 0b1000_0000,  // Emphasize blue
}

#[wasm_bindgen(js_name = PpuStatusFlag)]
pub enum StatusFlag {
    SpriteOverflow  = 0b0010_0000,  // Sprite overflow
    Hit             = 0b0100_0000,  // Sprite 0 hit
    VBlank          = 0b1000_0000,  // Vertical blank
}

#[wasm_bindgen]
pub enum SpriteAttribute {
    Palette         = 0b0000_0011,
    Zero            = 0b0001_1100, // Normally unused, serves to keep track of sprite zero
    Priority        = 0b0010_0000,
    FlipHorizontal  = 0b0100_0000,
    FlipVertical    = 0b1000_0000,
}

enum LoopyRegister {
    CoarseX         = 0b00000000_00011111, // Coarse X offset (0-31)
    CoarseY         = 0b00000011_11100000, // Coarse Y offset (0-31)
    Nametable       = 0b00001100_00000000, // Nametable select
    FineY           = 0b01110000_00000000, // Fine Y offset (0-7)
    _Unused         = 0b10000000_00000000,
}

const NAMETABLE_X_MASK: u16 = 0b00000100_00000000;
const NAMETABLE_Y_MASK: u16 = 0b00001000_00000000;

#[derive(Clone)]
pub struct Ppu {
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub nametables: Vec<u8>, // Nametables. 2x1KiB (2 screen states)
    pub palettes: Vec<u8>, // Palettes. 4x4 background + 4x4 sprite
    pub dot: u16,
    pub scanline: u16,
    pub framebuffer: Vec<u8>, // 512x480 -> 256x240 (32x30 = 960 tiles)
    pub frame: usize,
    read_buffer: u8,
    pub clock: ClockDivider,

    // Background
    cur_address: u16, // loopy_v
    tmp_address: u16, // loopy_t, top-left corner
    scroll_x_fine: u8, // Fine X offset (0-7)
    write_latch: bool,
    pattern_tile_id: u8,
    pattern_latch_hi: u8,
    pattern_latch_lo: u8,
    pattern_shift_hi: u16,
    pattern_shift_lo: u16,
    palette_latch: u8,
    palette_shift_hi: u16,
    palette_shift_lo: u16,
    
    // Sprites
    pub oam: Vec<u8>, // Sprite RAM: 64 * 4 bytes (Y, tile #, attribute, X)
    oam_index: u8,
    oam_index_overflowed: bool,
    oam_secondary: [u8; 32], // Sprites to be rendered on next scanline (max 8): 8 * 4 bytes
    oam_secondary_index: u8,
    oam_address: u8,
    sprite_shift_hi: [u8; 8],
    sprite_shift_lo: [u8; 8],
    sprite_attributes: [u8; 8],
    sprite_offsets: [u8; 8],
}

impl Ppu {
    pub fn new () -> Ppu {
        Ppu {
            ctrl: 0,
            mask: 0,
            status: StatusFlag::VBlank as u8,
            nametables: vec![0; 0x800],
            palettes: vec![0; 32],
            dot: 0,
            scanline: 0,
            framebuffer: vec![0; 256 * 240 * 4],
            frame: 0,
            read_buffer: 0,
            clock: ClockDivider::new(crate::clock::CLOCK_PPU_NTSC),
            cur_address: 0,
            tmp_address: 0,
            scroll_x_fine: 0,
            write_latch: false,
            pattern_tile_id: 0,
            pattern_latch_hi: 0,
            pattern_latch_lo: 0,
            pattern_shift_hi: 0,
            pattern_shift_lo: 0,
            palette_latch: 0,
            palette_shift_hi: 0,
            palette_shift_lo: 0,
            oam: vec![0; 256],
            oam_index: 0,
            oam_index_overflowed: false,
            oam_secondary: [0; 32],
            oam_secondary_index: 0,
            oam_address: 0,
            sprite_shift_hi: [0; 8],
            sprite_shift_lo: [0; 8],
            sprite_attributes: [0; 8],
            sprite_offsets: [0; 8],      
        }
    }

    pub fn tick (&mut self, time: f64, cartridge: &Cartridge, cpu: &mut Cpu) {
        if self.clock.tick(time) {
            self.cycle(cartridge, cpu);
        }
    }

    /**
     * https://wiki.nesdev.com/w/index.php/PPU_rendering
     * https://wiki.nesdev.com/w/index.php/PPU_scrolling
     * https://wiki.nesdev.com/w/index.php/PPU_OAM
     * https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
     */
    pub fn cycle (&mut self, cartridge: &Cartridge, cpu: &mut Cpu) {
        match self.scanline {
            0 ..= 239 | 261 => {
                // PPU busy fetching data, so PPU memory should not be accessed during this time (unless rendering is turned off - MaskFlags)
                match self.dot {
                    0 => {}, // Idle
                    1 ..= 256 | 321 ..= 336 => {
                        self.background_shift();
                        self.background_fetch(cartridge);

                        // Visible scanlines
                        if self.scanline != 261 {
                            self.sprite_evaluation(cartridge);

                            // Draw pixel on visible dots
                            if self.dot <= 256 {
                                self.draw_pixel(cartridge);
                            }
                        } else if self.dot == 1 {
                            // Pre-render, end of VBlank
                            self.vblank_end();
                        }

                        if self.dot == 256 {
                            self.y_increment();
                        }
                    },
                    257 ..= 320 => {
                        self.sprite_fetch(cartridge);

                        if self.mask & (MaskFlag::Background as u8 | MaskFlag::Foreground as u8) > 0 {
                            match (self.scanline, self.dot) {
                                (_, 257) => {
                                    self.x_reload();
                                },
                                (261, 280 ..= 304) => {
                                    self.y_reload();
                                },
                                _ => {},
                            }
                        }
                    },
                    // Unused NT byte fetches
                    337 ..= 340 => {},
                    _ => {},
                }
            },
            240 => {}, // Post-render
            241 => {
                if self.dot == 1 {
                    self.vblank_start(cpu);
                }
            },
            // The PPU makes no memory accesses during these scanlines, so PPU memory can be freely accessed by the program.
            242 ..= 260 => {},
            _ => {}
        }

        self.cycle_increment();
    }
    
    fn cycle_increment (&mut self) {
        self.dot += 1;
    
        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;
    
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame += 1;
    
                // Skip first dot on odd frames to make them 1 PPU clock shorter than normal
                if (self.mask & MaskFlag::Background as u8) > 0 && self.frame % 2 == 1 {
                    self.dot += 1;
                }
            }            
        }
    }

    /**
     * Shift background registers
     */
    fn background_shift (&mut self) {
        if (self.mask & MaskFlag::Background as u8) > 0 {
            self.pattern_shift_hi <<= 1;
            self.pattern_shift_lo <<= 1;
            self.palette_shift_hi <<= 1;
            self.palette_shift_lo <<= 1;
        }
    }

    /**
     * Load data for next background tile. Each memory access takes 2 PPU cycles to complete, and 4 must be performed per tile
     */
    fn background_fetch (&mut self, cartridge: &Cartridge) {
        match self.dot % 8 {
            0 => {
                self.x_increment();
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
            // https://wiki.nesdev.com/w/index.php/PPU_attribute_tables
            // See https://github.com/OneLoneCoder/olcNES/blob/master/Part%20%234%20-%20PPU%20Backgrounds/olc2C02.cpp#L802
            // and https://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching for the computed address
            3 => {
                let byte = self.read_vram(
                    cartridge,
                    0x23C0
                    | (self.cur_address & LoopyRegister::Nametable as u16)
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
    }

    /**
     * Prepare secondary OAM for next scanline
     * https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
     */
    fn sprite_evaluation (&mut self, _cartridge: &Cartridge) {
        match self.dot {
            // Clear secondary OAM
            1 ..= 64 => {
                if self.dot == 1 {
                    self.oam_secondary_index = 0;
                } else if self.dot % 2 == 0 {
                    self.oam_secondary[self.oam_secondary_index as usize] = 0xFF;
                    self.oam_secondary_index += 1;
                }
            },
            // Perform sprite evaluation
            65 ..= 256 => {
                if self.dot == 65 {
                    self.oam_index = 0;
                    self.oam_secondary_index = 0;
                    self.oam_index_overflowed = false;
                } else if self.dot % 2 == 0 {
                    let sprite_y = self.oam[self.oam_index as usize] as u16;
                    let sprite_height = if (self.ctrl & CtrlFlag::SpriteHeight as u8) > 0 { 16 } else { 8 };

                    // Sprite overlaps with current scanline
                    if !self.oam_index_overflowed && (self.scanline >= sprite_y) && (sprite_y + sprite_height > self.scanline) {
                        if (self.oam_secondary_index / 4) < 8 {
                            self.oam_secondary[self.oam_secondary_index as usize .. self.oam_secondary_index as usize + 4]
                                .copy_from_slice(&self.oam[self.oam_index as usize .. self.oam_index as usize + 4]);

                            // Store sprite zero hit info in the sprite's attribute
                            if self.oam_index == 0 {
                                self.oam_secondary[self.oam_secondary_index as usize + 2] |= SpriteAttribute::Zero as u8;
                            } else {
                                self.oam_secondary[self.oam_secondary_index as usize + 2] &= !(SpriteAttribute::Zero as u8);
                            }

                            self.oam_secondary_index += 4;
                        } else if (self.status & StatusFlag::SpriteOverflow as u8) == 0 {
                            self.status |= StatusFlag::SpriteOverflow as u8;
                        } else {
                            // Sprite overflow bug
                            let (index, overflow) = self.oam_index.overflowing_add(1);
                            self.oam_index = index;
                            self.oam_index_overflowed = self.oam_index_overflowed || overflow;
                        }
                    }

                    let (index, overflow) = self.oam_index.overflowing_add(4);
                    self.oam_index = index;
                    self.oam_index_overflowed = self.oam_index_overflowed || overflow;
                }
            },
            _ => {},
        }
    }

    /**
     * Sprite fetches. Garbage bytes are ignored
     */
    fn sprite_fetch (&mut self, cartridge: &Cartridge) {
        match (self.dot - 257) % 8 {
            cycle @ (4 | 6) => {
                let index = ((self.dot - 257) / 8) as usize;
                let (sprite_y, sprite_id, sprite_attributes, sprite_x) = (
                    self.oam_secondary[index * 4 + 0] as u16,
                    self.oam_secondary[index * 4 + 1] as u16,
                    self.oam_secondary[index * 4 + 2],
                    self.oam_secondary[index * 4 + 3],
                );

                if sprite_y != 0xFF {
                    let row = (self.scanline - sprite_y) % 8; // Take into account 16px high tiles
                    let address = if (self.ctrl & CtrlFlag::SpriteHeight as u8) > 0 {
                        let half = (self.scanline - sprite_y) / 8; // Either top (0) or bottom (1) half

                        (sprite_id & 0b0000_0001) * 0x1000
                        | ((sprite_id & 0b1111_1110) + half) * 16
                        | if (sprite_attributes & SpriteAttribute::FlipVertical as u8) > 0 { 7 - row } else { row }
                    } else {
                        (
                            if (self.ctrl & CtrlFlag::Sprite as u8) > 0 { 0x1000 } else { 0 }
                            | sprite_id * 16
                            | if (sprite_attributes & SpriteAttribute::FlipVertical as u8) > 0 { 7 - row } else { row }
                        )
                    };

                    let mut data = match cycle {
                        4 => self.read_vram(cartridge, address),
                        6 => self.read_vram(cartridge, address + 8),
                        _ => unreachable!(),
                    };

                    if (sprite_attributes & SpriteAttribute::FlipHorizontal as u8) > 0 {
                        data = data.reverse_bits();
                    }

                    match cycle {
                        4 => { self.sprite_shift_lo[index] = data; },
                        6 => { self.sprite_shift_hi[index] = data; },
                        _ => unreachable!(),
                    };

                    self.sprite_attributes[index] = sprite_attributes;
                    self.sprite_offsets[index] = sprite_x;
                }
            },
            _ => {},
        }
    }

    /**
     * Draw pixel at current location
     */
    fn draw_pixel (&mut self, cartridge: &Cartridge) {
        let (mut bg_pixel, mut bg_palette) = (0, 0);
        let (mut fg_pixel, mut fg_palette, mut fg_priority) = (0, 0, false);
        let mut sprite_zero_hit = false;

        if (self.mask & MaskFlag::Background as u8) > 0 {
            let (hi, lo) = ((self.pattern_shift_hi >> 8) as u8 >> (7 - self.scroll_x_fine), (self.pattern_shift_lo >> 8) as u8 >> (7 - self.scroll_x_fine));
            bg_pixel = (hi & 1) << 1 | (lo & 1);
            
            let (hi, lo) = ((self.palette_shift_hi >> 8) as u8 >> (7 - self.scroll_x_fine), (self.palette_shift_lo >> 8) as u8 >> (7 - self.scroll_x_fine));
            bg_palette = (hi & 1) << 1 | (lo & 1);
        }

        if (self.mask & MaskFlag::Foreground as u8) > 0 {
            let mut index = 0;

            // Simple loop is more performant than range iterator
            while index < 8 {
                if self.sprite_offsets[index] > 0 {
                    self.sprite_offsets[index] -= 1;
                } else {
                    self.sprite_shift_hi[index] <<= 1;
                    self.sprite_shift_lo[index] <<= 1;
                }

                // Loop will end at first non-transparent sprite pixel
                if self.sprite_offsets[index] == 0 && fg_pixel == 0 {
                    let (hi, lo) = (self.sprite_shift_hi[index] >> 7, self.sprite_shift_lo[index] >> 7);
                    fg_pixel = (hi & 1) << 1 | (lo & 1);
                    fg_palette = (self.sprite_attributes[index] & SpriteAttribute::Palette as u8) + 4;
                    fg_priority = (self.sprite_attributes[index] & SpriteAttribute::Priority as u8) == 0;

                    if (self.sprite_attributes[index] & SpriteAttribute::Zero as u8) > 0 {
                        sprite_zero_hit = true;
                    }
                }

                index += 1;
            }
        }

        let (pixel, palette) = match (bg_pixel, fg_pixel) {
            (0, 0) => (0, 0),
            (0, _) => (fg_pixel, fg_palette),
            (_, 0) => (bg_pixel, bg_palette),
            (_, _) => {
                // Sprite zero hit
                if sprite_zero_hit && self.dot < 255 && (self.dot > 8 || (self.mask & (MaskFlag::BackgroundLeft as u8 | MaskFlag::SpritesLeft as u8)) > 0) {
                    self.status |= StatusFlag::Hit as u8;
                }

                if fg_priority {
                    (fg_pixel, fg_palette)
                } else {
                    (bg_pixel, bg_palette)
                }
            },
        };

        let color = self.read_vram(cartridge, 0x3F00 + 4 * palette as u16 + pixel as u16);
        let (r, g, b) = PALETTE[color as usize];
        let n = (self.dot as usize - 1) + (256 * self.scanline as usize);
        self.framebuffer[4 * n .. 4 * n + 4].copy_from_slice(&[r, g, b, 255]);
    }

    /**
     * Vertical increment (fine because by-scanline basis)
     */
    fn y_increment (&mut self) {
        if self.mask & (MaskFlag::Background as u8 | MaskFlag::Foreground as u8) > 0 {
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
    }

    /**
     * Horizontal increment (coarse because by-tile basis)
     */
    fn x_increment (&mut self) {
        if self.mask & (MaskFlag::Background as u8 | MaskFlag::Foreground as u8) > 0 {
            if (self.cur_address & LoopyRegister::CoarseX as u16) < 31 {
                // Increment coarse X
                self.cur_address += 1;
            } else {
                // Nametable edge, go to other horizontal nametable and reset coarse X to 0
                self.cur_address ^= LoopyRegister::CoarseX as u16 | NAMETABLE_X_MASK as u16;
            }
        }
    }

    /**
     * Load X info from temporary address
     */
    fn x_reload (&mut self) {
        let mask = LoopyRegister::CoarseX as u16 | NAMETABLE_X_MASK;
        self.cur_address = (self.cur_address & !mask) | (self.tmp_address & mask);
    }

    /**
     * Load Y info from temporary address
     */
    fn y_reload (&mut self) {
        let mask = LoopyRegister::CoarseY as u16 | LoopyRegister::FineY as u16 | NAMETABLE_Y_MASK;
        self.cur_address = (self.cur_address & !mask) | (self.tmp_address & mask);
    }

    fn vblank_start (&mut self, cpu: &mut Cpu) {
        self.status |= StatusFlag::VBlank as u8;
        if self.ctrl & (CtrlFlag::Nmi as u8) > 0 {
            cpu.interrupt_request(Interrupt::NMI);
        }
    }

    fn vblank_end (&mut self) {
        self.status &= !(StatusFlag::VBlank as u8 | StatusFlag::Hit as u8 | StatusFlag::SpriteOverflow as u8);
        self.sprite_shift_lo = [0; 8];
        self.sprite_shift_hi = [0; 8];
    }

    /**
     * Read registers
     * https://wiki.nesdev.com/w/index.php/PPU_scrolling
     * https://wiki.nesdev.com/w/index.php/PPU_registers
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
            0x2004 => {
                self.oam[self.oam_address as usize]
            },
            // PPUDATA
            0x2007 => {
                let mut dummy = self.read_buffer;

                self.read_buffer = self.read_vram(cartridge, self.cur_address);

                // Palette reads are not buffered
                if self.cur_address >= 0x3F00 {
                    dummy = self.read_buffer;
                }

                self.cur_address += if (self.ctrl & CtrlFlag::Increment as u8) > 0 { 32 } else { 1 };

                dummy
            },
            _ => self.read_buffer
        }
    }

    pub fn peek (&self, address: u16) -> Option<u8> {
        match (address % 8) + 0x2000 {
            0x2002 => Some(self.status),
            0x2004 => Some(self.oam[self.oam_address as usize]),
            0x2007 => Some(self.read_buffer),
            _ => None
        }
    }

    /**
     * Write to registers
     * https://wiki.nesdev.com/w/index.php/PPU_scrolling
     * https://wiki.nesdev.com/w/index.php/PPU_registers
     */
    pub fn write (&mut self, cartridge: &mut Cartridge, address: u16, data: u8) {
        match (address % 8) + 0x2000 {
            // PPUCTRL
            0x2000 => {
                self.ctrl = data;
                self.tmp_address = (self.tmp_address & !(LoopyRegister::Nametable as u16)) | ((self.ctrl as u16 & CtrlFlag::Nametable as u16) << 10);
            },
            // PPUMASK
            0x2001 => {
                self.mask = data;
            },
            // OAMADDR
            0x2003 => {
                self.oam_address = data;
            },
            // OAMDATA
            0x2004 => {
                self.write_oam(data);
            },
            // PPUSCROLL
            0x2005 => {
                if self.write_latch {
                    // Y scroll
                    self.tmp_address = (self.tmp_address & !(LoopyRegister::FineY as u16)) | ((data & 0b0000_0111) as u16) << 12;
                    self.tmp_address = (self.tmp_address & !(LoopyRegister::CoarseY as u16)) | ((data >> 3) as u16) << 5;
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
            _ => {}, // panic!("Invalid I/O write @ {:#x}", address),
        }
    }

    /**
     * Read memory
     * https://wiki.nesdev.com/w/index.php/PPU_memory_map
     */
    pub fn read_vram (&self, cartridge: &Cartridge, address: u16) -> u8 {
        match address % 0x4000 {
            // Pattern tables in cartridge CHR ROM/RAM
            0x0000 ..= 0x1FFF => {
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
        match address % 0x4000 {
            // Pattern tables
            0x0000 ..= 0x1FFF => {
                cartridge.write_chr(address, data);
            },
            // Name tables
            0x2000 ..= 0x3EFF => {
                // info!("Write NT {:#x} (idx {:#x}) <- {:#x}", address, self.mirror(cartridge, address) as usize - 0x2000, data);
                let mirrored = self.mirror(cartridge, address) as usize - 0x2000;
                self.nametables[mirrored] = data;
            },
            // Palettes
            0x3F00 ..= 0x3FFF => {
                // info!("Write Palette {:#x} <- {:#x}", address, data);
                let mirrored = self.mirror_palette(address) as usize - 0x3F00;
                self.palettes[mirrored] = data;
            },
            _ => panic!("Invalid write @ {:#x}", address),
        }
    }

    /**
     * Copy bytes to OAM
     */
    pub fn write_oam (&mut self, data: u8) {
        self.oam[self.oam_address as usize] = data;
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    /**
     * Mirror a nametable address
     * https://wiki.nesdev.org/w/index.php/Mirroring#Nametable_Mirroring
     */
    pub fn mirror (&self, cartridge: &Cartridge, address: u16) -> u16 {
        match cartridge.get_mirroring() {
            Mirroring::OneScreenLower | Mirroring::OneScreenUpper => address % 0x400 + 0x2000,
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
}

#[test]
fn palette () {
    let ppu = Ppu::new();

    assert_eq!(ppu.mirror_palette(0x3F00), 0x3F00);
    assert_eq!(ppu.mirror_palette(0x3F01), 0x3F01);
    assert_eq!(ppu.mirror_palette(0x3F02), 0x3F02);
    assert_eq!(ppu.mirror_palette(0x3F03), 0x3F03);
    assert_eq!(ppu.mirror_palette(0x3F04), 0x3F04);
    assert_eq!(ppu.mirror_palette(0x3F05), 0x3F05);
    assert_eq!(ppu.mirror_palette(0x3F06), 0x3F06);
    assert_eq!(ppu.mirror_palette(0x3F07), 0x3F07);
    assert_eq!(ppu.mirror_palette(0x3F08), 0x3F08);
    assert_eq!(ppu.mirror_palette(0x3F09), 0x3F09);
    assert_eq!(ppu.mirror_palette(0x3F0A), 0x3F0A);
    assert_eq!(ppu.mirror_palette(0x3F0B), 0x3F0B);
    assert_eq!(ppu.mirror_palette(0x3F0C), 0x3F0C);
    assert_eq!(ppu.mirror_palette(0x3F0D), 0x3F0D);
    assert_eq!(ppu.mirror_palette(0x3F0E), 0x3F0E);
    assert_eq!(ppu.mirror_palette(0x3F0F), 0x3F0F);
    assert_eq!(ppu.mirror_palette(0x3F10), 0x3F00);
    assert_eq!(ppu.mirror_palette(0x3F11), 0x3F11);
    assert_eq!(ppu.mirror_palette(0x3F12), 0x3F12);
    assert_eq!(ppu.mirror_palette(0x3F13), 0x3F13);
    assert_eq!(ppu.mirror_palette(0x3F14), 0x3F04);
    assert_eq!(ppu.mirror_palette(0x3F15), 0x3F15);
    assert_eq!(ppu.mirror_palette(0x3F16), 0x3F16);
    assert_eq!(ppu.mirror_palette(0x3F17), 0x3F17);
    assert_eq!(ppu.mirror_palette(0x3F18), 0x3F08);
    assert_eq!(ppu.mirror_palette(0x3F19), 0x3F19);
    assert_eq!(ppu.mirror_palette(0x3F1A), 0x3F1A);
    assert_eq!(ppu.mirror_palette(0x3F1B), 0x3F1B);
    assert_eq!(ppu.mirror_palette(0x3F1C), 0x3F0C);
    assert_eq!(ppu.mirror_palette(0x3F1D), 0x3F1D);
    assert_eq!(ppu.mirror_palette(0x3F1E), 0x3F1E);
    assert_eq!(ppu.mirror_palette(0x3F1F), 0x3F1F);
    // assert_eq!(ppu.mirror_palette(0x3F20), 0x3F00);
}
