use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use wasm_nes::{cpu::Cpu, bus::Bus};

/**
 * CPU state representation
 */
#[derive(PartialEq)]
pub struct State {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: u8,
    pub sp: u8,
    pub cycles: usize,
}

impl State {
    pub fn from_str (string: &String) -> State {
        let pc = &string[..4];
        let registers = &string[48..73];
        let (a, x, y, status, sp, cycles) = (
            &registers[2..4],
            &registers[7..9],
            &registers[12..14],
            &registers[17..19],
            &registers[23..25],
            &string[90..],
        );

        State {
            pc: u16::from_str_radix(pc, 16).unwrap(),
            a: u8::from_str_radix(a, 16).unwrap(),
            x: u8::from_str_radix(x, 16).unwrap(),
            y: u8::from_str_radix(y, 16).unwrap(),
            status: u8::from_str_radix(status, 16).unwrap(),
            sp: u8::from_str_radix(sp, 16).unwrap(),
            cycles: usize::from_str_radix(cycles, 10).unwrap(),
        }
    }

    // pub fn to_str
}

#[test]
fn nestest () {
    let mut cpu = Cpu::new();
    let mut bus = Bus::new();

    // Load program into memory
    let rom = File::open("tests/roms/cpu/nestest/nestest.nes").expect("Could not open rom").bytes().map(|byte| byte.unwrap()).collect();
    let log: Vec<String> = BufReader::new(File::open("tests/roms/cpu/nestest/nestest.log").expect("Could not open log")).lines().map(|line| line.unwrap()).collect();
    
    bus.load(&rom);
    cpu.reset();
    cpu.cycle_full(&mut bus);
    cpu.pc = 0xC000;

    for n in 0..log.len() {
        let state = State::from_str(&log[n]);

        println!("{} EXPECTED  PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X} PPU:---,--- CYC:{}", n + 1, state.pc, state.a, state.x, state.y, state.status, state.sp, state.cycles);
        println!("{} ACTUAL    PC:{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:08b} SP:{:02X} PPU:---,--- CYC:{}", n + 1, cpu.pc, cpu.a, cpu.x, cpu.y, cpu.status, cpu.sp, cpu.cycles_total);

        assert_eq!(state.pc, cpu.pc, "PC differ");
        assert_eq!(state.a, cpu.a, "A registers differ");
        assert_eq!(state.x, cpu.x, "X registers differ");
        assert_eq!(state.y, cpu.y, "Y registers differ");
        assert_eq!(state.status, cpu.status, "Status registers differ");
        assert_eq!(state.sp, cpu.sp, "Stack pointers differ");
        assert_eq!(state.cycles, cpu.cycles_total, "Cycles differ");

        cpu.cycle_full(&mut bus);
    }    
}
