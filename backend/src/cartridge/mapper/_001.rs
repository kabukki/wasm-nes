/**
 * https://wiki.nesdev.org/w/index.php/MMC1
 */

use log::warn;
use crate::cartridge::Mirroring;

pub struct Mapper001 {
    prg_bank: u8,
    chr_bank_0: u8,
    chr_bank_1: u8,
    load: u8,
    ctrl: u8,
}
impl Mapper001 {
    const PRG_WINDOW: usize         = 0x4000; // 16 KiB
    const PRG_WINDOW_LARGE: usize   = 0x8000; // 32 KiB
    const CHR_WINDOW: usize         = 0x1000; // 4 KiB
    const CHR_WINDOW_LARGE: usize   = 0x2000; // 8 KiB
}
impl super::Mapper for Mapper001 {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8 {
        match (self.ctrl & 0b0001_0000) >> 4 {
            0 => chr[((self.chr_bank_0 & 0b0001_1110) as usize * Mapper001::CHR_WINDOW_LARGE) + (address as usize % Mapper001::CHR_WINDOW_LARGE)],
            1 => match address {
                0x0000 ..= 0x0FFF => chr[(self.chr_bank_0 as usize * Mapper001::CHR_WINDOW) + (address as usize % Mapper001::CHR_WINDOW)],
                0x1000 ..= 0x1FFF => chr[(self.chr_bank_1 as usize * Mapper001::CHR_WINDOW) + (address as usize % Mapper001::CHR_WINDOW)],
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>) {
        let mapped_address = match (self.ctrl & 0b0001_0000) >> 4 {
            0 => ((self.chr_bank_0 & 0b0001_1110) as usize * Mapper001::CHR_WINDOW_LARGE) + (address as usize % Mapper001::CHR_WINDOW_LARGE),
            1 => match address {
                0x0000 ..= 0x0FFF => (self.chr_bank_0 as usize * Mapper001::CHR_WINDOW) + (address as usize % Mapper001::CHR_WINDOW),
                0x1000 ..= 0x1FFF => (self.chr_bank_1 as usize * Mapper001::CHR_WINDOW) + (address as usize % Mapper001::CHR_WINDOW),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        chr[mapped_address] = data;
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
                match (self.ctrl & 0b0000_1100) >> 2 {
                    0b00 | 0b01 => prg_rom[((self.prg_bank & 0b0001_1110) as usize * Mapper001::PRG_WINDOW_LARGE) + (address as usize % Mapper001::PRG_WINDOW_LARGE)],
                    0b10 => match address {
                        0x8000 ..= 0xBFFF => prg_rom[address as usize % Mapper001::PRG_WINDOW],
                        0xC000 ..= 0xFFFF => prg_rom[(self.prg_bank as usize * Mapper001::PRG_WINDOW) + (address as usize % Mapper001::PRG_WINDOW)],
                        _ => unreachable!(),
                    },
                    0b11 => match address {
                        0x8000 ..= 0xBFFF => prg_rom[(self.prg_bank as usize * Mapper001::PRG_WINDOW) + (address as usize % Mapper001::PRG_WINDOW)],
                        0xC000 ..= 0xFFFF => prg_rom[(prg_rom.len() - Mapper001::PRG_WINDOW) + (address as usize % Mapper001::PRG_WINDOW)],
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            },
            _ => panic!("Invalid PRG read {:#x}", address),
        }
    }
        
    fn write_prg (&mut self, address: u16, data: u8, prg_ram: &mut Vec<u8>) {
        match address {
            0x6000 ..= 0x7FFF => {
                prg_ram[address as usize - 0x6000] = data;
            },
            0x8000 ..= 0xFFFF => {
                if (data & 0b1000_0000) > 0 {
                    self.load = 0b0001_0000;
                } else {
                    let last = (self.load & 0b0000_0001) == 1;

                    self.load = (self.load >> 1) | (data & 0b0000_0001) << 4;

                    // Five bits have been written to the load register
                    if last {
                        match address {
                            0x8000 ..= 0x9FFF => {
                                self.ctrl = self.load;
                            },
                            0xA000 ..= 0xBFFF => {
                                self.chr_bank_0 = self.load;
                            },
                            0xC000 ..= 0xDFFF => {
                                self.chr_bank_1 = self.load;
                            },
                            0xE000 ..= 0xFFFF => {
                                self.prg_bank = self.load;
                            },
                            _ => unreachable!(),
                        }
                        self.load = 0b0001_0000;
                    }
                }
            },
            _ => warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        match self.ctrl & 0b0000_0011 {
            0b00 => Some(Mirroring::OneScreenLower),
            0b01 => Some(Mirroring::OneScreenUpper),
            0b10 => Some(Mirroring::Vertical),
            0b11 => Some(Mirroring::Horizontal),
            _ => unreachable!(),
        }
    }
}
impl Default for Mapper001 {
    fn default () -> Self {
        Mapper001 {
            prg_bank: 0,
            chr_bank_0: 0,
            chr_bank_1: 0,
            load: 0b0001_0000,
            ctrl: 0b0001_1100,
        }
    }
}
