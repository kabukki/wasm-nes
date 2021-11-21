/**
 * iNES format http://wiki.nesdev.com/w/index.php/INES
 */

use std::io::prelude::*;
use std::io::Cursor;
use std::fmt;
use crate::cartridge::{
    mapper::{Mapper, get_mapper},
    ines::InesHeader,
};

pub mod mapper;
pub mod ines;
pub mod debug;

pub const PRG_BANK_SIZE: usize = 0x4000; // 16 KiB
pub const CHR_BANK_SIZE: usize = 0x2000; // 8 KiB
pub const RAM_BANK_SIZE: usize = 0x2000; // 8 KiB

pub enum ControlFlag1 {
    Vertical    =   0b0000_0001,
    Ram         =   0b0000_0010,
    Trainer     =   0b0000_0100,
    FourScreen  =   0b0000_1000,
    Mapper      =   0b1111_0000, // Lower nibble of mapper number
}

pub enum ControlFlag2 {
    Unused      =   0b0000_1111,
    Mapper      =   0b1111_0000, // Upper nibble of mapper number
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mirroring {
    OneScreenLower,
    OneScreenUpper,
    Horizontal,
    Vertical,
    FourScreen,
}

impl fmt::Display for Mirroring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mirroring::OneScreenLower => write!(f, "One screen (lower)"),
            Mirroring::OneScreenUpper => write!(f, "One screen (upper)"),
            Mirroring::Horizontal => write!(f, "Horizontal"),
            Mirroring::Vertical => write!(f, "Vertical"),
            Mirroring::FourScreen => write!(f, "4-screen"),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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
    mirroring: Mirroring,
    mapper: Box<dyn Mapper>,
    ines: InesHeader,
}

impl Cartridge {
    pub fn new (rom: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(rom);
        let mut header = [0u8; 16];
        cursor.read_exact(&mut header).expect("Could not read header");

        if &header[0..3] != b"NES" || header[3] != 0x1A {
            panic!("Invalid header constant");
        }

        let prg_banks = header[4] as usize;
        let chr_banks = header[5] as usize; 
        let chr_type = if chr_banks > 0 { ChrType::ROM } else { ChrType::RAM };
        let mapper = (header[6] & ControlFlag1::Mapper as u8) >> 4 | (header[7] & ControlFlag2::Mapper as u8);
        let trainer = header[6] & ControlFlag1::Trainer as u8 != 0;
        let ram = header[8] as usize;
        let mirroring = match (header[6] & ControlFlag1::FourScreen as u8 != 0, header[6] & ControlFlag1::Vertical as u8 != 0) {
            (true, _) => Mirroring::FourScreen,
            (false, false) => Mirroring::Horizontal,
            (false, true) => Mirroring::Vertical,
        };

        let mut cartridge = Cartridge {
            prg_ram: vec![0; std::cmp::max(ram, 1) * RAM_BANK_SIZE],
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

    pub fn get_tile (&self, n: usize) -> Vec<u8> {
        let mut tile = Vec::with_capacity(8 * 8);

        for tile_y in 0..8 {
            let (hi, lo) = (self.chr[n * 16 + tile_y + 8], self.chr[n * 16 + tile_y]);
    
            for tile_x in 0..8 {
                let (hi, lo) = (hi >> (7 - tile_x) & 1, lo >> (7 - tile_x) & 1);
                tile.push(hi << 1 | lo);
            }
        }

        tile
    }
}
