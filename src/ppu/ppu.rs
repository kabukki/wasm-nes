/**
 * 1 frame (scanline) = 340 cycles
 * https://wiki.nesdev.com/w/index.php/PPU_rendering
 * https://wiki.nesdev.com/w/index.php/PPU_frame_timing
 * https://wiki.nesdev.com/w/images/d/d1/Ntsc_timing.png
 */

use crate::cpu::memory::{Memory, MEMORY_IO_OAM_DMA};

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
    // vram: [u8; 0x4000],
    oam: [u8; 256], // Sprite RAM (64 * 4 bytes)
    cycles: usize,
}

impl Ppu {
    pub fn new () -> Ppu {
        return Ppu {
            oam: [0; 256],
            cycles: 0,
        };
    }

    fn cycle () {

    }

    // fn read_oam (&self, memory: &Memory) -> [u8; 256] {
    //     // self.vram[addr % 0x4000];
    // }
}
