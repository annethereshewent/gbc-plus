use std::{collections::{HashMap, VecDeque}, env, fs, io::Read, path::Path, process::exit, sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

extern crate gbc_plus;

use gbc_plus::cpu::{
    bus::{
        joypad::JoypadButtons,
        ppu::{
            SCREEN_HEIGHT,
            SCREEN_WIDTH
        }
    },
    CPU
};
use sdl2::{
    audio::{
        AudioCallback,
        AudioSpecDesired
    },
    controller::GameController,
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    GameControllerSubsystem
};
use zip::ZipArchive;

const BUTTON_CROSS: u8 = 0;
const BUTTON_SQUARE: u8 = 2;
const BUTTON_SELECT: u8 = 4;
const BUTTON_START: u8 = 6;
const BUTTON_UP: u8 = 11;
const BUTTON_DOWN: u8 = 12;
const BUTTON_LEFT: u8 = 13;
const BUTTON_RIGHT: u8 = 14;

// TODO: move this stuff to a separate frontend struct.

pub struct GbcAudioCallback {
    pub audio_samples: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioCallback for GbcAudioCallback {
    type Channel = f32;

    fn callback(&mut self, buf: &mut [Self::Channel]) {
        let mut audio_samples = self.audio_samples.lock().unwrap();
        let len = audio_samples.len();

        let mut left_sample: f32 = 0.0;
        let mut right_sample: f32 = 0.0;

        if len > 2 {
            left_sample = audio_samples[len - 2];
            right_sample = audio_samples[len - 1];
        }

        let mut is_left_sample = true;

        for b in buf.iter_mut() {
            *b = if let Some(sample) = audio_samples.pop_front() {
                sample
            } else {
                if is_left_sample {
                    left_sample
                } else {
                    right_sample
                }
            };
            is_left_sample = !is_left_sample;
        }
    }
}

fn reconnect_controller(controller_subsystem: &GameControllerSubsystem, controller_id: u32, retry_attempts: &mut usize) -> Option<GameController> {
    if *retry_attempts < 5 {
        match controller_subsystem.open(controller_id) {
            Ok(c) => {
                Some(c)
            }
            Err(_) => {
                *retry_attempts += 1;
                None
            }
        }
    } else {
        None
    }
}

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("syntax: ./gbc-plus <rom name>");
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let game_controller_subsystem = sdl_context.game_controller().unwrap();

    let available = game_controller_subsystem
        .num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e)).unwrap();

    let mut _controller = (0..available)
        .find_map(|id| {
        match game_controller_subsystem.open(id) {
            Ok(c) => {
                Some(c)
            }
            Err(_) => {
                None
            }
        }
    });

    let audio_subsystem = sdl_context.audio().unwrap();

    let spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: Some(4096)
    };

    let audio_buffer = Arc::new(Mutex::new(VecDeque::<f32>::new()));

    let device = audio_subsystem.open_playback(
        None,
        &spec,
        |_| GbcAudioCallback { audio_samples: audio_buffer.clone() }
    ).unwrap();

    device.resume();

    let window = video_subsystem
        .window("GBC+", (SCREEN_WIDTH * 3) as u32, (SCREEN_HEIGHT * 3) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .unwrap();

    let rom_path = &args[1];

    let mut cpu = CPU::new(audio_buffer, rom_path);

    let mut rom_bytes = fs::read(rom_path).unwrap();

    let button_map = HashMap::from([
        (BUTTON_CROSS, JoypadButtons::A),
        (BUTTON_SQUARE, JoypadButtons::B),
        (BUTTON_SELECT, JoypadButtons::Select),
        (BUTTON_START, JoypadButtons::Start),
        (BUTTON_UP, JoypadButtons::Up),
        (BUTTON_DOWN, JoypadButtons::Down),
        (BUTTON_LEFT, JoypadButtons::Left),
        (BUTTON_RIGHT, JoypadButtons::Right)
    ]);

    let keyboard_map = HashMap::from([
            (Keycode::W, JoypadButtons::Up),
            (Keycode::S, JoypadButtons::Down),
            (Keycode::A, JoypadButtons::Left),
            (Keycode::D, JoypadButtons::Right),
            (Keycode::J, JoypadButtons::B),
            (Keycode::K, JoypadButtons::A),
            (Keycode::LShift, JoypadButtons::Select),
            (Keycode::Return, JoypadButtons::Start)
        ]
    );

    if Path::new(rom_path).extension().unwrap().to_os_string() == "zip" {
        let file = fs::File::open(rom_path).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();

        let mut file_found = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            if file.is_file() {
                file_found = true;
                rom_bytes = vec![0; file.size() as usize];
                file.read_exact(&mut rom_bytes).unwrap();
                break;
            }
        }

        if !file_found {
            panic!("couldn't extract ROM from zip file!");
        }
    }

    cpu.load_rom(&rom_bytes);

    let mut retry_attempts = 0;
    let mut controller_id: Option<u32> = None;

    loop {
        while !cpu.bus.ppu.frame_finished {
            cpu.step();
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();

        if let Some(mbc) = &mut cpu.bus.cartridge.mbc {
            let last_updated = mbc.backup_file().last_updated;
            if mbc.backup_file().is_dirty &&
                current_time > last_updated &&
                last_updated != 0
            {
                let diff = current_time - last_updated;
                if diff >= 500 {
                    mbc.save();
                }
            }
        }

        cpu.bus.ppu.cap_fps();

        cpu.bus.ppu.frame_finished = false;

        texture.update(None, &cpu.bus.ppu.picture.data, SCREEN_WIDTH as usize * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();

        canvas.present();

        if let Some(id) = controller_id {
            _controller = reconnect_controller(&game_controller_subsystem, id, &mut retry_attempts);

            if _controller.is_some() || retry_attempts >= 5 {
                controller_id = None
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    if let Some(mbc) = &mut cpu.bus.cartridge.mbc {
                        if mbc.backup_file().is_dirty {
                            mbc.save();
                        }
                    }
                    exit(0);
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = keyboard_map.get(&keycode) {
                            cpu.bus.joypad.press_button(*button);
                        } else if keycode == Keycode::G {
                            cpu.debug_on = !cpu.debug_on;
                        }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = keyboard_map.get(&keycode) {
                            cpu.bus.joypad.release_button(*button);
                        }
                    }
                }
                Event::JoyButtonDown { button_idx, .. } => {
                    if let Some(button) = button_map.get(&button_idx) {
                        cpu.bus.joypad.press_button(*button);
                    }
                }
                Event::JoyButtonUp { button_idx, .. } => {
                    if let Some(button) = button_map.get(&button_idx) {
                        cpu.bus.joypad.release_button(*button);
                    }
                }
                Event::JoyDeviceAdded { which, .. } => {
                    _controller = match game_controller_subsystem.open(which) {
                        Ok(c) => {
                            Some(c)
                        }
                        Err(_) => {
                            controller_id = Some(which);
                            None
                        }
                    }
                }
                _ => { /* do nothing */ }
            }
        }
    }
}
