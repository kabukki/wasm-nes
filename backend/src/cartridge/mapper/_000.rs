/**
 * https://wiki.nesdev.org/w/index.php/NROM
 */

use crate::cartridge::{Mirroring, Bank};

#[derive(Default)]
pub struct Mapper000 {}

impl super::Mapper for Mapper000 {
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
            _ => log::warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }

    fn get_current_prg (&self, prg_rom: &Vec<u8>) -> Vec<Bank> {
        vec![Bank { number: 0, size: prg_rom.len() }]
    }

    fn get_current_chr (&self, chr: &Vec<u8>) -> Vec<Bank> {
        vec![Bank { number: 0, size: chr.len() }]
    }

    fn get_bank_at (&self, _prg_rom: &Vec<u8>, address: u16) -> u8 {
        match address {
            0x8000 ..= 0xFFFF => 0,
            _ => unreachable!(),
        }
    }
}
