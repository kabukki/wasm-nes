use wasm_bindgen::prelude::*;
use log::debug;
use std::io::prelude::*;
use std::io::Cursor;

// iNES format
// http://wiki.nesdev.com/w/index.php/INES

pub const PRG_ROM_BANK_SIZE: usize = 16 * 1024; // 16 KiB
pub const CHR_ROM_BANK_SIZE: usize = 8 * 1024; // 8 KiB
pub const RAM_BANK_SIZE: usize = 8 * 1024;

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

#[wasm_bindgen]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

pub struct Cartridge {
    pub sram: [u8; 2048],
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mirroring: Mirroring,
    // pub mapper: dyn Mapper
}

impl Cartridge {
    pub fn new (rom: &Vec<u8>) -> Self {
        let mut cursor = Cursor::new(rom);
        let mut header = [0u8; 16];
        cursor.read_exact(&mut header).expect("Could not read header");

        if &header[0..3] != b"NES" || header[3] != 0x1A {
            panic!("Invalid header constant");
        }

        let prg_rom_banks = header[4] as usize;
        let chr_rom_banks = header[5] as usize;
        let mapper = (header[6] & ControlFlag1::Mapper as u8) >> 4 | (header[7] & ControlFlag2::Mapper as u8);
        let trainer = header[6] & ControlFlag1::Trainer as u8 != 0;
        let ram = if header[8] == 0 { 1 } else { header[8] as usize } * 0x8000;
        let mirroring = match (header[6] & ControlFlag1::FourScreen as u8 != 0, header[6] & ControlFlag1::Vertical as u8 != 0) {
            (true, _) => Mirroring::FourScreen,
            (false, false) => Mirroring::Vertical,
            (false, true) => Mirroring::Horizontal,
        };

        debug!("PRG-ROM banks: {}\nCHR-ROM banks: {}\nMapper: {}\nRAM size: {}\nHas trainer ? {}\nMirroring: {:?}", prg_rom_banks, chr_rom_banks, mapper, ram, trainer, mirroring);

        let mut cartridge = Cartridge {
            sram: [0; 2048],
            prg_rom: vec![0; prg_rom_banks * PRG_ROM_BANK_SIZE],
            chr_rom: vec![0; chr_rom_banks * CHR_ROM_BANK_SIZE],
            // ram: vec![0; ram * RAM_BANK_SIZE],
            mirroring,
        };

        if trainer {
            cursor.seek(std::io::SeekFrom::Current(512)).expect("Could not read trainer");
        }

        cursor.read_exact(cartridge.prg_rom.as_mut()).expect("Could not read PRG-ROM");
        cursor.read_exact(cartridge.chr_rom.as_mut()).expect("Could not read CHR-ROM");

        cartridge
    }

    pub fn get_tile (&self, n: usize) -> Vec<u8> {
        let mut tile = Vec::with_capacity(8 * 8);

        for tile_y in 0..8 {
            let (hi, lo) = (self.chr_rom[n * 16 + tile_y + 8], self.chr_rom[n * 16 + tile_y]);
    
            for tile_x in 0..8 {
                let (hi, lo) = (hi >> (7 - tile_x) & 1, lo >> (7 - tile_x) & 1);
                tile.push(hi << 1 | lo);
            }
        }

        tile
    }

    pub fn read_chr (&self, address: u16) -> u8 {
        debug!("Read CHR @ {:#x} -> :#x", address);
        self.chr_rom[address as usize]
    }

    // Mapper
    pub fn read (&self, address: u16) -> u8 {
        match address {
            0x8000 ..= 0xFFFF => {
                self.prg_rom[address as usize % 0x4000] // should be handled by mapper if 1 or 2 banks
            },
            _ => panic!("Invalid cartridge read {:#x}", address),
        }
    }
        
    // Mapper
    pub fn write (&mut self, address: u16, data: u8) {
        match address {
            0x6000 ..= 0x7FFF => {
                self.sram[address as usize - 0x6000] = data;
            },
            0x8000 ..= 0xFFFF => {
                self.prg_rom[address as usize % 0x4000] = data; // should be handled by mapper if 1 or 2 banks
            },
            _ => panic!("Invalid cartridge write {:#x}", address),
        }
        // trace!("Read PRG @ {:#x} -> {:#x}", address, self.prg_rom[address as usize]);
    }

}
