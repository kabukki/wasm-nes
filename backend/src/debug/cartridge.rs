use crate::cartridge;

#[derive(serde::Serialize)]
pub struct Cartridge {
    pub ines: cartridge::InesHeader,
    pub pattern_tables: Vec<Vec<u8>>,
}
