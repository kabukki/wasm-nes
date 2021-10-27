use crate::apu::pulse::Pulse;

pub mod pulse;

/**
 * https://wiki.nesdev.org/w/index.php/APU_Length_Counter
 */
pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

/**
 * https://wiki.nesdev.org/w/index.php/APU
 */
pub struct Apu {
    mode: u8,
    square_1: Pulse,
    square_2: Pulse,
    // triangle_1,
    // noise_1,
    // sample_1,
    buffer: Vec<f32>,
    cycles: usize,
}

impl Apu {
    pub fn new () -> Self {
        Self {
            cycles: 0,
            mode: 0,
            square_1: Pulse::new(1),
            square_2: Pulse::new(2),
            buffer: vec![],
        }
    }

    pub fn cycle (&mut self) {
        self.square_1.cycle_timer();
        self.square_2.cycle_timer();

        self.cycles += 1;
        
        // https://wiki.nesdev.org/w/index.php/APU_Frame_Counter
        match self.mode {
            0 => {
                match self.cycles {
                    3729 => {
                        self.square_1.cycle_envelope();
                        self.square_2.cycle_envelope();
                    },
                    7457 => {
                        self.square_1.cycle_envelope();
                        self.square_1.cycle_length();
                        self.square_1.cycle_sweep();
                        self.square_2.cycle_envelope();
                        self.square_2.cycle_length();
                        self.square_2.cycle_sweep();
                    },
                    11186 => {
                        self.square_1.cycle_envelope();
                        self.square_2.cycle_envelope();
                    },
                    14915 => {
                        // e,l,irq
                        self.square_1.cycle_envelope();
                        self.square_1.cycle_length();
                        self.square_1.cycle_sweep();
                        self.square_2.cycle_envelope();
                        self.square_2.cycle_length();
                        self.square_2.cycle_sweep();
                        self.cycles = 0;
                    },
                    _ => {},
                }
            },
            1 => {
                match self.cycles {
                    3729 => {
                        self.square_1.cycle_envelope();
                        self.square_2.cycle_envelope();
                    },
                    7457 => {
                        self.square_1.cycle_envelope();
                        self.square_1.cycle_length();
                        self.square_1.cycle_sweep();
                        self.square_2.cycle_envelope();
                        self.square_2.cycle_length();
                        self.square_2.cycle_sweep();
                    },
                    11186 => {
                        self.square_1.cycle_envelope();
                        self.square_2.cycle_envelope();
                    },
                    18641 => {
                        // e,l,irq
                        self.square_1.cycle_envelope();
                        self.square_1.cycle_length();
                        self.square_1.cycle_sweep();
                        self.square_2.cycle_envelope();
                        self.square_2.cycle_length();
                        self.square_2.cycle_sweep();
                        self.cycles = 0;
                    },
                    _ => {},
                }
            },
            _ => unreachable!(),
        }

        self.buffer.push(self.mix());
    }
    
    /**
     * https://wiki.nesdev.org/w/index.php/APU_Mixer
     */
    pub fn mix (&self) -> f32 {
        let pulse_output = 95.88 / (8128.0 / (self.square_1.output() + self.square_2.output()) as f32 + 100.0);
        let tnd_output = 0.0;
        pulse_output + tnd_output
    }

    /**
     * Extract the sample by applying a downsampling factor.
     * The resulting output will contain (source.length / factor) samples.
     * For simplicity, sample rate conversion is not done here.
     */
    pub fn flush (&mut self, factor: f32) -> Vec<f32> {
        let buffer: Vec<f32> = self.buffer.iter().step_by(factor as usize).copied().collect();
        self.buffer.clear();
        buffer
    }

    pub fn read (&self, address: u16) -> u8 {
        match address {
            0x4015 => 0,
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
                if (data & 1) > 0 { self.square_1.enable(); } else { self.square_1.disable(); }
                if (data & 2) > 0 { self.square_2.enable(); } else { self.square_2.disable(); }
            },
            0x4017 => {},
            _ => {}, // panic!("Invalid APU write @ {:#x}", address),
        }
    }
}
