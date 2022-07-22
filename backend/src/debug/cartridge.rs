use wasm_bindgen::prelude::*;
use crate::Emulator;

#[wasm_bindgen]
impl Emulator {
    pub fn debug_cartridge_ines (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.cartridge.ines).unwrap()
    }

    pub fn debug_cartridge_prg_current (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.cartridge.mapper.get_current_prg(&self.bus.cartridge.prg_rom)).unwrap()
    }

    pub fn debug_cartridge_prg_capacity (&mut self) -> usize {
        self.bus.cartridge.prg_rom.len()
    }

    pub fn debug_cartridge_chr_current (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.cartridge.mapper.get_current_chr(&self.bus.cartridge.chr)).unwrap()
    }

    pub fn debug_cartridge_chr_capacity (&mut self) -> usize {
        self.bus.cartridge.chr.len()
    }

    pub fn debug_cartridge_pattern_tables (&mut self) -> JsValue {
        JsValue::from_serde(&(0..512).map(|n| {
            let mut img = image::RgbaImage::new(8, 8);

            let row = n / 16;
            let col = n % 16;
            let address = (0 << 12) | (row << 8) | (col << 4);

            for y in 0..8 {
                let (hi, lo) = (
                    self.bus.ppu.read_vram(&self.bus.cartridge, address as u16 + y as u16 + 8 as u16),
                    self.bus.ppu.read_vram(&self.bus.cartridge, address as u16 + y as u16),
                );

                for x in 0..8 {
                    let (hi, lo) = (hi >> (7 - x) & 1, lo >> (7 - x) & 1);
                    let (r, g, b) = crate::ppu::PALETTE[self.bus.ppu.read_vram(&self.bus.cartridge, 0x3F00 + (0 << 4) as u16 + (hi << 1 | lo) as u16) as usize];
                    img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                }
            }

            img.into_vec()
        }).collect::<Vec<Vec<u8>>>()).unwrap()
    }

    pub fn debug_cartridge_ram (&mut self) -> Vec<u8> {
        self.bus.cartridge.prg_ram.to_vec()
    }
}
