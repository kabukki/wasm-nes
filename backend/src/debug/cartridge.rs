use crate::cartridge;

#[derive(serde::Serialize)]
pub struct Cartridge {
    pub ines: cartridge::InesHeader,
    pub ram: Vec<u8>,
    // pub rom: Vec<u8>,
    pub pattern_tables: Vec<Vec<u8>>,
}
