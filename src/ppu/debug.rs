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

use wasm_bindgen::{prelude::*, Clamped};
use crate::{
    ppu::{Ppu, tilemap::Tilemap},
    debug::Probe,
    cartridge::Cartridge,
};

impl Ppu {
    /**
     * Get the contents of the CHR-ROM pattern tables.
     * Pattern tables contain background graphics (right) and sprite graphics (left)
     * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
     */
    fn get_pattern_tables (&self, cartridge: &Cartridge) -> Vec<u8> {
        let mut map = Tilemap::new(16, 32);
        let palette = &self.palettes[..4];
    
        for n in 0..512 {
            let x = n % 16;
            let y = n / 16;

            let tile = cartridge.get_tile(n);
            map.write_tile(x, y, tile.as_slice(), palette);
        }
    
        map.buffer
    }

    /**
     * Get the palettes in use
     */
    fn get_palettes (&self) -> Vec<u8> {
        let mut map = Tilemap::new(16, 2);

        for n in 0..32 {
            let color = self.palettes[n];
            let x = n % 16;
            let y = n / 16;
            let tile = vec![0; 8 * 8];
            map.write_tile(x, y, tile.as_slice(), &[color]);
        }

        map.buffer
    }

    /**
     * Get the system palette
     */
    fn get_palette (&self) -> Vec<u8> {
        let mut map = Tilemap::new(16, 4);

        for color in 0..64 {
            let tile = vec![0; 8 * 8];
            map.write_tile(color % 16, color / 16, tile.as_slice(), &[color as u8]);
        }

        map.buffer
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PpuDebug {
    oam: Vec<u8>,
    pattern_tables: Vec<u8>,
    palettes: Vec<u8>,
    palette: Vec<u8>,
}

#[wasm_bindgen]
impl PpuDebug {
    #[wasm_bindgen(getter)]
    pub fn oam (&self) -> Vec<u8> {
        self.oam.to_owned()
    }

    #[wasm_bindgen(getter = patternTables)]
    pub fn pattern_tables (&self) -> Clamped<Vec<u8>> {
        Clamped(self.pattern_tables.to_owned())
    }

    #[wasm_bindgen(getter)]
    pub fn palettes (&self) -> Clamped<Vec<u8>> {
        Clamped(self.palettes.to_owned())
    }

    #[wasm_bindgen(getter)]
    pub fn palette (&self) -> Clamped<Vec<u8>> {
        Clamped(self.palette.to_owned())
    }
}

impl Probe<PpuDebug> for Ppu {
    fn get_debug (&self, cartridge: &Cartridge) -> PpuDebug {
        PpuDebug {
            oam: self.oam.to_vec(),
            pattern_tables: self.get_pattern_tables(cartridge),
            palettes: self.get_palettes(),
            palette: self.get_palette(),
        }
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
