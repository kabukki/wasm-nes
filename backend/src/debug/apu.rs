use wasm_bindgen::prelude::*;
use crate::Emulator;

#[wasm_bindgen]
impl Emulator {
    pub fn debug_apu_clock (&mut self) -> JsValue {
        JsValue::from_serde(&self.bus.apu.clock).unwrap()
    }
}
