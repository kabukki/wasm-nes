/**
 * iNES format http://wiki.nesdev.com/w/index.php/INES
 */

use wasm_bindgen::prelude::*;
use crate::{
    cartridge::{Cartridge, ines::InesHeader},
    debug::Probe,
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct CartridgeDebug {
    ines: InesHeader,
    ram: Vec<u8>,
    rom: Vec<u8>,
}

#[wasm_bindgen]
impl CartridgeDebug {
    #[wasm_bindgen(getter)]
    pub fn ines (&self) -> InesHeader {
        self.ines.to_owned()
    }

    #[wasm_bindgen(getter)]
    pub fn ram (&self) -> Vec<u8> {
        self.ram.to_owned()
    }

    #[wasm_bindgen(getter)]
    pub fn rom (&self) -> Vec<u8> {
        self.rom.to_owned()
    }
}

impl Probe<CartridgeDebug> for Cartridge {
    fn get_debug (&self, _cartridge: &Cartridge) -> CartridgeDebug {
        CartridgeDebug {
            ines: self.ines.to_owned(),
            ram: self.prg_ram.to_vec(),
            rom: self.prg_rom.to_vec(),
        }
    }
}
