/**
 * https://wiki.nesdev.org/w/index.php/INES_Mapper_003
 */

use crate::cartridge::{Mirroring, Bank};

pub struct Mapper003 {
    chr_bank: u8,
}

impl Mapper003 {
    const CHR_WINDOW: usize = 0x2000; // 8 KiB
}

impl super::Mapper for Mapper003 {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8 {
        chr[(self.chr_bank as usize * Mapper003::CHR_WINDOW) + (address as usize % Mapper003::CHR_WINDOW)]
    }

    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>) {
        chr[(self.chr_bank as usize * Mapper003::CHR_WINDOW) + (address as usize % Mapper003::CHR_WINDOW)] = data;
    }

    fn read_prg (&self, address: u16, _prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> u8 {
        match address {
            0x4020 ..= 0x5FFF => {
                0
            },
            0x8000 ..= 0xFFFF => {
                prg_rom[(address as usize - 0x8000) % prg_rom.len()]
            },
            _ => panic!("Invalid PRG read {:#x}", address),
        }
    }
        
    fn write_prg (&mut self, address: u16, data: u8, _prg_ram: &mut Vec<u8>) {
        match address {
            0x8000 ..= 0xFFFF => {
                self.chr_bank = data & 0b0000_0011; // Max. 4 * 8 KiB = 32 KiB CHR
            },
            _ => log::warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }

    fn get_current_prg (&self, prg_rom: &Vec<u8>) -> Vec<Bank> {
        vec![Bank { number: 0, size: prg_rom.len() }]
    }

    fn get_current_chr (&self, _chr: &Vec<u8>) -> Vec<Bank> {
        vec![Bank { number: self.chr_bank, size: Mapper003::CHR_WINDOW }]
    }

    fn get_bank_at (&self, _prg_rom: &Vec<u8>, address: u16) -> u8 {
        match address {
            0x8000 ..= 0xFFFF => 0,
            _ => unreachable!(),
        }
    }
}

impl Default for Mapper003 {
    fn default () -> Self {
        Mapper003 {
            chr_bank: 0,
        }
    }
}
