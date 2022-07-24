use wasm_bindgen::prelude::*;
use crate::{ppu, Emulator};

#[derive(Clone, serde::Serialize)]
struct Oam {
    id: u16,
    x: u8,
    y: u16,
    attr: u8,
    tile: Vec<u8>,
}

#[wasm_bindgen]
impl Emulator {
    pub fn debug_ppu_ctrl (&mut self) -> u8 {
        self.bus.ppu.ctrl
    }

    pub fn debug_ppu_mask (&mut self) -> u8 {
        self.bus.ppu.mask
    }

    pub fn debug_ppu_status (&mut self) -> u8 {
        self.bus.ppu.status
    }

    pub fn debug_ppu_dot (&mut self) -> u16 {
        self.bus.ppu.dot
    }

    pub fn debug_ppu_scanline (&mut self) -> u16 {
        self.bus.ppu.scanline
    }

    pub fn debug_ppu_frame (&mut self) -> usize {
        self.bus.ppu.frame
    }

    pub fn debug_ppu_oam (&mut self) -> JsValue {
        let oam: Vec<Oam> = (0..64).map(|n| {
            let (sprite_y, sprite_id, sprite_attributes, sprite_x) = (
                self.bus.ppu.oam[n * 4 + 0] as u16,
                self.bus.ppu.oam[n * 4 + 1] as u16,
                self.bus.ppu.oam[n * 4 + 2],
                self.bus.ppu.oam[n * 4 + 3],
            );
            let palette_num = sprite_attributes & ppu::SpriteAttribute::Palette as u8;
            let sprite_height = if (self.bus.ppu.ctrl & ppu::CtrlFlag::SpriteHeight as u8) > 0 { 16 } else { 8 };

            let mut img = image::RgbaImage::new(8, sprite_height);

            for y in 0..sprite_height {
                let row = y as u16 % 8; // Take into account 16px high tiles
                let address = if sprite_height == 16 {
                    let half = y as u16 / 8; // Either top (0) or bottom (1) half
                    (sprite_id & 1) * 0x1000 + ((sprite_id & 0b1111_1110) + half) * 16
                } else {
                    16 * sprite_id + if (self.bus.ppu.ctrl & ppu::CtrlFlag::Sprite as u8) > 0 { 0x1000 } else { 0 }
                };

                let (hi, lo) = (
                    self.bus.ppu.read_vram(&self.bus.cartridge, address + row as u16 + 8 as u16),
                    self.bus.ppu.read_vram(&self.bus.cartridge, address + row as u16),
                );

                for x in 0..8 {
                    let (hi, lo) = (hi >> (7 - x) & 1, lo >> (7 - x) & 1);
                    let (r, g, b) = ppu::PALETTE[self.bus.ppu.read_vram(&self.bus.cartridge, 0x3F10 + (palette_num << 2) as u16 + (hi << 1 | lo) as u16) as usize];
                    img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                }
            }

            if (sprite_attributes & ppu::SpriteAttribute::FlipHorizontal as u8) > 0 {
                image::imageops::flip_horizontal_in_place(&mut img);
            }

            if (sprite_attributes & ppu::SpriteAttribute::FlipVertical as u8) > 0 {
                image::imageops::flip_vertical_in_place(&mut img);
            }

            Oam {
                id: sprite_id,
                x: sprite_x,
                y: sprite_y,
                attr: sprite_attributes,
                tile: img.into_vec(),
            }
        }).collect();

        JsValue::from_serde(&oam).unwrap()
    }

    pub fn debug_ppu_palettes (&mut self) -> JsValue {
        let palettes: Vec<Vec<u32>> = (0..8).map(|n| {
            (0..4).map(|color| {
                let (r, g, b) = ppu::PALETTE[self.bus.ppu.read_vram(&self.bus.cartridge, 0x3F00 + n * 4 + color) as usize];
                ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
            }).collect()
        }).collect();

        JsValue::from_serde(&palettes).unwrap()
    }

    pub fn debug_ppu_palette (&mut self) -> JsValue {
        let palette: Vec<u32> = ppu::PALETTE.iter().map(|&(r, g, b)| {
            ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }).collect();

        JsValue::from_serde(&palette).unwrap()
    }

    pub fn debug_ppu_nametables (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.ppu.nametables).unwrap()
    }

    pub fn debug_ppu_clock (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.ppu.clock).unwrap()
    }
}
