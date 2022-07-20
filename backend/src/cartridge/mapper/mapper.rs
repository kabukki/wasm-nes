/**
 * https://wiki.nesdev.org/w/index.php/Mapper
 */

use crate::cartridge;

pub fn get_mapper (id: u8) -> Box<dyn Mapper> {
    match id {
        0   => Box::new(cartridge::Mapper000::default()),
        1   => Box::new(cartridge::Mapper001::default()),
        2   => Box::new(cartridge::Mapper002::default()),
        3   => Box::new(cartridge::Mapper003::default()),
        7   => Box::new(cartridge::Mapper007::default()),
        66  => Box::new(cartridge::Mapper066::default()),
        _   => unimplemented!("Unsupported mapper ({})", id),
    }
}

pub trait Mapper {
    fn read_chr (&self, address: u16, chr: &Vec<u8>) -> u8;
    fn write_chr (&mut self, address: u16, data: u8, chr: &mut Vec<u8>);
    fn read_prg (&self, address: u16, prg_ram: &Vec<u8>, prg_rom: &Vec<u8>) -> u8;
    fn write_prg (&mut self, address: u16, data: u8, prg_ram: &mut Vec<u8>);
    fn get_mirroring (&self) -> Option<cartridge::Mirroring>;
    // Debug utilities
    fn get_current_prg (&self, prg_rom: &Vec<u8>) -> Vec<cartridge::Bank>;
    fn get_current_chr (&self, chr: &Vec<u8>) -> Vec<cartridge::Bank>;
    fn get_bank_at (&self, prg_rom: &Vec<u8>, address: u16) -> u8;
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Mapper> {
    fn clone(&self) -> Box<dyn Mapper> {
        todo!()
    }
}
