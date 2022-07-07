/**
 * https://wiki.nesdev.org/w/index.php/Cycle_reference_chart
 * IDEA: memoize step ?
 */

pub const CLOCK_MASTER_NTSC: f64    = 21_477_272.0;
pub const CLOCK_PPU_NTSC: f64       = CLOCK_MASTER_NTSC / 4.0;
pub const CLOCK_CPU_NTSC: f64       = CLOCK_MASTER_NTSC / 12.0;

pub const CLOCK_MASTER_PAL: f64     = 26_601_712.5;
pub const CLOCK_PPU_PAL: f64        = CLOCK_MASTER_PAL / 5.0;
pub const CLOCK_CPU_PAL: f64        = CLOCK_MASTER_PAL / 16.0;

/// A structure that holds the emulated time passed.
#[derive(Copy, Clone, serde::Serialize)]
pub struct Clock {
    pub rate: f64,
    pub time: f64,
}

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

/// A structure that represents passed time and is cycle-aware
#[derive(Copy, Clone, serde::Serialize)]
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
