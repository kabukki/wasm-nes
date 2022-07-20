use wasm_bindgen::prelude::*;
use crate::Emulator;

#[wasm_bindgen]
impl Emulator {
    pub fn debug_cpu_pc (&mut self) -> u16 {
        self.cpu.pc
    }
    
    pub fn debug_cpu_sp (&mut self) -> u8 {
        self.cpu.sp
    }

    pub fn debug_cpu_a (&mut self) -> u8 {
        self.cpu.a
    }

    pub fn debug_cpu_x (&mut self) -> u8 {
        self.cpu.x
    }

    pub fn debug_cpu_y (&mut self) -> u8 {
        self.cpu.sp
    }

    pub fn debug_cpu_status (&mut self) -> u8 {
        self.cpu.status
    }

    pub fn debug_cpu_cycles (&mut self) -> usize {
        self.cpu.cycles
    }

    pub fn debug_cpu_interrupt (&mut self) -> JsValue {
        JsValue::from_serde(&self.cpu.interrupt).unwrap()
    }

    pub fn debug_cpu_clock (&mut self) -> JsValue {
        JsValue::from_serde(&self.cpu.clock).unwrap()
    }
}
