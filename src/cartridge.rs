use wasm_bindgen::prelude::*;
use std::io::prelude::*;
use std::io::Cursor;
use crate::ppu::palette::BACKGROUND_PALETTE;

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

#[wasm_bindgen]
pub struct Cartridge {
    sram: [u8; 2048],
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    ram: Vec<u8>,
    mirroring: Mirroring,
    // pub mapper: dyn Mapper
}

#[wasm_bindgen]
impl Cartridge {
    pub fn new (rom: &[u8]) -> Cartridge {
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

        println!("PRG-ROM banks: {}", prg_rom_banks);
        println!("CHR-ROM banks: {}", chr_rom_banks);
        println!("Mapper: {}", mapper);
        println!("RAM size: {}", ram);
        println!("Has trainer ? {}", trainer);

        // check header
        let mut cartridge = Cartridge {
            sram: [0; 2048],
            prg_rom: vec![0; prg_rom_banks * PRG_ROM_BANK_SIZE],
            chr_rom: vec![0; chr_rom_banks * CHR_ROM_BANK_SIZE],
            ram: vec![0; ram * RAM_BANK_SIZE],
            mirroring,
        };

        if trainer {
            cursor.seek(std::io::SeekFrom::Current(512)).expect("Could not read trainer");
        }

        cursor.read_exact(cartridge.prg_rom.as_mut()).expect("Could not read PRG-ROM");
        cursor.read_exact(cartridge.chr_rom.as_mut()).expect("Could not read CHR-ROM");

        cartridge
    }

    /**
     * Get the nth tile from CHR-ROM pattern tables.
     * https://wiki.nesdev.com/w/index.php/PPU_pattern_tables
     */
    pub fn get_tile (&self, n: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(8 * 8 * 4);
        let tile = &self.chr_rom[n * 16..n * 16 + 16];

        for y in 0..8 {
            let (hi, lo) = (tile[y + 8], tile[y]);
    
            for x in 0..8 {
                let (hi, lo) = (hi >> (7 - x) & 1, lo >> (7 - x) & 1);
                let pixel = hi << 1 | lo;
    
                let (r, g, b) = match pixel {
                    0 => BACKGROUND_PALETTE[0x00],
                    1 => BACKGROUND_PALETTE[0x16],
                    2 => BACKGROUND_PALETTE[0x28],
                    3 => BACKGROUND_PALETTE[0x19],
                    _ => panic!("Invalid color"),
                };

                result.extend([r, g, b, 255].iter());
            }
        }

        result
    }

    pub fn get_tiles (&self) -> Vec<u8> {
        let mut result = Vec::new();

        for n in 0..512 {
            result.extend(self.get_tile(n).iter());
        }

        result
    }

    pub fn get_mirroring (&self) -> Mirroring {
        self.mirroring
    }
    
    // Mapper
    pub fn read (&self, address: u16) -> u8 {
        0
    }

    pub fn read_chr (&self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    pub fn read_prg (&self, address: u16) -> u8 {
        self.prg_rom[address as usize]
    }
}
