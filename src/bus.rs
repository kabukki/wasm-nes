use crate::ppu::Ppu;
use crate::cartridge::Cartridge;
use crate::controller::Controller;

// One page = 0xFF. Zero page is 0-0x00FF
// stack: 0x0100 - 0x01FF (1 page)
// 1 page = 256 bytes (0x0100). addresses are in the form 0x[hh][ll], hh being the page number.
// ex: 0x01aa = page 1, 0x09ff = page 9
// MMU

// https://en.wikibooks.org/wiki/NES_Programming/Memory_Map
// https://wiki.nesdev.com/w/index.php/PPU_registers

#[derive(Debug, Clone, Copy)]
pub struct Dma {
    pub page: u8,
    pub wait: bool,
    pub count: u8,
    pub read_buffer: u8,
}

/**
 * Memory map for the CPU
 */
pub struct Bus {
    pub wram: [u8; 0x800], // 2 KiB, mirrored (max 11 bits)
    pub ppu: Ppu,
    pub cartridge: Option<Cartridge>,
    pub dma: Option<Dma>,
    pub controllers: [Controller; 2],
    pub read_buffer: u8, // Open bus
}

impl Bus {
    pub fn new () -> Bus {
        Bus {
            wram: [0; 0x800],
            ppu: Ppu::new(),
            cartridge: None,
            dma: None,
            controllers: [Controller::new(); 2],
            read_buffer: 0,
        }
    }

    pub fn load (&mut self, rom: &Vec<u8>) {
        self.cartridge = Some(Cartridge::new(rom));
    }

    pub fn read (&mut self, address: u16) -> u8 {
        let cartridge = self.cartridge.as_ref().unwrap();

        let data = match address {
            0x0000 ..= 0x1FFF => self.wram[address as usize % 0x800],
            0x2000 ..= 0x3FFF => self.ppu.read(cartridge, address),
            0x4000 ..= 0x4015 => unimplemented!("APU not implemented"),
            0x4016 => self.controllers[0].read() | (self.read_buffer & 0b1110_0000),
            0x4017 => self.controllers[1].read() | (self.read_buffer & 0b1110_0000),
            0x4018 ..= 0x401F => panic!("Disabled functionality"),
            0x4020 ..= 0xFFFF => cartridge.read_prg(address),
        };

        self.read_buffer = data;

        data
    }

    pub fn write (&mut self, address: u16, data: u8) {
        let cartridge = self.cartridge.as_mut().unwrap();

        match address {
            0x0000 ..= 0x1FFF => {
                self.wram[address as usize % 0x800] = data;
            },
            0x2000 ..= 0x3FFF => {
                self.ppu.write(cartridge, address, data);
            }
            0x4000 ..= 0x4013 => {}, // APU
            0x4014 => {
                self.dma = Some(Dma {
                    page: data,
                    wait: true,
                    count: 0,
                    read_buffer: 0,
                });
            },
            0x4015 => {}, // APU
            0x4016 => {
                self.controllers[0].write(data);
                self.controllers[1].write(data);
            },
            0x4017 => {}, // APU
            0x4018 ..= 0x401F => panic!("Disabled functionality"),
            0x4020 ..= 0xFFFF => {
                cartridge.write_prg(address, data);
            },
        };
    }
}

impl Default for Bus {
    fn default () -> Self {
        Bus::new()
    }
}
