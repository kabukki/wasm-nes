use wasm_bindgen::prelude::*;
use crate::Emulator;

#[wasm_bindgen]
impl Emulator {
    pub fn debug_input (&mut self) -> Vec<u8> {
        vec![
            self.bus.controllers[0].peek().unwrap(),
            self.bus.controllers[1].peek().unwrap(),
        ]
    }
}
