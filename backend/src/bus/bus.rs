/**
 * One page = 0xFF. Zero page is 0-0x00FF
 * stack: 0x0100 - 0x01FF (1 page)
 * 1 page = 256 bytes (0x0100). addresses are in the form 0x[hh][ll], hh being the page number.
 * ex: 0x01aa = page 1, 0x09ff = page 9
 * MMU
 * https://en.wikibooks.org/wiki/NES_Programming/Memory_Map
 * https://wiki.nesdev.com/w/index.php/PPU_registers
 */

use crate::{
    ppu::Ppu,
    apu::Apu,
    cartridge::Cartridge,
    input::Controller,
    bus::Dma,
};

pub struct Bus {
    pub wram: Vec<u8>, // 2 KiB, mirrored (max 11 bits)
    pub ppu: Ppu,
    pub apu: Apu,
    pub cartridge: Cartridge,
    pub dma: Option<Dma>,
    pub controllers: [Controller; 2],
    pub read_buffer: u8, // Open bus
}

impl Bus {
    pub fn new (rom: &Vec<u8>, sample_rate: f64) -> Bus {
        Bus {
            wram: vec![0; 0x800],
            ppu: Ppu::new(),
            apu: Apu::new(sample_rate),
            cartridge: Cartridge::new(rom),
            dma: None,
            controllers: [Controller::new(); 2],
            read_buffer: 0,
        }
    }

    pub fn read (&mut self, address: u16) -> u8 {
        let data = match address {
            0x0000 ..= 0x1FFF => self.wram[address as usize % 0x800],
            0x2000 ..= 0x3FFF => self.ppu.read(&self.cartridge, address),
            0x4000 ..= 0x4015 => self.apu.read(address),
            0x4016 => self.controllers[0].read() | (self.read_buffer & 0b1110_0000),
            0x4017 => self.controllers[1].read() | (self.read_buffer & 0b1110_0000),
            0x4018 ..= 0x401F => panic!("Disabled functionality"),
            0x4020 ..= 0xFFFF => self.cartridge.read_prg(address),
        };

        self.read_buffer = data;

        data
    }

    pub fn write (&mut self, address: u16, data: u8) {
        match address {
            0x0000 ..= 0x1FFF => {
                self.wram[address as usize % 0x800] = data;
            },
            0x2000 ..= 0x3FFF => {
                self.ppu.write(&mut self.cartridge, address, data);
            }
            0x4000 ..= 0x4013 | 0x4015 | 0x4017 => {
                self.apu.write(address, data);
            },
            0x4014 => {
                self.dma = Some(Dma {
                    page: data,
                    wait: true,
                    count: 0,
                    read_buffer: 0,
                });
            },
            0x4016 => {
                self.controllers[0].write(data);
                self.controllers[1].write(data);
            },
            0x4018 ..= 0x401F => panic!("Disabled functionality"),
            0x4020 ..= 0xFFFF => {
                self.cartridge.write_prg(address, data);
            },
        };
    }
}
