use wee_alloc::WeeAlloc;

#[global_allocator]
static GLOBAL: WeeAlloc = WeeAlloc::INIT;

pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod clock;
pub mod core;
pub mod cpu;
pub mod input;
pub mod ppu;
pub mod util;
pub mod debug;
