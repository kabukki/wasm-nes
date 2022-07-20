use wasm_bindgen::prelude::*;
use crate::{cpu, Emulator};

#[wasm_bindgen]
impl Emulator {
    pub fn debug_bus_ram (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.wram).unwrap()
    }

    pub fn debug_bus_stack (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.wram[cpu::MEMORY_RAM_STACK_START as usize .. cpu::MEMORY_RAM_STACK_START as usize + u8::MAX as usize]).unwrap()
    }

    pub fn debug_bus_dma (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.dma).unwrap()
    }
}
