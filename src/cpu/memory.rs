// One page = 0xFF. Zero page is 0-0x00FF
// stack: 0x0100 - 0x01FF (1 page)
// 1 page = 256 bytes (0x0100). addresses are in the form 0x[hh][ll], hh being the page number.
// ex: 0x01aa = page 1, 0x09ff = page 9
// MMU

// https://en.wikibooks.org/wiki/NES_Programming/Memory_Map

const MEMORY_RAM_START: u16 = 0x0000;
const MEMORY_RAM_END: u16 = 0x1FFF;
const MEMORY_IO_START: u16 = 0x2000;
const MEMORY_IO_END: u16 = 0x401F;
pub const MEMORY_CARTRIDGE_START: u16 = 0x4020;
const MEMORY_CARTRIDGE_END: u16 = 0xFFFF;

pub const MEMORY_RAM_STACK_START: u16 = 0x0100;
pub const MEMORY_CARTRIDGE_PRG_LOWER_START: u16 = 0x8000;
pub const MEMORY_CARTRIDGE_PRG_LOWER_SIZE: usize = 0x4000;
pub const MEMORY_CARTRIDGE_PRG_UPPER_START: u16 = 0xC000;
pub const MEMORY_CARTRIDGE_PRG_UPPER_SIZE: usize = 0x4000;

pub const PAGE_SIZE: usize = 0x0100;

pub enum Section {
    RAM,
    IORegisters,
    ExpansionROM,
    SRAM,
    ROM,
}

pub struct Memory {
    pub ram: [u8; 0x0800],
    // io
    pub rom: [u8; 0xBFE0],
}

impl Memory {
    pub fn new () -> Memory {
        return Memory {
            ram: [0; 0x0800],
            rom: [0; 0xBFE0],
        };
    }

    // IDEA use a lib like byteorder + read_8 or 16
    pub fn read (&self, addr: u16) -> u8 {
        match addr {
            MEMORY_RAM_START ..= MEMORY_RAM_END => self.ram[usize::from(addr - MEMORY_RAM_START) % 0x0800],
            MEMORY_IO_START ..= MEMORY_IO_END => self.ram[usize::from(addr - MEMORY_IO_START)],
            MEMORY_CARTRIDGE_START ..= MEMORY_CARTRIDGE_END => self.rom[usize::from(addr - MEMORY_CARTRIDGE_START)],
            // MEMORY_SRAM_START ..= MEMORY_SRAM_END => self.ram[usize::from(addr - MEMORY_SRAM_START)],
            // MEMORY_ROM_START ..= MEMORY_ROM_END => self.ram[usize::from(addr - MEMORY_ROM_START)],
        }
    }

    pub fn write (&mut self, addr: u16, data: u8) {
        match addr {
            MEMORY_RAM_START ..= MEMORY_RAM_END => self.ram[usize::from(addr - MEMORY_RAM_START) % 0x0800] = data,
            MEMORY_IO_START ..= MEMORY_IO_END => self.ram[usize::from(addr - MEMORY_IO_START)] = data,
            MEMORY_CARTRIDGE_START ..= MEMORY_CARTRIDGE_END => self.rom[usize::from(addr - MEMORY_CARTRIDGE_START)] = data,
            // MEMORY_SRAM_START ..= MEMORY_SRAM_END => self.ram[usize::from(addr - MEMORY_SRAM_START)],
            // MEMORY_ROM_START ..= MEMORY_ROM_END => self.ram[usize::from(addr - MEMORY_ROM_START)],
        };
    }
}

pub fn get_page (address: u16) -> u8 {
    (address & 0xff00) as u8
}
