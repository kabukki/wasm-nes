use log::info;

/**
 * https://wiki.nesdev.org/w/index.php/APU_Pulse
 */
#[derive(Default)]
struct Pulse {
    output: u8,
    sequence: u8,
    enabled: bool,
    // Divider
    reload: u16,
    counter: u16,
}

impl Pulse {
    pub fn cycle (&mut self) {
        if self.counter == 0 {
            self.counter = self.reload;
        } else {
            self.output = self.output.rotate_right(1);
            self.output = self.sequence & 1;
            self.counter -= 1;
        }
    }

    pub fn write_ctrl (&mut self, data: u8) {
        let duty = (data & 0b1100_0000) >> 6;

        self.sequence = match duty {
            0 => 0b0000_0001, // -_______ 12.5%
            1 => 0b0000_0011, // -______- 25%
            2 => 0b0000_1111, // -____--- 50%
            3 => 0b1111_1100, // _------_ 25%
            _ => unreachable!(),
        };
    }

    pub fn write_sweep (&mut self, _data: u8) {}

    pub fn write_lo (&mut self, data: u8) {
        self.reload = (self.reload & 0xFF00) | data as u16;
    }
    
    pub fn write_hi (&mut self, data: u8) {
        // self.load = (data & 0b1111_1000) >> 3;
        self.reload = (self.reload & 0x00FF) | (data as u16 & 0b0000_0111) << 8;
        self.reload();
    }

    pub fn reload (&mut self) {
        self.counter = self.reload;
    }
}

/**
 * https://wiki.nesdev.org/w/index.php/APU
 */
pub struct Apu {
    pub frame: usize,
    square_1: Pulse,
    square_2: Pulse,
    // square_2,
    // triangle_1,
    // noise_1,
    // sample_1,
    pub sample: u8,
}

impl Apu {
    pub fn new () -> Self {
        Self {
            frame: 0,
            square_1: Pulse::default(),
            square_2: Pulse::default(),
            sample: 0,
        }
    }

    pub fn cycle (&mut self) {
        if self.square_1.enabled {
            self.square_1.cycle();
        }
        // self.square_2.cycle();
        self.sample = self.square_1.output + self.square_2.output;

        // TODO half and quarter frames https://wiki.nesdev.org/w/index.php/APU_Frame_Counter
        self.frame += 1;
    }

    pub fn read (&self, address: u16) -> u8 {
        match address {
            _ => panic!("Invalid APU read @ {:#x}", address),
        }
    }

    pub fn write (&mut self, address: u16, data: u8) {
        match address {
            0x4000 => {
                self.square_1.write_ctrl(data);
            },
            0x4001 => {
                self.square_1.write_sweep(data);
            }
            0x4002 => {
                self.square_1.write_lo(data);
            },
            0x4003 => {
                self.square_1.write_hi(data);
            },
            0x4004 => {
                self.square_2.write_ctrl(data);
            },
            0x4005 => {
                self.square_2.write_sweep(data);
            }
            0x4006 => {
                self.square_2.write_lo(data);
            },
            0x4007 => {
                self.square_2.write_hi(data);
            },
            0x4015 => {
                info!("4015 write {:08b}", data);
                self.square_1.enabled = (data & 1) > 0;
                self.square_2.enabled = ((data & 2) >> 1) > 0;
            },
            _ => {}, // panic!("Invalid APU write @ {:#x}", address),
        }
    }
}
