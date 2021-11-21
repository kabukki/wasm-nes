use crate::{
    apu::pulse::Pulse,
    cpu::{Cpu, /* Interrupt */},
    clock::ClockDivider,
};

pub mod pulse;

/**
 * https://wiki.nesdev.org/w/index.php/APU_Length_Counter
 */
pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6,
    160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

pub enum StatusFlag {
    DMCInterrupt    = 0b1000_0000,
    FrameInterrupt  = 0b0100_0000,
    DMC             = 0b0001_0000,
    Noise           = 0b0000_1000,
    Triangle        = 0b0000_0100,
    Square2         = 0b0000_0010,
    Square1         = 0b0000_0001,
}

#[derive(PartialEq)]
pub enum FrameCounterMode {
    FourStep    = 0,
    FiveStep    = 1,
}

/**
 * https://wiki.nesdev.org/w/index.php/APU
 */
pub struct Apu {
    status: u8,
    mode: FrameCounterMode,
    irq_inhibit: bool,
    square_1: Pulse,
    square_2: Pulse,
    // triangle_1,
    // noise_1,
    // sample_1,
    buffer: Vec<f32>,
    frame: usize,
    pub clock: ClockDivider,
    pub clock_sample: ClockDivider,
}

impl Apu {
    pub fn new (sample_rate: f64) -> Self {
        Self {
            status: 0,
            mode: FrameCounterMode::FiveStep,
            irq_inhibit: false,
            square_1: Pulse::new(1),
            square_2: Pulse::new(2),
            buffer: vec![],
            frame: 0,
            clock: ClockDivider::new(crate::clock::NTSC_CLOCK_CPU),
            clock_sample: ClockDivider::new(sample_rate),
        }
    }

    pub fn tick (&mut self, time: f64, cpu: &mut Cpu) {
        if self.clock.tick(time) {
            self.cycle(cpu);
        }

        if self.clock_sample.tick(time) {
            self.sample();
        }
    }

    pub fn cycle (&mut self, cpu: &mut Cpu) {
        if self.clock.cycles % 2 == 0 {
            self.square_1.cycle_timer();
            self.square_2.cycle_timer();
            self.frame += 1;

            self.cycle_frame(cpu);
        }
    }

    /**
     * https://wiki.nesdev.org/w/index.php/APU_Frame_Counter
     */
    pub fn cycle_frame (&mut self, _cpu: &mut Cpu) {
        match self.mode {
            FrameCounterMode::FourStep => {
                match self.frame {
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
                        self.square_1.cycle_envelope();
                        self.square_1.cycle_length();
                        self.square_1.cycle_sweep();
                        self.square_2.cycle_envelope();
                        self.square_2.cycle_length();
                        self.square_2.cycle_sweep();
                        if !self.irq_inhibit {
                            self.status |= StatusFlag::FrameInterrupt as u8;
                            // cpu.interrupt_request(Interrupt::IRQ);
                        }
                        self.frame = 0;
                    },
                    _ => {},
                }
            },
            FrameCounterMode::FiveStep => {
                match self.frame {
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
                        self.square_1.cycle_envelope();
                        self.square_1.cycle_length();
                        self.square_1.cycle_sweep();
                        self.square_2.cycle_envelope();
                        self.square_2.cycle_length();
                        self.square_2.cycle_sweep();
                        self.frame = 0;
                    },
                    _ => {},
                }
            },
        }
    }

    pub fn reset (&mut self) {
        self.status = 0;
        self.frame = 0;
        self.write(0x4015, 0);
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
     * Collect a sound sample and append it to the buffer
     */
    pub fn sample (&mut self) {
        self.buffer.push(self.mix());
    }

    /**
     * Flush the sound sample buffer and returns its content
     */
    pub fn flush (&mut self) -> Vec<f32> {
        let buffer: Vec<f32> = self.buffer.to_vec();
        self.buffer.clear();
        buffer
    }

    pub fn read (&mut self, address: u16) -> u8 {
        match address {
            // Status
            0x4015 => {
                let status = (if self.square_1.length > 0 { 1 } else { 0 })
                    | (if self.square_2.length > 0 { 1 } else { 0 } << 1)
                    | (if (self.status & StatusFlag::FrameInterrupt as u8) > 0 { 1 } else { 0 } << 6);
                self.status &= !(StatusFlag::FrameInterrupt as u8);
                status
            },
            _ => panic!("Invalid APU read @ {:#x}", address),
        }
    }

    pub fn write (&mut self, address: u16, data: u8) {
        match address {
            // Pulse 1
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
            // Pulse 2
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
            // Status
            0x4015 => {
                if (data & StatusFlag::Square1 as u8) > 0 { self.square_1.enable(); } else { self.square_1.disable(); }
                if (data & StatusFlag::Square2 as u8) > 0 { self.square_2.enable(); } else { self.square_2.disable(); }
            },
            // Frame counter
            0x4017 => {
                self.mode = if (data & 0b1000_0000) > 0 { FrameCounterMode::FiveStep } else { FrameCounterMode::FourStep };
                self.irq_inhibit = (data & 0b0100_0000) > 0;

                if self.irq_inhibit {
                    self.status &= !(StatusFlag::FrameInterrupt as u8);
                }

                if self.mode == FrameCounterMode::FiveStep {
                    self.square_1.cycle_envelope();
                    self.square_1.cycle_length();
                    self.square_1.cycle_sweep();
                    self.square_2.cycle_envelope();
                    self.square_2.cycle_length();
                    self.square_2.cycle_sweep();
                }

                self.frame = 0;
            },
            _ => {}, // panic!("Invalid APU write @ {:#x}", address),
        }
    }
}
