use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use wasm_nes::Nes;
use wasm_nes::cpu::memory::CARTRIDGE_BANK_SIZE;
use wasm_nes::cpu::instruction;

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
    let (total, implemented) = instruction::coverage();
    println!("CPU ASI coverage: {} / {} legal", implemented, total);

    let rom: String = env::args().nth(1).expect("Missing mandatory ROM file");
    println!("--- Using ROM: {} ---", rom);

    let mut nes = Nes::new();

    // Load program into memory
    let file: Vec<u8> = File::open(rom).unwrap().bytes().map(|byte| byte.unwrap()).collect();
    let log: Vec<String> = BufReader::new(File::open("roms/nestest.log").unwrap()).lines().map(|line| line.unwrap()).collect();
    println!("ROM length: {} bytes", file.len());

    nes.load(file);
    // nes.memory.write(0x2000, CtrlFlag::Sprite)

    for n in 0..CARTRIDGE_BANK_SIZE {
        // let state = State::from_str(&log[n]);
        // let pass = state.pc == nes.cpu.pc && state.a == nes.cpu.a && state.x == nes.cpu.x && state.y == nes.cpu.y && state.status == nes.cpu.status && state.sp == nes.cpu.sp;

        // println!("EXPECTED  PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X}", state.pc, state.a, state.x, state.y, state.status, state.sp);
        // println!("ACTUAL    PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X}", nes.cpu.pc, nes.cpu.a, nes.cpu.x, nes.cpu.y, nes.cpu.status, nes.cpu.sp);

        // if pass {
        //     // println!("✅ Logs match");
        // } else {
        //     println!("❌ Logs differ on line {}", n + 1);
        //     break;
        // }

        nes.cycle();
    }
}
