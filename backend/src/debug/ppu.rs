#[derive(Clone, serde::Serialize)]
pub struct Ppu {
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub dot: u16,
    pub scanline: u16,
    pub frame: usize,
    pub oam: Vec<Oam>,
    pub palettes: Vec<Vec<u32>>,
    pub palette: Vec<u32>,
    pub nametables: Vec<u8>,
    pub clock: crate::clock::ClockDivider,
}

#[derive(Clone, serde::Serialize)]
pub struct Oam {
    pub id: u16,
    pub x: u8,
    pub y: u16,
    pub attr: u8,
    pub tile: Vec<u8>,
}
