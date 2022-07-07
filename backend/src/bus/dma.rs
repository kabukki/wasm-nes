#[derive(Debug, Copy, Clone, serde::Serialize)]
pub struct Dma {
    pub page: u8,
    pub wait: bool,
    pub count: u8,
    pub read_buffer: u8,
}
