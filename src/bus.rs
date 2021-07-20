use log::debug;
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
    pub wram: [u8; 0x0800], // 2 KiB, mirrored (max 11 bits)
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

    pub fn read (&self, address: u16) -> u8 {
        match address {
            MEMORY_RAM_START ..= MEMORY_RAM_END => self.wram[usize::from(address - MEMORY_RAM_START) % 0x0800],
            a @ MEMORY_IO_START ..= MEMORY_IO_END => {
                debug!("I/O READ");
                match a % 8 + 0x2000 {
                    0x2002 => self.ppu.status,
                    0x2004 => self.ppu.oam_data,
                    0x2007 => self.ppu.data,
                    _ => panic!("Invalid I/O read"),
                }
            },
            MEMORY_CARTRIDGE_PRG_START ..= MEMORY_CARTRIDGE_END => self.cartridge.as_ref().unwrap().read_prg((address - MEMORY_CARTRIDGE_PRG_START as u16) % 0x4000), // should be handled by mapper if 1 or 2 banks
            _ => unimplemented!(),
            // MEMORY_SRAM_START ..= MEMORY_SRAM_END => self.wram[usize::from(address - MEMORY_SRAM_START)],
            // MEMORY_ROM_START ..= MEMORY_ROM_END => self.wram[usize::from(address - MEMORY_ROM_START)],
        }
    }

    pub fn write (&mut self, address: u16, data: u8) {
        match address {
            MEMORY_RAM_START ..= MEMORY_RAM_END => self.wram[usize::from(address - MEMORY_RAM_START) % 0x0800] = data,
            a @ MEMORY_IO_START ..= MEMORY_IO_END => {
                debug!("I/O WRITE");
                match a % 8 + 0x2000 {
                    0x2000 => { self.ppu.ctrl = data; },
                    0x2001 => { self.ppu.mask = data; },
                    0x2003 => { self.ppu.oam_address = data; },
                    0x2004 => { self.ppu.oam_data = data; },
                    0x2005 => { self.ppu.scroll = data; },
                    // 0x2006 => { self.ppu.address = data; }, ppu.write_address()
                    0x2007 => { self.ppu.data = data; },
                    _ => panic!("Invalid I/O read"),
                }
            },
            _ => unimplemented!(),
            // 0x4014 => ppu.write_dma(data)
            // MEMORY_SRAM_START ..= MEMORY_SRAM_END => self.wram[usize::from(address - MEMORY_SRAM_START)],
            // MEMORY_ROM_START ..= MEMORY_ROM_END => self.wram[usize::from(address - MEMORY_ROM_START)],
        };
    }
}

impl Default for Bus {
    fn default () -> Self {
        Bus::new()
    }
}
