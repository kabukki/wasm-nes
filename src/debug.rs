use crate::cartridge::Cartridge;

pub trait Probe<T> {
    fn get_debug (&self, cartridge: &Cartridge) -> T;
}
