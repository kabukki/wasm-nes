use crate::{cpu, clock};

#[derive(Copy, Clone, serde::Serialize)]
pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub interrupt: Option<cpu::Interrupt>,
    pub clock: clock::ClockDivider,
}
