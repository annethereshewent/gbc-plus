use std::{
    collections::VecDeque,
    env,
    fs,
    io::Read,
    path::Path,
    sync::{Arc, Mutex}
};

extern crate gbc_plus;

use frontend::Frontend;
use gbc_plus::cpu::CPU;
use zip::ZipArchive;

pub mod frontend;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("syntax: ./gbc-plus <rom name>");
    }

    let mut rom_path = args[1].clone();

    let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));

    let mut rom_bytes = fs::read(rom_path.clone()).unwrap();

    if Path::new(&rom_path).extension().unwrap().to_os_string() == "zip" {
        let file = fs::File::open(rom_path.to_string()).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();

        let mut file_found = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            if file.is_file() {
                file_found = true;
                rom_bytes = vec![0; file.size() as usize];
                file.read_exact(&mut rom_bytes).unwrap();

                let real_file_name = file.name().to_string();

                let mut split_str: Vec<&str> = rom_path.split('/').collect();

                split_str.pop();

                split_str.push(&real_file_name);

                rom_path = split_str.join("/");

                break;
            }
        }

        if !file_found {
            panic!("couldn't extract ROM from zip file!");
        }
    }

    let mut cpu = CPU::new(audio_buffer.clone(), &rom_path.clone());

    let mut frontend = Frontend::new(&mut cpu, audio_buffer);

    cpu.load_rom(&rom_bytes);

    loop {
        while !cpu.bus.ppu.frame_finished {
            cpu.step();
        }

        frontend.update_rtc(&mut cpu);
        frontend.check_saves(&mut cpu);
        frontend.render_screen(&mut cpu);
        frontend.check_controller_status();
        frontend.handle_events(&mut cpu);
    }
}
