use wasm_bindgen::prelude::*;
use wee_alloc::WeeAlloc;

pub mod bus;
pub mod cartridge;
pub mod mapper;
pub mod cpu;
pub mod instruction;
pub mod ppu;
pub mod tilemap;
pub mod controller;
pub mod apu;
pub mod clock;
pub mod debug;
pub mod nes;

#[global_allocator]
static GLOBAL: WeeAlloc = WeeAlloc::INIT;

#[wasm_bindgen]
pub fn set_panic_hook () {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn set_log () {
    console_log::init_with_level(log::Level::Trace).expect("Could not set up logger");
}
