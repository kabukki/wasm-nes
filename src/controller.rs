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
