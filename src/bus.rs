// use log::{debug, trace};
use crate::ppu::Ppu;
use crate::cartridge::Cartridge;

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

/**
 * Memory map for the CPU
 */
pub struct Bus {
    pub wram: [u8; 0x800], // 2 KiB, mirrored (max 11 bits)
    pub ppu: Ppu,
    pub cartridge: Option<Cartridge>,
}

impl Bus {
    pub fn new () -> Bus {
        Bus {
            wram: [0; 0x800],
            ppu: Ppu::new(),
            cartridge: None,
        }
    }

    pub fn load (&mut self, rom: &Vec<u8>) {
        self.cartridge = Some(Cartridge::new(rom));
    }

    pub fn read (&mut self, address: u16) -> u8 {
        let cartridge = self.cartridge.as_ref().unwrap();

        match address {
            MEMORY_RAM_START ..= MEMORY_RAM_END => self.wram[usize::from(address - MEMORY_RAM_START) % 0x800],
            MEMORY_IO_START ..= MEMORY_IO_END => {
                // debug!("I/O READ {:#x}", address);
                match address {
                    0x2002 ..= 0x2007 => self.ppu.read(cartridge, address),
                    0x4016 => 0, // Controller 1
                    0x4017 => 0, // Controller 2
                    _ => panic!("Invalid I/O read {:#x}", address),
                }
            },
            MEMORY_CARTRIDGE_PRG_START ..= MEMORY_CARTRIDGE_END => cartridge.read_prg((address - MEMORY_CARTRIDGE_PRG_START as u16) % 0x4000), // should be handled by mapper if 1 or 2 banks
            _ => unimplemented!(),
            // MEMORY_SRAM_START ..= MEMORY_SRAM_END => self.wram[usize::from(address - MEMORY_SRAM_START)],
            // MEMORY_ROM_START ..= MEMORY_ROM_END => self.wram[usize::from(address - MEMORY_ROM_START)],
        }
    }

    pub fn write (&mut self, address: u16, data: u8) {
        let cartridge = self.cartridge.as_mut().unwrap();

        match address {
            MEMORY_RAM_START ..= MEMORY_RAM_END => self.wram[usize::from(address - MEMORY_RAM_START) % 0x800] = data,
            MEMORY_IO_START ..= MEMORY_IO_END => {
                // trace!("I/O WRITE {:#x} {}", address, data);
                match address {
                    0x2000 ..= 0x3FFF => { self.ppu.write(cartridge, address, data); }
                    0x4000 ..= 0x4013 => {}, // APU
                    0x4014 => {}, // OAM DMA
                    0x4015 => {}, // APU
                    0x4016 => {}, // Controllers
                    0x4017 => {}, // APU
                    _ => panic!("Invalid I/O write {:#x}", address),
                }
            },
            // 0x4014 => ppu.write_dma(data)
            // MEMORY_SRAM_START ..= MEMORY_SRAM_END => self.wram[usize::from(address - MEMORY_SRAM_START)],
            MEMORY_CARTRIDGE_START ..= MEMORY_CARTRIDGE_END => {
                cartridge.write(address, data);
            },
            _ => panic!("Invalid write {:#x}", address),
        };
    }
}

impl Default for Bus {
    fn default () -> Self {
        Bus::new()
    }
}
