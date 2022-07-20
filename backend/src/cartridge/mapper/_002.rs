/**
 * https://wiki.nesdev.org/w/index.php/UxROM
 */

use crate::cartridge::{Mirroring, Bank};

pub struct Mapper002 {
    prg_bank: u8,
}

impl Mapper002 {
    const PRG_WINDOW: usize = 0x4000; // 16 KiB
}

impl super::Mapper for Mapper002 {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8 {
        chr[address as usize]
    }

    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>) {
        chr[address as usize] = data;
    }

    fn read_prg (&self, address: u16, _prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> u8 {
        match address {
            0x4020 ..= 0x5FFF => {
                0
            },
            0x8000 ..= 0xBFFF => {
                prg_rom[(self.prg_bank as usize * Mapper002::PRG_WINDOW) + (address as usize % Mapper002::PRG_WINDOW)]
            },
            0xC000 ..= 0xFFFF => {
                prg_rom[(prg_rom.len() - Mapper002::PRG_WINDOW) + (address as usize % Mapper002::PRG_WINDOW)]
            },
            _ => panic!("Invalid PRG read {:#x}", address),
        }
    }
        
    fn write_prg (&mut self, address: u16, data: u8, _prg_ram: &mut Vec<u8>) {
        match address {
            0x8000 ..= 0xFFFF => {
                self.prg_bank = data & 0b0000_1111; // Max. 16 * 16 KiB = 256 KiB PRG
            },
            _ => log::warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }

    fn get_current_prg (&self, prg_rom: &Vec<u8>) -> Vec<Bank> {
        vec![
            Bank { number: self.prg_bank, size: Mapper002::PRG_WINDOW }, 
            Bank { number: (prg_rom.len() / Mapper002::PRG_WINDOW) as u8 - 1, size: Mapper002::PRG_WINDOW },
        ]
    }

    fn get_current_chr (&self, chr: &Vec<u8>) -> Vec<Bank> {
        vec![Bank { number: 0, size: chr.len() }]
    }

    fn get_bank_at (&self, prg_rom: &Vec<u8>, address: u16) -> u8 {
        match address {
            0x8000 ..= 0xBFFF => self.prg_bank,
            0xC000 ..= 0xFFFF => (prg_rom.len() / Mapper002::PRG_WINDOW) as u8 - 1,
            _ => unreachable!(),
        }
    }
}

impl Default for Mapper002 {
    fn default () -> Self {
        Mapper002 {
            prg_bank: 0,
        }
    }
}
