use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum Button {
    None    = 0b0000_0000,
    A       = 0b0000_0001,
    B       = 0b0000_0010,
    Select  = 0b0000_0100,
    Start   = 0b0000_1000,
    Up      = 0b0001_0000,
    Down    = 0b0010_0000,
    Left    = 0b0100_0000,
    Right   = 0b1000_0000,
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
