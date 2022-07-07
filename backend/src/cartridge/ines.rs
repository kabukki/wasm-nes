/**
 * iNES format http://wiki.nesdev.com/w/index.php/INES
 */

use crate::cartridge::{ChrType, Mirroring};

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
