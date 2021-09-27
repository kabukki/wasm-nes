use crate::cartridge::{Mirroring, PRG_BANK_SIZE};

pub fn get_mapper (id: u8) -> Box<dyn Mapper> {
    match id {
        0 => Box::new(Mapper000::default()),
        2 => Box::new(Mapper002::default()),
        _ => unimplemented!("Unsupported mapper"),
    }
}

/**
 * https://wiki.nesdev.org/w/index.php/Mapper
 */
pub trait Mapper {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8;
    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>);
    fn read_prg (&self, address: u16, prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> u8;
    fn write_prg (&mut self, address: u16, data: u8, prg_ram: &mut Vec<u8>);
    fn get_mirroring (&self) -> Option<Mirroring>;
}

/**
 * https://wiki.nesdev.org/w/index.php/NROM
 */
#[derive(Default)]
pub struct Mapper000 {}
impl Mapper for Mapper000 {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8 {
        chr[address as usize]
    }

    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>) {
        chr[address as usize] = data;
    }

    fn read_prg (&self, address: u16, prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> u8 {
        match address {
            0x4020 ..= 0x5FFF => {
                0
            },
            0x6000 ..= 0x7FFF => {
                prg_ram[(address as usize - 0x6000) % prg_ram.len()]
            },
            0x8000 ..= 0xFFFF => {
                prg_rom[(address as usize - 0x8000) % prg_rom.len()]
            },
            _ => panic!("Invalid PRG read {:#x}", address),
        }
    }
        
    fn write_prg (&mut self, address: u16, data: u8, prg_ram: &mut Vec<u8>) {
        match address {
            0x6000 ..= 0x7FFF => {
                prg_ram[address as usize - 0x6000] = data;
            },
            _ => panic!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }
}

/**
 * https://wiki.nesdev.org/w/index.php/UxROM
 */
pub struct Mapper002 {
    bank: u8,
}
impl Mapper for Mapper002 {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8 {
        chr[address as usize]
    }

    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>) {
        chr[address as usize] = data;
    }

    fn read_prg (&self, address: u16, prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> u8 {
        match address {
            0x4020 ..= 0x5FFF => {
                0
            },
            0x6000 ..= 0x7FFF => {
                prg_ram[(address as usize - 0x6000) % prg_ram.len()]
            },
            0x8000 ..= 0xBFFF => {
                prg_rom[self.bank as usize * PRG_BANK_SIZE + address as usize % PRG_BANK_SIZE]
            },
            0xC000 ..= 0xFFFF => {
                prg_rom[prg_rom.len() - PRG_BANK_SIZE + address as usize % PRG_BANK_SIZE]
            },
            _ => panic!("Invalid PRG read {:#x}", address),
        }
    }
        
    fn write_prg (&mut self, address: u16, data: u8, prg_ram: &mut Vec<u8>) {
        match address {
            0x6000 ..= 0x7FFF => {
                prg_ram[address as usize - 0x6000] = data;
                // hook to save
            },
            0x8000 ..= 0xFFFF => {
                self.bank = data & 0b0000_1111;
            },
            _ => panic!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }
}
impl Default for Mapper002 {
    fn default () -> Self {
        Mapper002 {
            bank: 0,
        }
    }
}
