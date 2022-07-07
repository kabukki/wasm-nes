use crate::bus;

#[derive(serde::Serialize)]
pub struct Bus {
    pub ram: Vec<u8>,
    pub dma: Option<bus::Dma>,
    // disassembly: Vec<(u16, Instruction)>, from prg_rom
}
