use super::LENGTH_TABLE;

const PULSE_TABLE: [u8; 4] = [
    0b0000_0001, // -_______ 12.5%
    0b0000_0011, // -______- 25%
    0b0000_1111, // -____--- 50%
    0b1111_1100, // _------_ 25% inverted
];

/**
 * https://wiki.nesdev.org/w/index.php/APU_Pulse
 */
pub struct Pulse {
    id: u8,
    output: u8, // 0-15
    sequence: u8,
    pub length: u8,
    length_halt: bool,
    timer: u16,
    timer_reload: u16, // Frequency (?)
    volume: u8,
    volume_constant: u8,
    envelope_start: bool,
    envelope_timer: u8,
    envelope_timer_reload: u8,
    envelope_loop: bool,
    envelope_enabled: bool,
    sweep_timer: u8,
    sweep_negate: bool,
    sweep_reload: u8,
    sweep_shift: u8,
    sweep_enabled: bool,
    enabled: bool,
}

impl Pulse {
    pub fn new (id: u8) -> Self {
        Self {
            id,
            output: 0,
            sequence: 0,
            length: 0,
            length_halt: false,
            timer: 0,
            timer_reload: 0,
            volume: 0,
            volume_constant: 15,
            envelope_start: false,
            envelope_timer: 0,
            envelope_timer_reload: 0,
            envelope_loop: false,
            envelope_enabled: false,
            sweep_timer: 0,
            sweep_negate: false,
            sweep_reload: 0,
            sweep_shift: 0,
            sweep_enabled: false,
            enabled: true,
        }
    }

    pub fn cycle_timer (&mut self) {
        if self.enabled {
            if self.timer == 0 {
                self.sequence = self.sequence.rotate_right(1);
                self.output = self.sequence & 1;
                self.timer = self.timer_reload;
            } else {
                self.timer -= 1;
            }
        }
    }
    
    pub fn cycle_length (&mut self) {
        if !self.length_halt && self.length > 0 {
            self.length -= 1;
        }
    }

    pub fn cycle_envelope (&mut self) {
        if self.envelope_start {
            self.volume = 15;
            self.envelope_timer = self.envelope_timer_reload;
            self.envelope_start = false;
        } else if self.envelope_timer == 0 {
            if self.volume > 0 {
                self.volume -= 1;
            } else if self.envelope_loop {
                self.volume = 15;
            }

            self.envelope_timer = self.envelope_timer_reload;
        } else {
            self.envelope_timer -= 1;
        }
    }

    pub fn cycle_sweep (&mut self) {
        if self.sweep_timer == 0 {
            if self.sweep_enabled {
                let amount = self.timer_reload >> self.sweep_shift;
    
                if self.sweep_negate {
                    self.timer_reload -= amount;
                    
                    if self.id == 1 {
                        self.timer_reload -= 1;
                    }
                } else {
                    self.timer_reload += amount;
                }
            }

            self.sweep_timer = self.sweep_reload;
        } else {
            self.sweep_timer -= 1;
        }
    }

    pub fn write_ctrl (&mut self, data: u8) {
        let duty = (data & 0b1100_0000) >> 6;

        self.sequence = PULSE_TABLE[duty as usize];
        self.length_halt = (data & 0b0010_0000) > 0;
        self.envelope_loop = !self.length_halt;
        self.envelope_enabled = (data & 0b0001_0000) == 0;
        self.envelope_timer_reload = data & 0b000_1111;
        self.volume_constant = data & 0b000_1111;
        self.envelope_start = true;
    }

    pub fn write_sweep (&mut self, data: u8) {
        self.sweep_enabled = (data & 0b1000_0000) > 0;
        self.sweep_reload = (data & 0b0111_0000) >> 4;
        self.sweep_negate = (data & 0b0000_1000) > 0;
        self.sweep_shift = data & 0b0000_0111;
        self.sweep_timer = self.sweep_reload;
    }

    pub fn write_lo (&mut self, data: u8) {
        self.timer_reload = (self.timer_reload & 0xFF00) | data as u16;
    }

    pub fn write_hi (&mut self, data: u8) {
        self.length = LENGTH_TABLE[((data & 0b1111_1000) >> 3) as usize];
        self.timer_reload = (self.timer_reload & 0x00FF) | (data as u16 & 0b0000_0111) << 8;
        self.timer = self.timer_reload;
        self.envelope_start = true;
    }

    pub fn output (&self) -> u8 {
        if !self.enabled || self.length == 0 || self.timer_reload < 8 || self.timer_reload > 0x7FF { // Max. 11 bits
            0
        } else {
            self.output * if self.envelope_enabled { self.volume } else { self.volume_constant }
        }
    }

    pub fn enable (&mut self) {
        self.enabled = true;
    }

    pub fn disable (&mut self) {
        self.enabled = false;
        self.length = 0;
    }
}
