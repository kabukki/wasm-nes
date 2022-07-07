/**
 * https://wiki.nesdev.org/w/index.php/AxROM
 */

use log::warn;
use crate::cartridge::Mirroring;

pub struct Mapper007 {
    prg_bank: u8,
    mirroring: Option<Mirroring>,
}
impl Mapper007 {
    const PRG_WINDOW: usize = 0x8000; // 32 KiB
}
impl super::Mapper for Mapper007 {
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
            0x8000 ..= 0xFFFF => {
                prg_rom[(self.prg_bank as usize * Mapper007::PRG_WINDOW) + (address as usize % Mapper007::PRG_WINDOW)]
            },
            _ => panic!("Invalid PRG read {:#x}", address),
        }
    }
        
    fn write_prg (&mut self, address: u16, data: u8, _prg_ram: &mut Vec<u8>) {
        match address {
            0x8000 ..= 0xFFFF => {
                self.prg_bank = data & 0b0000_0111; // Max. 8 * 32 KiB = 256 KiB PRG
                self.mirroring = if (data & 0b0001_0000) > 0 { Some(Mirroring::OneScreenLower) } else { None }
            },
            _ => warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        self.mirroring
    }
}
impl Default for Mapper007 {
    fn default () -> Self {
        Mapper007 {
            prg_bank: 0,
            mirroring: None,
        }
    }
}
