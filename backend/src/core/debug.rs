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
        
                    let sprite_address = if sprite_height == 16 {
                        let bank = sprite_id & 1;
                        bank * 0x1000 + (sprite_id >> 1) * 16
                    } else {
                        let bank = (self.bus.ppu.ctrl & crate::ppu::CtrlFlag::Sprite as u8) as u16;
                        bank * 0x1000 + sprite_id * 16
                    };
        
                    let mut img = image::RgbaImage::new(8, sprite_height);
                    
                    for y in 0..sprite_height {
                        // fix ?
                        let (hi, lo) = (
                            self.bus.cartridge.read_chr(sprite_address + y as u16 + 8),
                            self.bus.cartridge.read_chr(sprite_address + y as u16),
                        );
                        log::trace!("Sprite {:X}, hi {:04X}, lo {:04X}", sprite_id, sprite_address + y as u16, sprite_address + y as u16 + 8);
                
                        for x in 0..8 {
                            let (hi, lo) = (hi >> (7 - x) & 1, lo >> (7 - x) & 1);
                            let (r, g, b) = crate::ppu::PALETTE[self.bus.ppu.read_vram(&self.bus.cartridge, 0x3F10 + (palette_num << 2) as u16 + (hi << 1 | lo) as u16) as usize];
                            img.put_pixel(x, y as u32, image::Rgba([r, g, b, 255]));
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
                palettes: self.bus.ppu.palettes.iter().map(|&n| {
                    let (r, g, b) = crate::ppu::PALETTE[n as usize];
                    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                }).collect(),
                palette: crate::ppu::PALETTE.iter().map(|&(r, g, b)| {
                    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
                }).collect(),
                nametables: self.bus.ppu.nametables.clone(),
                clock: self.bus.ppu.clock,
            },
            cartridge: crate::debug::Cartridge {
                ines: self.bus.cartridge.ines,
                rom: self.bus.cartridge.prg_rom.clone(),
                ram: self.bus.cartridge.prg_ram.clone(),
                pattern_tables: self.bus.cartridge.get_pattern_tables(),
            },
        }).expect("Caca")
    }
}
