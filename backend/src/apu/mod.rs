mod apu;
mod pulse;

pub use apu::*;
pub use pulse::*;

/**
 * https://wiki.nesdev.org/w/index.php/APU_Length_Counter
 */
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6,
    160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];
