/**
 * iNES format http://wiki.nesdev.com/w/index.php/INES
 */

// use wasm_bindgen::prelude::wasm_bindgen;
use crate::cartridge::{ChrType, Mirroring};

// #[wasm_bindgen]
#[derive(Copy, Clone, serde::Serialize)]
pub struct InesHeader {
    pub prg_banks: usize,
    pub chr_banks: usize,
    pub chr_type: ChrType,
    pub mapper: u8,
    pub trainer: bool,
    pub ram: usize,
    pub mirroring: Mirroring,
}
