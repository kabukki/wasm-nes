use log::{info};
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

pub const MEMORY_RAM_START: u16                 = 0x0000;
pub const MEMORY_RAM_STACK_START: u16           = 0x0100;
pub const MEMORY_RAM_END: u16                   = 0x1FFF;
pub const MEMORY_IO_START: u16                  = 0x2000;
pub const MEMORY_IO_CTRL: u16                   = 0x2000;
pub const MEMORY_IO_MASK: u16                   = 0x2001;
pub const MEMORY_IO_STATUS: u16                 = 0x2002;
pub const MEMORY_IO_OAM_ADDRESS: u16            = 0x2003;
pub const MEMORY_IO_OAM_DATA: u16               = 0x2004;
pub const MEMORY_IO_SCROLL: u16                 = 0x2005;
pub const MEMORY_IO_ADDRESS: u16                = 0x2006;
pub const MEMORY_IO_DATA: u16                   = 0x2007;
pub const MEMORY_IO_OAM_DMA: u16                = 0x4014; // DMA write takes up 512 cycles, blocking the CPU
pub const MEMORY_IO_END: u16                    = 0x401F;
pub const MEMORY_CARTRIDGE_START: u16           = 0x4020;
pub const MEMORY_CARTRIDGE_SRAM_START: u16      = 0x6000;
pub const MEMORY_CARTRIDGE_PRG_START: u16       = 0x8000;
pub const MEMORY_CARTRIDGE_END: u16             = 0xFFFF;

pub const PAGE_SIZE: usize                      = 0x0100;
pub const CARTRIDGE_BANK_SIZE: usize            = 0x4000;

#[derive(Debug, Clone, Copy)]
pub struct Dma {
    pub page: u8,
    pub wait: bool,
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
}

impl Bus {
    pub fn new () -> Bus {
        Bus {
            wram: [0; 0x800],
            ppu: Ppu::new(),
            cartridge: None,
            dma: None,
            controllers: [Controller::new(); 2],
        }
    }

    pub fn load (&mut self, rom: &Vec<u8>) {
        self.cartridge = Some(Cartridge::new(rom));
    }

    pub fn read (&mut self, address: u16) -> u8 {
        let cartridge = self.cartridge.as_ref().unwrap();

        match address {
            0x0000 ..= 0x1FFF => self.wram[address as usize % 0x800],
            0x2000 ..= 0x3FFF => self.ppu.read(cartridge, address),
            0x4000 ..= 0x4015 => unimplemented!("APU not implemented"),
            0x4016 => self.controllers[0].read(),
            0x4017 => self.controllers[1].read(),
            0x4018 ..= 0x401F => panic!("Disabled functionality"),
            0x4020 ..= 0xFFFF => cartridge.read(address),
        }
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
                cartridge.write(address, data);
            },
        };
    }
}

impl Default for Bus {
    fn default () -> Self {
        Bus::new()
    }
}
