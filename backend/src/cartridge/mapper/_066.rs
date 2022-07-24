/**
 * https://wiki.nesdev.org/w/index.php/GxROM
 */

use crate::cartridge::{Mirroring, Bank};

pub struct Mapper066 {
    prg_bank: u8,
    chr_bank: u8,
}

impl Mapper066 {
    const PRG_WINDOW: usize = 0x8000; // 32 KiB
    const CHR_WINDOW: usize = 0x2000; // 8 KiB
}

impl super::Mapper for Mapper066 {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8 {
        chr[(self.chr_bank as usize * Mapper066::CHR_WINDOW) + (address as usize % Mapper066::CHR_WINDOW)]
    }

    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>) {
        chr[(self.chr_bank as usize * Mapper066::CHR_WINDOW) + (address as usize % Mapper066::CHR_WINDOW)] = data;
    }

    fn read_prg (&self, address: u16, _prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> u8 {
        match address {
            0x4020 ..= 0x5FFF => {
                0
            },
            0x8000 ..= 0xFFFF => {
                prg_rom[(self.prg_bank as usize * Mapper066::PRG_WINDOW) + (address as usize % Mapper066::PRG_WINDOW)]
            },
            _ => panic!("Invalid PRG read {:#x}", address),
        }
    }

    fn peek_prg (&self, address: u16, _prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> Option<u8> {
        match address {
            0x8000 ..= 0xFFFF => Some(prg_rom[(self.prg_bank as usize * Mapper066::PRG_WINDOW) + (address as usize % Mapper066::PRG_WINDOW)]),
            _ => None,
        }
    }

    fn write_prg (&mut self, address: u16, data: u8, _prg_ram: &mut Vec<u8>) {
        match address {
            0x8000 ..= 0xFFFF => {
                self.prg_bank = (data & 0b0011_0000) >> 4; // Max. 4 * 32 KiB = 128 KiB PRG
                self.chr_bank = (data & 0b0000_0011) >> 0; // Max. 4 * 8 KiB = 32 KiB CHR
            },
            _ => log::warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }

    fn get_current_prg (&self, _prg_rom: &Vec<u8>) -> Vec<Bank> {
        vec![Bank { number: self.prg_bank, size: Mapper066::PRG_WINDOW }]
    }

    fn get_current_chr (&self, _chr: &Vec<u8>) -> Vec<Bank> {
        vec![Bank { number: self.chr_bank, size: Mapper066::CHR_WINDOW }]
    }

    fn get_bank_at (&self, _prg_rom: &Vec<u8>, address: u16) -> u8 {
        match address {
            0x8000 ..= 0xFFFF => self.prg_bank,
            _ => unreachable!(),
        }
    }
}

impl Default for Mapper066 {
    fn default () -> Self {
        Mapper066 {
            prg_bank: 0,
            chr_bank: 0,
        }
    }
}
