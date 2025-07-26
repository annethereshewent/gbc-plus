use std::{
    env,
    fs,
    io::Read,
    path::Path
};

extern crate gbc_plus;

use frontend::Frontend;
use gbc_plus::cpu::{bus::apu::NUM_SAMPLES, CPU};
use ringbuf::{traits::Split, HeapRb};
use zip::ZipArchive;

pub mod frontend;
pub mod cloud_service;

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("syntax: ./gbc-plus <rom name>");
    }

    let mut rom_path = args[1].clone();

    // let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));
    let ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

    let (producer, consumer) = ringbuffer.split();

    let waveform_ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

    let (waveform_producer, waveform_consumer) = waveform_ringbuffer.split();

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

    let mut split_vec: Vec<&str> = rom_path.split('.').collect();

    // remove the extension
    split_vec.pop();

    let filename = format!("{}.sav", split_vec.join("."));

    split_vec = filename.split('/').collect();

    let save_name = split_vec.pop().unwrap();

    let mut cpu = CPU::new(producer, Some(waveform_producer), Some(filename.clone()), false, true);

    let mut frontend = Frontend::new(&mut cpu, consumer, waveform_consumer, save_name.to_string());

    cpu.load_rom(&rom_bytes);

    let cloud_service_clone = frontend.cloud_service.clone();

    let logged_in = {
        cloud_service_clone.lock().unwrap().logged_in
    };

    if logged_in {
        cpu.bus.cartridge.set_save_file(None);

        let data = frontend.cloud_service.lock().unwrap().get_save();

        if data.len() > 0 {
            cpu.bus.cartridge.load_save(&data);
        }
    }

    loop {
        while !cpu.bus.ppu.frame_finished {
            cpu.step();
        }

        frontend.clear_framebuffer();

        frontend.update_rtc(&mut cpu);
        frontend.check_saves(&mut cpu, logged_in);
        frontend.render_screen(&mut cpu);
        frontend.render_ui(&mut cpu, logged_in);
        frontend.check_controller_status();
        frontend.end_frame();

        frontend.handle_events(&mut cpu, logged_in);

        if frontend.show_waveform {
            frontend.plot_waveform();
        }

    }
}
