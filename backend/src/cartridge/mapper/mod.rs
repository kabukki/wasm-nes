use crate::cartridge::Mirroring;

mod _000;
mod _001;
mod _002;
mod _003;
mod _007;
mod _066;

pub fn get_mapper (id: u8) -> Box<dyn Mapper> {
    match id {
        0   => Box::new(_000::Mapper000::default()),
        1   => Box::new(_001::Mapper001::default()),
        2   => Box::new(_002::Mapper002::default()),
        3   => Box::new(_003::Mapper003::default()),
        7   => Box::new(_007::Mapper007::default()),
        66  => Box::new(_066::Mapper066::default()),
        _   => unimplemented!("Unsupported mapper ({})", id),
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

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Mapper> {
    fn clone(&self) -> Box<dyn Mapper> {
        todo!()
    }
}
