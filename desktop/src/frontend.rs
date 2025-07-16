use std::{
    collections::{
        HashMap, VecDeque
    },
    fs::{
        self,
        File,
        OpenOptions
    },
    io::{
        Read,
        Seek,
        SeekFrom,
        Write
    },
    process::exit,
    sync::{
        Arc,
        Mutex
    },
    time::{
        SystemTime,
        UNIX_EPOCH
    }
};

use dirs_next::data_dir;
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
        AudioDevice,
        AudioSpecDesired
    }, controller::GameController, event::Event, keyboard::Keycode, pixels::PixelFormatEnum, render::Canvas, video::Window, EventPump, GameControllerSubsystem
};
use serde::{Deserialize, Serialize};

const BUTTON_CROSS: u8 = 0;
const BUTTON_SQUARE: u8 = 2;
const BUTTON_SELECT: u8 = 4;
const BUTTON_START: u8 = 6;
const BUTTON_UP: u8 = 11;
const BUTTON_DOWN: u8 = 12;
const BUTTON_LEFT: u8 = 13;
const BUTTON_RIGHT: u8 = 14;

#[derive(Serialize, Deserialize)]
struct EmuConfig {
    current_palette: usize
}

impl EmuConfig {
    pub fn new() -> Self {
        Self {
            current_palette: 1
        }
    }
}

pub struct Frontend {
    controller: Option<GameController>,
    canvas: Canvas<Window>,
    _device: AudioDevice<GbcAudioCallback>,
    event_pump: EventPump,
    button_map: HashMap<u8, JoypadButtons>,
    keyboard_map: HashMap<Keycode, JoypadButtons>,
    controller_id: Option<u32>,
    game_controller_subsystem: GameControllerSubsystem,
    retry_attempts: usize,
    config: EmuConfig,
    config_file: File,
    last_check: Option<u128>
}

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

impl Frontend {
    pub fn reconnect_controller(&mut self, controller_id: u32) -> Option<GameController> {
        if self.retry_attempts < 5 {
            match self.game_controller_subsystem.open(controller_id) {
                Ok(c) => {
                    Some(c)
                }
                Err(_) => {
                    self.retry_attempts += 1;
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn new(cpu: &mut CPU, audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let game_controller_subsystem = sdl_context.game_controller().unwrap();

        let available = game_controller_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {}", e)).unwrap();

        let controller = (0..available)
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
            freq: Some(48000),
            channels: Some(2),
            samples: Some(4096)
        };

        let _device = audio_subsystem.open_playback(
            None,
            &spec,
            |_| GbcAudioCallback { audio_samples: audio_buffer.clone() }
        ).unwrap();

        _device.resume();

        let window = video_subsystem
            .window("GBC+", (SCREEN_WIDTH * 3) as u32, (SCREEN_HEIGHT * 3) as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.set_scale(3.0, 3.0).unwrap();

        let event_pump = sdl_context.event_pump().unwrap();

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

        let mut config_path = data_dir().expect("Couldn't find application directory");

        config_path.push("GBC+");

        fs::create_dir_all(&config_path).expect("Couldn't find app directory");

        config_path.push("config.json");

        let mut config_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&config_path)
                .unwrap();

        let mut str: String = "".to_string();

        config_file.read_to_string(&mut str).unwrap();

        config_file.seek(SeekFrom::Start(0)).unwrap();

        let mut config = EmuConfig::new();

        match serde_json::from_str(&str) {
            Ok(config_json) => config = config_json,
            Err(_) => ()
        }

        cpu.bus.ppu.set_dmg_palette(config.current_palette);

        Self {
            controller,
            canvas,
            _device,
            event_pump,
            button_map,
            keyboard_map,
            controller_id: None,
            retry_attempts: 0,
            game_controller_subsystem,
            config,
            config_file,
            last_check: None
        }
    }

    pub fn render_screen(&mut self, cpu: &mut CPU) {
        cpu.bus.ppu.cap_fps();

        cpu.bus.ppu.frame_finished = false;

        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
            .unwrap();

        texture.update(None, &cpu.bus.ppu.picture.data, SCREEN_WIDTH as usize * 3).unwrap();

        self.canvas.copy(&texture, None, None).unwrap();

        self.canvas.present();
    }

    pub fn update_rtc(&mut self, cpu: &mut CPU) {

        if let Some(mbc) = &mut cpu.bus.cartridge.mbc {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("an error occurred")
                .as_millis();
            if let Some(last_check) = self.last_check {
                if current_time - last_check >= 1500 {
                    mbc.save_rtc();
                    self.last_check = None;
                }
            } else {
                self.last_check = Some(SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("an error occurred")
                    .as_millis());
            }
        }
    }

    pub fn check_saves(&mut self, cpu: &mut CPU) {
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
    }

    pub fn check_controller_status(&mut self) {
        if let Some(controller_id) = self.controller_id {
            self.controller = self.reconnect_controller(controller_id);

            if self.controller.is_some() || self.retry_attempts >= 5 {
                self.controller_id = None;
                self.retry_attempts = 0;
            }
        }
    }

    pub fn handle_events(&mut self, cpu: &mut CPU) {
        for event in self.event_pump.poll_iter() {
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
                        if let Some(button) = self.keyboard_map.get(&keycode) {
                            cpu.bus.joypad.press_button(*button);
                        } else if keycode == Keycode::G {

                            cpu.bus.ppu.debug_on = !cpu.bus.ppu.debug_on;
                            cpu.bus.debug_on = !cpu.bus.debug_on;
                            cpu.debug_on = !cpu.debug_on;
                        } else if keycode == Keycode::F2 {
                            cpu.bus.ppu.current_palette = (cpu.bus.ppu.current_palette + 1) % cpu.bus.ppu.palette_colors.len();

                            self.config.current_palette = cpu.bus.ppu.current_palette;

                            let json = match serde_json::to_string(&self.config) {
                                Ok(result) => result,
                                Err(_) => "".to_string()
                            };

                            if json != "" {
                                self.config_file.seek(SeekFrom::Start(0)).unwrap();
                                self.config_file.write_all(json.as_bytes()).unwrap();
                            }
                        }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(keycode) = keycode {
                        if let Some(button) = self.keyboard_map.get(&keycode) {
                            cpu.bus.joypad.release_button(*button);
                        }
                    }
                }
                Event::JoyButtonDown { button_idx, .. } => {
                    if let Some(button) = self.button_map.get(&button_idx) {
                        cpu.bus.joypad.press_button(*button);
                    }
                }
                Event::JoyButtonUp { button_idx, .. } => {
                    if let Some(button) = self.button_map.get(&button_idx) {
                        cpu.bus.joypad.release_button(*button);
                    }
                }
                Event::JoyDeviceAdded { which, .. } => {
                    self.controller = match self.game_controller_subsystem.open(which) {
                        Ok(c) => {
                            Some(c)
                        }
                        Err(_) => {
                            self.controller_id = Some(which);
                            self.retry_attempts = 0;
                            None
                        }
                    }
                }
                _ => { /* do nothing */ }
            }
        }
    }
}