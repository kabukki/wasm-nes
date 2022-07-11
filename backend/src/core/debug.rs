use wasm_bindgen::prelude::*;
use crate::core::Emulator;

#[derive(serde::Serialize)]
pub struct Debug {
    pub clock: crate::clock::Clock,
    pub bus: crate::debug::Bus,
    pub cpu: crate::debug::Cpu,
    pub ppu: crate::debug::Ppu,
    pub cartridge: crate::debug::Cartridge,
}

#[wasm_bindgen]
impl Emulator {
    pub fn get_debug (&self) -> JsValue {
        JsValue::from_serde(&Debug {
            clock: self.clock,
            bus: crate::debug::Bus {
                ram: self.bus.wram.clone(),
                dma: self.bus.dma,
            },
            cpu: crate::debug::Cpu {
                pc: self.cpu.pc,
                sp: self.cpu.sp,
                a: self.cpu.a,
                x: self.cpu.x,
                y: self.cpu.y,
                status: self.cpu.status,
                interrupt: self.cpu.interrupt,
                clock: self.cpu.clock,
            },
            ppu: crate::debug::Ppu {
                ctrl: self.bus.ppu.ctrl,
                mask: self.bus.ppu.mask,
                status: self.bus.ppu.status,
                dot: self.bus.ppu.dot,
                scanline: self.bus.ppu.scanline,
                frame: self.bus.ppu.frame,
                oam: (0..64).map(|n| {
                    let (sprite_y, sprite_id, sprite_attributes, sprite_x) = (
                        self.bus.ppu.oam[n * 4 + 0] as u16,
                        self.bus.ppu.oam[n * 4 + 1] as u16,
                        self.bus.ppu.oam[n * 4 + 2],
                        self.bus.ppu.oam[n * 4 + 3],
                    );
                    let palette_num = sprite_attributes & crate::ppu::SpriteAttribute::Palette as u8;
                    let sprite_height = if (self.bus.ppu.ctrl & crate::ppu::CtrlFlag::SpriteHeight as u8) > 0 { 16 } else { 8 };

                    let mut img = image::RgbaImage::new(8, sprite_height);

                    for y in 0..sprite_height {
                        let row = y as u16 % 8; // Take into account 16px high tiles
                        let address = if sprite_height == 16 {
                            let half = y as u16 / 8; // Either top (0) or bottom (1) half
                            (sprite_id & 1) * 0x1000 + ((sprite_id & 0b1111_1110) + half) * 16
                        } else {
                            16 * sprite_id + if (self.bus.ppu.ctrl & crate::ppu::CtrlFlag::Sprite as u8) > 0 { 0x1000 } else { 0 }
                        };

                        let (hi, lo) = (
                            self.bus.ppu.read_vram(&self.bus.cartridge, address + row as u16 + 8 as u16),
                            self.bus.ppu.read_vram(&self.bus.cartridge, address + row as u16),
                        );

                        for x in 0..8 {
                            let (hi, lo) = (hi >> (7 - x) & 1, lo >> (7 - x) & 1);
                            let (r, g, b) = crate::ppu::PALETTE[self.bus.ppu.read_vram(&self.bus.cartridge, 0x3F10 + (palette_num << 2) as u16 + (hi << 1 | lo) as u16) as usize];
                            img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                        }
                    }

                    if (sprite_attributes & crate::ppu::SpriteAttribute::FlipHorizontal as u8) > 0 {
                        image::imageops::flip_horizontal_in_place(&mut img);
                    }

                    if (sprite_attributes & crate::ppu::SpriteAttribute::FlipVertical as u8) > 0 {
                        image::imageops::flip_vertical_in_place(&mut img);
                    }

                    crate::debug::Oam {
                        id: sprite_id,
                        x: sprite_x,
                        y: sprite_y,
                        attr: sprite_attributes,
                        tile: img.into_vec(),
                    }
                }).collect(),
                palettes: (0..8).map(|n| {
                    (0..4).map(|color| {
                        let (r, g, b) = crate::ppu::PALETTE[self.bus.ppu.read_vram(&self.bus.cartridge, 0x3F00 + n * 4 + color) as usize];
                        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                    }).collect()
                }).collect(),
                palette: crate::ppu::PALETTE.iter().map(|&(r, g, b)| {
                    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                }).collect(),
                nametables: self.bus.ppu.nametables.clone(),
                clock: self.bus.ppu.clock,
            },
            cartridge: crate::debug::Cartridge {
                ines: self.bus.cartridge.ines,
                ram: self.bus.cartridge.prg_ram.clone(),
                pattern_tables: (0..512).map(|n| {
                    // let sprite_height = if (self.bus.ppu.ctrl & crate::ppu::CtrlFlag::SpriteHeight as u8) > 0 { 16 } else { 8 };
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
                }).collect()
            },
        }).expect("Could not get debug info")
    }
}
