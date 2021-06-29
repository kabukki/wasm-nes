use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use wasm_nes::cpu::cpu::Cpu;
use wasm_nes::cpu::memory::{Memory, MEMORY_CARTRIDGE_PRG_LOWER_START, MEMORY_CARTRIDGE_PRG_UPPER_START, MEMORY_CARTRIDGE_PRG_UPPER_SIZE};

/**
 * CPU state representation
 */
#[derive(PartialEq)]
struct State {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub sp: u8,
}

impl State {
    pub fn from_str (string: &String) -> State {
        let pc = &string[..4];
        let registers = &string[48..73];
        let (a, x, y, status, sp) = (&registers[2..4], &registers[7..9], &registers[12..14], &registers[17..19], &registers[23..25]);

        State {
            pc: u16::from_str_radix(pc, 16).unwrap(),
            a: u8::from_str_radix(a, 16).unwrap(),
            x: u8::from_str_radix(x, 16).unwrap(),
            y: u8::from_str_radix(y, 16).unwrap(),
            status: u8::from_str_radix(status, 16).unwrap(),
            sp: u8::from_str_radix(sp, 16).unwrap(),
        }
    }
}

fn main() {
    let rom: String = env::args().nth(1).expect("Missing mandatory ROM file");
    println!("--- Using ROM: {} ---", rom);

    let mut cpu = Cpu::new();
    let mut memory = Memory::new();

    // Load program into memory
    let file: Vec<u8> = File::open(rom).unwrap().bytes().map(|byte| byte.unwrap()).collect();
    let log: Vec<String> = BufReader::new(File::open("roms/nestest.log").unwrap()).lines().map(|line| line.unwrap()).collect();
    println!("ROM length: {} bytes", file.len());

    // Copy 0x4000 bytes into upper & lower ROM PRG (write twice while we don't have a mapper)
    for n in 0..MEMORY_CARTRIDGE_PRG_UPPER_SIZE {
        memory.write(MEMORY_CARTRIDGE_PRG_LOWER_START + n as u16, file[n + 0x10]);
        memory.write(MEMORY_CARTRIDGE_PRG_UPPER_START + n as u16, file[n + 0x10]);
    }

    cpu.pc = MEMORY_CARTRIDGE_PRG_UPPER_START;

    for n in 0..MEMORY_CARTRIDGE_PRG_UPPER_SIZE {
        let state = State::from_str(&log[n]);
        let pass = state.pc == cpu.pc && state.a == cpu.a && state.x == cpu.x && state.y == cpu.y && state.status == cpu.status && state.sp == cpu.sp;

        println!("EXPECTED  PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X}", state.pc, state.a, state.x, state.y, state.status, state.sp);
        println!("ACTUAL    PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X}", cpu.pc, cpu.a, cpu.x, cpu.y, cpu.status, cpu.sp);

        if pass {
            println!("✅ Logs match");
        } else {
            println!("❌ Logs differ");
            break;
        }

        cpu.cycle(&mut memory);
    }
}
