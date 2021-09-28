use log::warn;
use crate::cartridge::Mirroring;

pub fn get_mapper (id: u8) -> Box<dyn Mapper> {
    match id {
        0 => Box::new(Mapper000::default()),
        1 => Box::new(Mapper001::default()),
        2 => Box::new(Mapper002::default()),
        3 => Box::new(Mapper003::default()),
        66 => Box::new(Mapper066::default()),
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
            _ => warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }
}

/**
 * https://wiki.nesdev.org/w/index.php/MMC1
 */
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
impl Mapper for Mapper001 {
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

/**
 * https://wiki.nesdev.org/w/index.php/UxROM
 */
pub struct Mapper002 {
    prg_bank: u8,
}
impl Mapper002 {
    const PRG_WINDOW: usize = 0x4000; // 16 KiB
}
impl Mapper for Mapper002 {
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
            _ => warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }
}
impl Default for Mapper002 {
    fn default () -> Self {
        Mapper002 {
            prg_bank: 0,
        }
    }
}

/**
 * https://wiki.nesdev.org/w/index.php/INES_Mapper_003
 */
pub struct Mapper003 {
    chr_bank: u8,
}
impl Mapper003 {
    const CHR_WINDOW: usize = 0x2000; // 8 KiB
}
impl Mapper for Mapper003 {
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
            _ => warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
    }
}
impl Default for Mapper003 {
    fn default () -> Self {
        Mapper003 {
            chr_bank: 0,
        }
    }
}

/**
 * https://wiki.nesdev.org/w/index.php/GxROM
 */
pub struct Mapper066 {
    prg_bank: u8,
    chr_bank: u8,
}
impl Mapper066 {
    const PRG_WINDOW: usize = 0x8000; // 32 KiB
    const CHR_WINDOW: usize = 0x2000; // 8 KiB
}
impl Mapper for Mapper066 {
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
        
    fn write_prg (&mut self, address: u16, data: u8, _prg_ram: &mut Vec<u8>) {
        match address {
            0x8000 ..= 0xFFFF => {
                self.prg_bank = (data & 0b0011_0000) >> 4; // Max. 4 * 32 KiB = 128 KiB PRG
                self.chr_bank = (data & 0b0000_0011) >> 0; // Max. 4 * 8 KiB = 32 KiB CHR
            },
            _ => warn!("Invalid PRG write {:#x}", address),
        }
    }

    fn get_mirroring (&self) -> Option<Mirroring> {
        None
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
