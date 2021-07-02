/*
    The NES's master clock frequency is 21.477272 Mhz.
    The CPU divides it by 12, hence runs at 1.7897727 Mhz.
    The PPU divides it by 4, hence runs at 5.369318 Mhz (3x CPU).
    The APU divides it by 89490, hence runs at 239.996335 Hz.
    Since 12 / 4 = 3 there are 3 PPU clocks per 1 CPU clock.
    Since 89490 / 12 = 7457.5 there are 7457.5 CPU clocks per 1 APU clock.
*/

use crate::cpu::cpu::Cpu;
use crate::cpu::memory::{Memory, MEMORY_CARTRIDGE_PRG_LOWER_START, MEMORY_CARTRIDGE_PRG_UPPER_START, CARTRIDGE_BANK_SIZE};
use crate::ppu::ppu::Ppu;

pub mod cpu {
    pub mod cpu;
    pub mod instruction;
    pub mod interrupt;
    pub mod memory;
}

pub mod ppu {
    pub mod ppu;
}

pub struct Nes {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub memory: Memory,
}

impl Nes {
    pub fn new () -> Nes {
        return Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            memory: Memory::new(),
        };
    }

    /**
     * Load a ROM
     */
    pub fn load (&mut self, rom: Vec<u8>) {
        // Copy 0x4000 bytes into upper & lower ROM PRG (write twice while we don't have a mapper)
        for n in 0..CARTRIDGE_BANK_SIZE {
            self.memory.write(MEMORY_CARTRIDGE_PRG_LOWER_START + n as u16, rom[n + 0x10]);
            self.memory.write(MEMORY_CARTRIDGE_PRG_UPPER_START + n as u16, rom[n + 0x10]);
        }

        self.cpu.pc = MEMORY_CARTRIDGE_PRG_UPPER_START;
    }

    pub fn cycle (&mut self) {
        self.cpu.cycle(&mut self.memory);
    }
}
