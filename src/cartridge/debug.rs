/**
 * iNES format http://wiki.nesdev.com/w/index.php/INES
 */

use wasm_bindgen::prelude::*;
use crate::{
    cartridge::{Cartridge, ChrType, Mirroring},
    debug::Probe,
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct CartridgeDebug {
    pub (super) prg_banks: usize,
    pub (super) chr_banks: usize,
    pub (super) chr_type: ChrType,
    pub (super) mapper: u8,
    pub (super) trainer: bool,
    pub (super) ram: usize,
    pub (super) mirroring: Mirroring,
}

#[wasm_bindgen]
impl CartridgeDebug {
    #[wasm_bindgen(getter = prgBanks)]
    pub fn prg_banks (&self) -> usize {
        self.prg_banks
    }

    #[wasm_bindgen(getter = chrBanks)]
    pub fn chr_banks (&self) -> usize {
        self.chr_banks
    }

    #[wasm_bindgen(getter = chrType)]
    pub fn chr_type (&self) -> String {
        self.chr_type.to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn mapper (&self) -> u8 {
        self.mapper
    }

    #[wasm_bindgen(getter)]
    pub fn trainer (&self) -> bool {
        self.trainer
    }

    #[wasm_bindgen(getter)]
    pub fn ram (&self) -> usize {
        self.ram
    }

    #[wasm_bindgen(getter)]
    pub fn mirroring (&self) -> String {
        self.mirroring.to_string()
    }
}

impl Probe<CartridgeDebug> for Cartridge {
    fn get_debug (&self, _cartridge: &Cartridge) -> CartridgeDebug {
        self.debug.to_owned()
    }
}
