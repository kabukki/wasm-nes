/**
 * https://wiki.nesdev.org/w/index.php/Cycle_reference_chart
 */

pub const NTSC_CLOCK_MASTER: f64    = 21_477_272.0;
pub const NTSC_CLOCK_PPU: f64       = NTSC_CLOCK_MASTER / 4.0;
pub const NTSC_CLOCK_CPU: f64       = NTSC_CLOCK_MASTER / 12.0;

pub const PAL_CLOCK_MASTER: f64     = 26_601_712.5;
pub const PAL_CLOCK_PPU: f64        = PAL_CLOCK_MASTER / 5.0;
pub const PAL_CLOCK_CPU: f64        = PAL_CLOCK_MASTER / 16.0;

pub struct Clock {
    pub rate: f64,
    pub time: f64,
}

// IDEA: memoize step ?

impl Clock {
    pub fn new (rate: f64) -> Self {
        Self {
            rate,
            time: 0.0,
        }
    }

    pub fn tick (&mut self) {
        self.time += 1.0 / self.rate;
    }

    pub fn reset (&mut self) {
        self.time = 0.0;
    }
}

pub struct ClockDivider {
    pub rate: f64,
    pub cycles: usize,
}

impl ClockDivider {
    pub fn new (rate: f64) -> Self {
        Self {
            rate,
            cycles: 0,
        }
    }

    pub fn tick (&mut self, clock: f64) -> bool {
        let previous = self.cycles;
        self.cycles = (clock / (1.0 / self.rate)) as usize;
        self.cycles != previous
    }
}
