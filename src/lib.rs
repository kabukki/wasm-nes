/*
    The NES's master clock frequency is 21.477272 Mhz.
    The CPU divides it by 12, hence runs at 1.7897727 Mhz.
    The PPU divides it by 4, hence runs at 5.369318 Mhz.
    The APU divides it by 89490, hence runs at 239.996335 Hz.
    Since 12 / 4 = 3 there are 3 PPU clocks per 1 CPU clock.
    Since 89490 / 12 = 7457.5 there are 7457.5 CPU clocks per 1 APU clock.
*/

pub mod cpu {
    pub mod cpu;
    pub mod instruction;
    pub mod interrupt;
    pub mod memory;
}

// use crate::cpu::cpu::Cpu;
// use crate::cpu::memory::Memory;

// pub struct Nes {
//     cpu: Cpu,
//     memory: Memory,
//     // ram,
//     // ppu,
// }

// impl Nes {
//     fn load_rom (&mut self, rom: &[u8]) {
//         // self.memory.rom = [];
//     }
// }