/**
 * https://wiki.nesdev.org/w/index.php/Mapper
 */

pub fn get_mapper (id: u8) -> Box<dyn Mapper> {
    let mapper = match id {
        0 => Mapper000 {},
        _ => unimplemented!("Unsupported mapper"),
    };

    Box::new(mapper)
}

pub trait Mapper {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8;
    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>);
    fn read_prg (&self, address: u16, prg_ram: &[u8], prg_rom: &Vec<u8>) -> u8;
    fn write_prg (&mut self, address: u16, data: u8, prg_ram: &mut [u8]);
}

/**
 * https://wiki.nesdev.org/w/index.php/NROM
 */
pub struct Mapper000 {}
impl Mapper for Mapper000 {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8 {
        chr[address as usize]
    }

    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>) {
        chr[address as usize] = data;
    }

    fn read_prg (&self, address: u16, prg_ram: &[u8], prg_rom: &Vec<u8>) -> u8 {
        match address {
            0x4020 ..= 0x5FFF => {
                0
            },
            0x6000 ..= 0x7FFF => {
                prg_ram[address as usize - 0x6000]
            },
            0x8000 ..= 0xFFFF => {
                let len = prg_rom.len();
                prg_rom[(address as usize - 0x8000) % len]
            },
            _ => panic!("Invalid cartridge read {:#x}", address),
        }
    }
        
    fn write_prg (&mut self, address: u16, data: u8, prg_ram: &mut [u8]) {
        match address {
            0x6000 ..= 0x7FFF => {
                prg_ram[address as usize - 0x6000] = data;
                // hook to save
            },
            _ => panic!("Invalid cartridge write {:#x}", address),
        }
    }
}
