use std::{io::{Cursor, prelude::*}, fmt};
use crate::cartridge::*;

const PRG_BANK_SIZE: usize = 0x4000; // 16 KiB
const CHR_BANK_SIZE: usize = 0x2000; // 8 KiB
const RAM_BANK_SIZE: usize = 0x2000; // 8 KiB

enum ControlFlag1 {
    Vertical    =   0b0000_0001,
    Ram         =   0b0000_0010,
    Trainer     =   0b0000_0100,
    FourScreen  =   0b0000_1000,
    Mapper      =   0b1111_0000, // Lower nibble of mapper number
}

enum ControlFlag2 {
    _Unused     =   0b0000_1111,
    Mapper      =   0b1111_0000, // Upper nibble of mapper number
}

#[derive(Debug, PartialEq, Copy, Clone, serde::Serialize)]
pub enum ChrType {
    ROM,
    RAM,
}

impl fmt::Display for ChrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChrType::ROM => write!(f, "ROM"),
            ChrType::RAM => write!(f, "RAM"),
        }
    }
}

pub struct Cartridge {
    pub prg_ram: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr: Vec<u8>,
    pub mirroring: Mirroring,
    pub mapper: Box<dyn Mapper>,
    pub ines: InesHeader,
}

impl Cartridge {
    pub fn new (rom: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(rom);
        let mut header = [0u8; 16];
        cursor.read_exact(&mut header).expect("Could not read header");

        if &header[0..4] != b"NES\x1A" {
            panic!("Invalid header constant");
        }

        let prg_banks = header[4] as usize;
        let chr_banks = header[5] as usize; 
        let chr_type = if chr_banks > 0 { ChrType::ROM } else { ChrType::RAM };
        let mapper = (header[6] & ControlFlag1::Mapper as u8) >> 4 | (header[7] & ControlFlag2::Mapper as u8);
        let trainer = (header[6] & ControlFlag1::Trainer as u8) != 0;
        let ram = (header[6] & ControlFlag1::Ram as u8) != 0;
        let ram_size = header[8] as usize;
        let mirroring = match (header[6] & ControlFlag1::FourScreen as u8 != 0, header[6] & ControlFlag1::Vertical as u8 != 0) {
            (true, _)       => Mirroring::FourScreen,
            (false, false)  => Mirroring::Horizontal,
            (false, true)   => Mirroring::Vertical,
        };

        let mut cartridge = Cartridge {
            prg_ram: vec![0; std::cmp::max(ram_size, 1) * RAM_BANK_SIZE], // Value 0 infers 8 KB for compatibility
            prg_rom: vec![0; prg_banks * PRG_BANK_SIZE],
            chr: vec![0; std::cmp::max(chr_banks, 1) * CHR_BANK_SIZE], // No distinction between CHR ROM and RAM
            mirroring,
            mapper: get_mapper(mapper),
            ines: InesHeader {
                prg_banks,
                chr_banks,
                chr_type,
                mapper,
                ram,
                trainer,
                mirroring,
            },
        };

        if trainer {
            cursor.seek(std::io::SeekFrom::Current(512)).expect("Could not read trainer");
        }

        cursor.read_exact(cartridge.prg_rom.as_mut()).expect("Could not read PRG-ROM");
        if chr_type == ChrType::ROM {
            cursor.read_exact(cartridge.chr.as_mut()).expect("Could not read CHR-ROM");
        }

        cartridge
    }

    pub fn read_chr (&self, address: u16) -> u8 {
        self.mapper.read_chr(address, &self.chr)
    }

    pub fn write_chr (&mut self, address: u16, data: u8) {
        self.mapper.write_chr(address, data, &mut self.chr);
    }

    pub fn read_prg (&self, address: u16) -> u8 {
        self.mapper.read_prg(address, &self.prg_ram, &self.prg_rom)
    }
        
    pub fn write_prg (&mut self, address: u16, data: u8) {
        // println!("Write PRG @ {:#x} <- {:#x}", address, data);
        self.mapper.write_prg(address, data, &mut self.prg_ram);
    }

    pub fn get_mirroring (&self) -> Mirroring {
        self.mapper.get_mirroring().unwrap_or(self.mirroring)
    }
}
