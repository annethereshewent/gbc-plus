use std::{env, fs};

use cpu::CPU;


pub mod cpu;


fn main() {
    let mut cpu = CPU::new();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("syntax: ./gbc-plus <rom name>");
    }

    let rom_path = &args[1];

    let rom_bytes = fs::read(rom_path).unwrap();

    cpu.load_rom(&rom_bytes);

    loop {
        cpu.step();
    }
}
