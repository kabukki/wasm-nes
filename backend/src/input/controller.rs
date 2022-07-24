use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum Button {
    None    = 0b00000000,
    A       = 0b00000001,
    B       = 0b00000010,
    Select  = 0b00000100,
    Start   = 0b00001000,
    Up      = 0b00010000,
    Down    = 0b00100000,
    Left    = 0b01000000,
    Right   = 0b10000000,
}

#[derive(Clone, Copy)]
pub struct Controller {
    shift: u8,
    state: u8,
    strobe: bool,
}

impl Controller {
    pub fn new () -> Self {
        Self {
            shift: 0,
            state: 0,
            strobe: false,
        }
    }

    pub fn read (&mut self) -> u8 {
        if self.strobe {
            self.shift = self.state;
        }

        let data = self.shift & 1;
        self.shift >>= 1;
        self.shift |= 0b1000_0000;

        data
    }

    pub fn peek (&self) -> Option<u8> {
        Some(self.state)
    }

    pub fn write (&mut self, data: u8) {
        self.strobe = (data & 1) == 1;
        if self.strobe {
            self.shift = self.state;
        }
    }

    /**
     * Update input state
     */
    pub fn update (&mut self, data: u8) {
        self.state = data;
    }
}
