use std::{
    collections::HashMap, fs::{
        self,
        File,
        OpenOptions
    }, io::{
        Read,
        Seek,
        SeekFrom,
        Write
    }, path::PathBuf,
    process::exit,
    sync::{
        Arc,
        Mutex
    },
    thread,
    time::{
        SystemTime,
        UNIX_EPOCH
    }
};
use chrono::{Local, NaiveDateTime};
use glow::RGBA;
use imgui_glow_renderer::{
  glow::{
    HasContext,
    NativeTexture,
    PixelUnpackData,
    COLOR_ATTACHMENT0,
    COLOR_BUFFER_BIT,
    NEAREST,
    READ_FRAMEBUFFER,
    RGBA8,
    TEXTURE_2D,
    TEXTURE_MAG_FILTER,
    TEXTURE_MIN_FILTER,
    UNSIGNED_BYTE
  },
  Renderer
};
use dirs_next::data_dir;
use gbc_plus::cpu::{
    bus::{
        apu::NUM_SAMPLES,
        cartridge::mbc::{mbc3::RtcFile, MBC},
        joypad::JoypadButtons,
        ppu::{
            SCREEN_HEIGHT,
            SCREEN_WIDTH
        }
    },
    CPU
};
use imgui_sdl2_support::SdlPlatform;
use native_dialog::FileDialog;
use num_enum::TryFromPrimitive;
use ringbuf::{
    storage::Heap, traits::{
        Consumer,
        Observer, Split,
    }, wrap::caching::Caching, HeapRb, SharedRb
};
use sdl2::{
    audio::{
        AudioCallback,
        AudioDevice,
        AudioSpecDesired
    },
    controller::GameController,
    event::{
        Event,
        WindowEvent
    },
    keyboard::Keycode,
    pixels::Color,
    render::Canvas,
    video::{GLContext, GLProfile, Window},
    EventPump,
    GameControllerSubsystem
};
use serde::{Deserialize, Serialize};
use imgui::{Context, Textures};
use zip::ZipArchive;

use crate::cloud_service::CloudService;

#[derive(Debug, Copy, Clone, TryFromPrimitive, Serialize, Deserialize)]
#[repr(u8)]
enum ButtonIndex {
    Cross = 0,
    Circle = 1,
    Square = 2,
    Triangle = 3,
    Select = 4,
    Home = 5,
    Start = 6,
    LeftThumbstick = 7,
    RightThumbstick = 8,
    L1 = 9,
    R1 = 10,
    Up = 11,
    Down = 12,
    Left = 13,
    Right = 14,
    Touchpad = 15
}

impl ButtonIndex {
    pub fn to_string(&self) -> String {
        let mut val = format!("{:?}", self);

        if val == "LeftThumbstick" {
            val = "Left thumbstick".to_string();
        } else if val == "RightThumbstick" {
            val = "Right thumbstick".to_string();
        }

        val
    }
}

const WAVEFORM_LENGTH: usize = 683;
const WAVEFORM_HEIGHT: usize = 256;

const THEME_NAMES: [&str; 10] = [
    "Classic green",
    "Grayscale",
    "Solarized",
    "Maverick",
    "Oceanic",
    "Burnt peach",
    "Grape soda",
    "Strawberry milk",
    "Witching hour",
    "Void dream"
];

const ACTIONS: [&str; 8] = [
    "Up",
    "Down",
    "Left",
    "Right",
    "Select",
    "Start",
    "B",
    "A"
];

const JOY_ACTIONS: [&str; 4] = [
    "Select",
    "Start",
    "B",
    "A"
];

#[derive(Serialize, Deserialize)]
struct EmuConfig {
    current_palette: usize,
    button_map: HashMap<u8, JoypadButtons>,
    button_to_keys: HashMap<JoypadButtons, String>,
    button_to_index: HashMap<JoypadButtons, ButtonIndex>,
    keyboard_map: HashMap<String, JoypadButtons>,
}

impl EmuConfig {
    pub fn new() -> Self {
        Self {
            current_palette: 1,
            button_map: HashMap::new(),
            keyboard_map: HashMap::new(),
            button_to_index: HashMap::new(),
            button_to_keys: HashMap::new()
        }
    }
}

pub struct Frontend {
    controller: Option<GameController>,
    device: AudioDevice<GbcAudioCallback>,
    event_pump: EventPump,
    button_map: HashMap<u8, JoypadButtons>,
    keyboard_map: HashMap<Keycode, JoypadButtons>,
    button_to_keycode: HashMap<JoypadButtons, Keycode>,
    controller_id: Option<u32>,
    game_controller_subsystem: GameControllerSubsystem,
    retry_attempts: usize,
    config: EmuConfig,
    config_file: File,
    gl: imgui_glow_renderer::glow::Context,
    texture: NativeTexture,
    platform: SdlPlatform,
    window: Window,
    renderer: Renderer,
    imgui: imgui::Context,
    textures: Textures<NativeTexture>,
    _gl_context: GLContext,
    waveform_canvas: Canvas<Window>,
    pub show_waveform: bool,
    wave_consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
    samples: Vec<f32>,
    pub cloud_service: Arc<Mutex<CloudService>>,
    display_ui: bool,
    file_to_delete: Option<PathBuf>,
    confirm_delete_dialog: bool,
    show_palette_picker_popup: bool,
    last_check: Option<u128>,
    show_bindings_popup: bool,
    ctrl_strs_to_buttons: HashMap<String, JoypadButtons>,
    current_input: Option<JoypadButtons>,
    current_joy_input: Option<JoypadButtons>,
    current_action: Option<usize>,
    button_to_index: HashMap<JoypadButtons, ButtonIndex>
}

pub struct GbcAudioCallback {
    pub consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>
}

impl AudioCallback for GbcAudioCallback {
    type Channel = f32;

    fn callback(&mut self, buf: &mut [Self::Channel]) {
        let mut left_sample: f32 = 0.0;
        let mut right_sample: f32 = 0.0;

        if self.consumer.vacant_len() > 2 {
            left_sample = *self.consumer.try_peek().unwrap_or(&0.0);
            right_sample = *self.consumer.try_peek().unwrap_or(&0.0);
        }

        let mut is_left_sample = true;

        for b in buf.iter_mut() {
            *b = if let Some(sample) = self.consumer.try_pop() {
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
    fn glow_context(window: &Window) -> imgui_glow_renderer::glow::Context {
        unsafe {
            imgui_glow_renderer::glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
        }
    }

    pub fn process_save_states<F>(game_path: String, mut callback: F)
        where F: FnMut(String, PathBuf)
    {
        let mut dir = data_dir().unwrap();

        dir.push("GBC+");

        let mut split: Vec<&str> = game_path.split('/').collect();

        let game_name = split.pop().unwrap();

        dir.push(&game_name);

        fs::create_dir_all(&dir).expect("Couldn't create save state directory");

        let paths = fs::read_dir(&dir).unwrap();

        let mut files: Vec<String> = Vec::new();

        for path in paths {
            let result_path = path.unwrap();

            if let Some(extension) = result_path.path().extension() {
                if let Some(extension_str) = extension.to_str() {
                    if extension_str == "state" {
                        files.push(result_path.file_name().to_str().unwrap().to_string());
                    }
                }
            }
        }

        for file in files {
            let mut dir = data_dir().unwrap();

            dir.push("GBC+");
            dir.push(&game_name);
            dir.push(&file);

            let mut split: Vec<&str> = file.split('.').collect();

            split.pop();

            let filename = split.pop().unwrap().to_string();

            let filename = if filename == "quick_save" {
                "Quick save".to_string()
            } else {
                split = filename.split('_').collect();

                let date_str = split.pop().unwrap();

                let date = NaiveDateTime::parse_from_str(date_str, "%Y%m%d%H%M%S").unwrap();

                format!("Save on {}", date.format("%m-%d-%Y %H:%M:%S"))
            };

            callback(filename, dir);
        }
    }

    pub fn plot_waveform(&mut self) {
        let mut new_samples: Vec<f32> = self.wave_consumer.pop_iter().collect();
        self.samples.append(&mut new_samples);

        while self.samples.len() >= WAVEFORM_LENGTH * 2 {
            self.samples.remove(0);
        }

        self.waveform_canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.waveform_canvas.clear();

        self.waveform_canvas.set_draw_color(Color::RGB(0, 0xff, 0));

        for x in (0..self.samples.len()).step_by(2) {
            let y1 = self.samples[x];
            let y2 = if x + 1 < self.samples.len() {
                self.samples[x + 1]
            } else {
                break;
            };

            let real_y1 = WAVEFORM_HEIGHT as f32 / 2.0 - (y1 * WAVEFORM_HEIGHT as f32) / 2.0;
            let real_y2 = WAVEFORM_HEIGHT as f32 / 2.0 - (y2 * WAVEFORM_HEIGHT as f32) / 2.0;

            let _ = self.waveform_canvas.draw_line((x as i32 / 2, real_y1 as i32), ((x as i32 + 1) / 2, real_y2 as i32)).unwrap();
        }

        self.waveform_canvas.present();
    }

    pub fn new(
        cpu: &mut CPU,
        consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
        wave_consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
        game_name: String
    ) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let game_controller_subsystem = sdl_context.game_controller().unwrap();

        let available = game_controller_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {}", e)).unwrap();

        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_profile(GLProfile::Core);

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
            freq: Some(44100),
            channels: Some(2),
            samples: Some(4096)
        };

        let device = audio_subsystem.open_playback(
            None,
            &spec,
            |_| GbcAudioCallback { consumer }
        ).unwrap();

        device.resume();

        let window = video_subsystem
            .window("GBC+", (SCREEN_WIDTH * 3) as u32, (SCREEN_HEIGHT * 3) as u32)
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let window_positon = window.position();

        let mut waveform_window = video_subsystem
            .window("Waveform Viewer", 683, 256)
            .position((window_positon.0 as f32 * 1.70) as i32, window_positon.1)
            .build()
            .unwrap();

        waveform_window.hide();

        let mut canvas = waveform_window.into_canvas().software().present_vsync().build().unwrap();
        canvas.set_scale(1.0, 1.0).unwrap();


        let gl_context = window.gl_create_context().unwrap();

        window.gl_make_current(&gl_context).unwrap();

        window.subsystem().gl_set_swap_interval(1).unwrap();

        let gl = Self::glow_context(&window);

        let texture = unsafe { gl.create_texture().unwrap() };
        let framebuffer = unsafe { gl.create_framebuffer().unwrap() };

        let mut textures = Textures::<NativeTexture>::new();

        unsafe {
            gl.bind_texture(TEXTURE_2D, Some(texture));

            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as i32);
            gl.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as i32);

            gl.tex_storage_2d(
                TEXTURE_2D,
                1,
                RGBA8,
                SCREEN_WIDTH as i32 * 2,
                SCREEN_HEIGHT as i32 * 2
            );

            gl.bind_framebuffer(READ_FRAMEBUFFER, Some(framebuffer));
            gl.framebuffer_texture_2d(
                READ_FRAMEBUFFER,
                COLOR_ATTACHMENT0,
                TEXTURE_2D,
                Some(texture),
                0
            );

            gl.clear_color(0.0, 0.0, 0.0, 0.0);
        }

        let mut imgui = Context::create();

        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);

        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let renderer = Renderer::new(
            &gl,
            &mut imgui,
            &mut textures,
            false
        ).unwrap();

        let platform = SdlPlatform::new(&mut imgui);

        let event_pump = sdl_context.event_pump().unwrap();

        let mut button_map = HashMap::from([
            (ButtonIndex::Cross as u8, JoypadButtons::A),
            (ButtonIndex::Square as u8, JoypadButtons::B),
            (ButtonIndex::Select as u8, JoypadButtons::Select),
            (ButtonIndex::Start as u8, JoypadButtons::Start),
            (ButtonIndex::Up as u8, JoypadButtons::Up),
            (ButtonIndex::Down as u8, JoypadButtons::Down),
            (ButtonIndex::Left as u8, JoypadButtons::Left),
            (ButtonIndex::Right as u8, JoypadButtons::Right)
        ]);

        let mut button_to_index = HashMap::from([
            (JoypadButtons::Up, ButtonIndex::Up),
            (JoypadButtons::Down, ButtonIndex::Down),
            (JoypadButtons::Left, ButtonIndex::Left),
            (JoypadButtons::Right, ButtonIndex::Right),
            (JoypadButtons::Select, ButtonIndex::Select),
            (JoypadButtons::Start, ButtonIndex::Start),
            (JoypadButtons::A, ButtonIndex::Cross),
            (JoypadButtons::B, ButtonIndex::Square)

        ]);

        let mut button_to_keys = HashMap::from([
            (JoypadButtons::Up, Keycode::W),
            (JoypadButtons::Down, Keycode::S),
            (JoypadButtons::Left, Keycode::A),
            (JoypadButtons::Right, Keycode::D),
            (JoypadButtons::Select, Keycode::Tab),
            (JoypadButtons::Start, Keycode::Return),
            (JoypadButtons::A, Keycode::K),
            (JoypadButtons::B, Keycode::J)
        ]);

        let mut keyboard_map = HashMap::from([
            (Keycode::W, JoypadButtons::Up),
            (Keycode::S, JoypadButtons::Down),
            (Keycode::A, JoypadButtons::Left),
            (Keycode::D, JoypadButtons::Right),
            (Keycode::J, JoypadButtons::B),
            (Keycode::K, JoypadButtons::A),
            (Keycode::Tab, JoypadButtons::Select),
            (Keycode::Return, JoypadButtons::Start)
        ]);

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

        if config.button_map.len() == 8 {
            button_map = config.button_map.clone();
            button_to_index = config.button_to_index.clone();

        }
        if config.keyboard_map.len() == 8 {
            let mut keyboard_map_clone = HashMap::<Keycode, JoypadButtons>::new();
            let mut button_to_keys_clone = HashMap::<JoypadButtons, Keycode>::new();

            for (key, value) in config.keyboard_map.clone() {
                keyboard_map_clone.insert(Keycode::from_name(&key).unwrap(), value);
                button_to_keys_clone.insert(value, Keycode::from_name(&key).unwrap());
            }

            keyboard_map = keyboard_map_clone;
            button_to_keys = button_to_keys_clone;
        }

        cpu.bus.ppu.set_dmg_palette(config.current_palette);

        Self {
            controller,
            device,
            event_pump,
            button_map,
            keyboard_map,
            controller_id: None,
            retry_attempts: 0,
            game_controller_subsystem,
            config,
            config_file,
            last_check: None,
            gl,
            texture,
            platform,
            window,
            renderer,
            imgui,
            textures,
            _gl_context: gl_context,
            waveform_canvas: canvas,
            show_waveform: false,
            wave_consumer,
            samples: Vec::with_capacity(NUM_SAMPLES),
            cloud_service: Arc::new(Mutex::new(CloudService::new(game_name))),
            display_ui: true,
            file_to_delete: None,
            confirm_delete_dialog: false,
            show_palette_picker_popup: false,
            show_bindings_popup: false,
            ctrl_strs_to_buttons:  HashMap::from([
                ("Up".to_string(), JoypadButtons::Up),
                ("Down".to_string(), JoypadButtons::Down),
                ("Left".to_string(), JoypadButtons::Left),
                ("Right".to_string(), JoypadButtons::Right),
                ("Start".to_string(), JoypadButtons::Start),
                ("Select".to_string(), JoypadButtons::Select),
                ("A".to_string(), JoypadButtons::A),
                ("B".to_string(), JoypadButtons::B)
            ]),
            button_to_keycode: button_to_keys,
            button_to_index,
            current_input: None,
            current_action: None,
            current_joy_input: None
        }
    }

    pub fn render_screen(&mut self, cpu: &mut CPU) {
        cpu.bus.ppu.cap_fps();

        cpu.bus.ppu.frame_finished = false;

        let screen = &cpu.bus.ppu.picture.data;

        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            self.gl.bind_texture(TEXTURE_2D, Some(self.texture));

            self.gl.tex_sub_image_2d(
                    TEXTURE_2D,
                    0,
                    0,
                    0,
                    SCREEN_WIDTH as i32,
                    SCREEN_HEIGHT as i32,
                    RGBA,
                    UNSIGNED_BYTE,
                    PixelUnpackData::Slice(&screen)
            );

            self.gl.blit_framebuffer(
                    0,
                    SCREEN_HEIGHT as i32,
                    SCREEN_WIDTH as i32,
                    0,
                    0,
                    0,
                    SCREEN_WIDTH as i32 * 3,
                    SCREEN_HEIGHT as i32 * 3,
                    COLOR_BUFFER_BIT,
                    NEAREST
            );
        }
    }

    pub fn end_frame(&mut self) {
        self.window.gl_swap_window();
    }

    pub fn load_rtc(&mut self, cpu: &mut CPU) {
        match &mut cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc3) => {
                let bytes = {
                    let mut cloud_service = self.cloud_service.lock().unwrap();
                    let mut rtc_name = cloud_service.game_name.strip_suffix(".sav").unwrap().to_string();

                    rtc_name.push_str(".rtc");

                    cloud_service.get_file(Some(rtc_name))
                };

                let json_str = str::from_utf8(&bytes).unwrap();

                if json_str != "" {
                    mbc3.load_rtc(json_str.to_string());
                } else {
                    self.update_rtc(cpu, true, true);
                }
            }
            _ => ()
        }
    }

    pub fn update_rtc(&mut self, cpu: &mut CPU, logged_in: bool, is_initial: bool) {
        match &mut cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc3) => {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("an error occurred")
                    .as_millis();
                if (self.last_check.is_some() && current_time - self.last_check.unwrap() >= 30 * 60 * 1000) ||
                    is_initial ||
                    mbc3.is_dirty
                {
                    mbc3.is_dirty = false;
                    if logged_in {
                        let rtc_json = RtcFile::new(
                            mbc3.start.timestamp() as usize,
                            mbc3.halted,
                            mbc3.carry_bit,
                            mbc3.num_wraps
                        );

                        let json_str = serde_json::to_string::<RtcFile>(&rtc_json).unwrap();

                        let mut cloud_service = self.cloud_service.lock().unwrap();

                        let mut rtc_name = cloud_service.game_name.strip_suffix(".sav").unwrap().to_string();

                        rtc_name.push_str(".rtc");

                        cloud_service.upload_file(json_str.as_bytes(), Some(rtc_name));
                    } else {
                        mbc3.save_rtc();
                    }
                    self.last_check = None;
                } else {
                    self.last_check = Some(SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("an error occurred")
                        .as_millis());
                }
            }
            _ => ()
        }
    }

    // used throughout the emulator's lifetime and saves when certain conditions are met.
    pub fn check_saves(&mut self, cpu: &mut CPU, logged_in: bool) {
        let mbc = &mut cpu.bus.cartridge.mbc;
        match mbc {
            MBC::MBC1(mbc) => {
                if mbc.check_save(logged_in) {
                    if logged_in {
                        let data = mbc.backup_file.ram.clone();

                        mbc.backup_file.is_dirty = false;
                        mbc.backup_file.last_updated = 0;

                        mbc.backup_file.last_saved = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("an error occurred")
                            .as_millis();

                        let cloud_service = self.cloud_service.clone();
                        thread::spawn(move || {
                            cloud_service.lock().unwrap().upload_file(&data, None);
                        });
                    } else {
                        mbc.backup_file.save_file();
                    }
                }
            }
            MBC::MBC3(mbc) => if mbc.check_save(logged_in) {
                if logged_in {
                   let data = mbc.backup_file.ram.clone();

                    mbc.backup_file.is_dirty = false;
                    mbc.backup_file.last_updated = 0;

                    mbc.backup_file.last_saved = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("an error occurred")
                        .as_millis();

                    let cloud_service = self.cloud_service.clone();
                    thread::spawn(move || {
                        cloud_service.lock().unwrap().upload_file(&data, None);
                    });
                } else {
                    mbc.backup_file.save_file();
                }
            }
            MBC::MBC5(mbc) => if mbc.check_save(logged_in) {
                if logged_in {
                    let data = mbc.backup_file.ram.clone();

                    mbc.backup_file.is_dirty = false;
                    mbc.backup_file.last_updated = 0;

                    mbc.backup_file.last_saved = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("an error occurred")
                        .as_millis();

                    let cloud_service = self.cloud_service.clone();
                    thread::spawn(move || {
                        cloud_service.lock().unwrap().upload_file(&data, None);
                    });
                } else {
                    mbc.backup_file.save_file();
                }
            }
            _ => ()
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

    pub fn clear_framebuffer(&mut self) {
        unsafe {
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn unzip_game(rom_path: PathBuf) -> (Vec<u8>, String) {
        let file = fs::File::open(rom_path.to_str().unwrap()).unwrap();
        let mut archive = ZipArchive::new(file).unwrap();

        let mut file_found = false;

        let mut rom_bytes = Vec::new();
        let mut rom_path_str = "".to_string();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            if file.is_file() {
                file_found = true;
                rom_bytes = vec![0; file.size() as usize];
                file.read_exact(&mut rom_bytes).unwrap();

                let real_file_name = file.name().to_string();

                let mut split_str: Vec<&str> = rom_path.to_str().unwrap().split("/").collect();

                split_str.pop();

                split_str.push(&real_file_name);

                rom_path_str = split_str.join("/");

                break;
            }
        }

        if !file_found {
            panic!("couldn't extract ROM from zip file!");
        }

        (rom_bytes, rom_path_str)
    }

    fn reload_cpu(
        cpu: &mut CPU,
        current_palette: usize,
        producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
        waveform_producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
        rom_bytes: &[u8],
        rom_path: String,
        logged_in: bool,
        cloud_service: Arc<Mutex<CloudService>>,
        fetch_save: bool
    ) -> Vec<u8> {
        *cpu = CPU::new(producer, Some(waveform_producer), Some(rom_path), false, true);

        cpu.load_rom(rom_bytes, logged_in);

        cpu.bus.ppu.set_dmg_palette(current_palette);

        if logged_in && fetch_save {
            let bytes = cloud_service.lock().unwrap().get_file(None);

            if bytes.len() > 0 {
                cpu.bus.cartridge.load_save(&bytes);

                return bytes;
            }
        }

        Vec::new()
    }

    fn create_state(cpu: &mut CPU, game_path: String) {
        let (data, _) = cpu.create_save_state();

        let compressed = zstd::encode_all(&*data, 9).unwrap();

        let now = Local::now();

        let name = format!("save_state_{}.state", now.format("%Y%m%d%H%M%S"));

        let mut dir = data_dir().unwrap();

        dir.push("GBC+");

        let mut split: Vec<&str> = game_path.split('/').collect();

        let game_name = split.pop().unwrap();

        dir.push(game_name);

        fs::create_dir_all(&dir).expect("Couldn't create save state directory");

        dir.push(name);

        fs::write(dir, compressed).unwrap();
    }

    fn load_state(
        cpu: &mut CPU,
        dir: PathBuf,
        rom_bytes: &[u8],
        producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
        waveform_producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>
    ) {
        let compressed = fs::read(dir).unwrap();

        let bytes = zstd::decode_all(&*compressed).unwrap();

        cpu.load_save_state(&bytes);

        cpu.reload_rom(rom_bytes);

        cpu.bus.apu.producer = Some(producer);
        cpu.bus.apu.waveform_producer = Some(waveform_producer);
    }

    pub fn render_ui(
        &mut self,
        cpu: &mut CPU,
        logged_in: &mut bool,
        rom_bytes: &mut Vec<u8>,
        save_name: &mut String,
        save_bytes: &mut Option<Vec<u8>>
    ) {
        self.platform.prepare_frame(&mut self.imgui, &mut self.window, &self.event_pump);

        let mut should_reset = false;
        let mut reuse_save = false;

        let ui = self.imgui.new_frame();

        let previous_logged_in = *logged_in;

        if self.display_ui {
            if self.show_bindings_popup {
                ui.open_popup("bindings")
            }
            if self.confirm_delete_dialog {
                ui.open_popup("confirm_delete");
            }
            if self.show_palette_picker_popup {
                ui.open_popup("palette_picker")
            }
            if let Some(token) = ui.begin_popup("palette_picker") {
                ui.text("Choose a palette:");

                for i in 0..cpu.bus.ppu.palette_colors.len() {
                    let color = cpu.bus.ppu.palette_colors[i][1];
                    let theme_name = THEME_NAMES[i];

                    let color_normalized = [color.r as f32 / 255.0, color.g as f32 / 255.0, color.b as f32 / 255.0, 1.0];

                    ui.text(theme_name);

                    ui.same_line();

                    if ui.color_button(theme_name, color_normalized) {
                        cpu.bus.ppu.set_dmg_palette(i);

                        self.config.current_palette = i;

                        Self::write_config_file(&self.config, &mut self.config_file);

                        self.show_palette_picker_popup = false;
                        ui.close_current_popup();
                    }
                }

                token.end();
            }
            if let Some(token) = ui.begin_popup("confirm_delete") {
                ui.text("Are you sure you want to delete this save state?");

                if ui.button("Yes") {
                    if let Some(filepath) = &self.file_to_delete {
                        fs::remove_file(filepath).unwrap();

                        self.file_to_delete = None;
                        self.confirm_delete_dialog = false;
                        ui.close_current_popup();
                    }
                }

                ui.same_line();

                if ui.button("No") {
                    self.file_to_delete = None;
                    self.confirm_delete_dialog = false;

                    ui.close_current_popup();
                }

                token.end();
            }
            if let Some(token) = ui.begin_popup("bindings") {
                let button_width = 100.0;
                let button_height = 15.0;

                let popup_width = ui.content_region_avail()[0];

                if let Some(tab_bar) = ui.tab_bar("Controller bindings") {
                    if let Some(tab) = ui.tab_item("Keyboard") {
                        let cursor = ui.cursor_pos();

                        ui.set_cursor_pos([cursor[0], cursor[1] + 20.0]);

                        for i in 0..ACTIONS.len() {
                            let action = ACTIONS[i];

                            ui.separator();

                            let mut color = [1.0, 1.0, 1.0, 1.0];

                            let button = *self.ctrl_strs_to_buttons.get(action).unwrap();

                            if let Some(current_input) = self.current_input {
                                if current_input == button {
                                    color = [0.0, 1.0, 0.0, 1.0];
                                }
                            }

                            ui.text_colored(color, format!("{action}"));

                            ui.same_line_with_spacing(100.0, 0.0);

                            let keycode = *self.button_to_keycode.get(&button).unwrap();

                            if ui.button_with_size(Keycode::name(keycode), [button_width, button_height]) {
                                self.current_input = Some(button);
                                self.current_action = Some(i);
                            }
                        }
                        tab.end();
                    }
                    if let Some(tab) = ui.tab_item("Joypad") {
                        let cursor = ui.cursor_pos();

                        ui.set_cursor_pos([cursor[0], cursor[1] + 20.0]);

                        for i in 0..JOY_ACTIONS.len() {
                            let action = JOY_ACTIONS[i];

                            ui.separator();

                            let mut color = [1.0, 1.0, 1.0, 1.0];

                            let button = *self.ctrl_strs_to_buttons.get(action).unwrap();

                            if let Some(current_input) = self.current_joy_input {
                                if current_input == button {
                                    color = [0.0, 1.0, 0.0, 1.0];
                                }
                            }

                            ui.text_colored(color, format!("{action}"));

                            ui.same_line_with_spacing(100.0, 0.0);

                            let button_index = *self.button_to_index.get(&button).unwrap();

                            if ui.button_with_size(format!("{}", button_index.to_string()), [button_width, button_height]) {
                                self.current_joy_input = Some(button);
                                self.current_action = Some(i);
                            }
                        }
                        tab.end();
                    }

                    tab_bar.end();
                }

                ui.spacing();

                if self.current_input.is_some() {
                    ui.text_colored([0.0, 1.0, 0.0, 1.0], "Awaiting input...");
                }

                let cursor = ui.cursor_pos();

                let x = (popup_width - button_width) * 0.5;

                ui.set_cursor_pos([x, cursor[1] + 20.0]);

                if ui.button_with_size("Close", [button_width, button_height]) {
                    ui.close_current_popup();
                    self.show_bindings_popup = false;
                }

                token.end();
            }

            ui.main_menu_bar(|| {
                if let Some(menu) = ui.begin_menu("File") {
                    if ui.menu_item("Open") {
                        match FileDialog::new()
                        .add_filter("GBC rom file", &["gbc", "gb", "zip"])
                        .show_open_single_file() {
                            Ok(path) => {
                                if let Some(path) = path {
                                    let extension = path.extension().unwrap().to_str().unwrap();
                                    if extension == "zip" {
                                        let (new_bytes, rom_path) = Self::unzip_game(path);

                                        *rom_bytes = new_bytes;

                                        let mut split_str_vec: Vec<&str> = rom_path.split('.').collect();

                                        split_str_vec.pop();

                                        *save_name = format!("{}.sav", split_str_vec.join("."));

                                        if *logged_in {
                                            let mut split_str_vec: Vec<&str> = save_name.split('/').collect();

                                            let game_name = split_str_vec.pop().unwrap().to_string();

                                            self.cloud_service.lock().unwrap().game_name = game_name;
                                        }
                                    } else {
                                        *rom_bytes = fs::read(&path).unwrap();

                                        let rom_path = path.to_str().unwrap();

                                        let mut split_str_vec: Vec<&str> = rom_path.split('.').collect();

                                        split_str_vec.pop();

                                        *save_name = format!("{}.sav", split_str_vec.join("."));

                                        if *logged_in {
                                            let mut split_str_vec: Vec<&str> = save_name.split('/').collect();

                                            let game_name = split_str_vec.pop().unwrap().to_string();

                                            self.cloud_service.lock().unwrap().game_name = game_name;
                                        }
                                    }
                                    should_reset = true;
                                }
                            }
                            Err(_) => ()
                        }
                    }
                    if ui.menu_item("Reset") {
                        reuse_save = true;
                        should_reset = true;
                    }
                    if let Some(menu) = ui.begin_menu("Cloud saves") {
                        if !*logged_in {
                            if ui.menu_item("Log in") {
                                let mut cloud_service = self.cloud_service.lock().unwrap();

                                cloud_service.login();
                                *logged_in = true;

                                should_reset = true;
                            }
                        } else {
                            if ui.menu_item("Log out") {
                                let mut cloud_service = self.cloud_service.lock().unwrap();

                                cloud_service.logout();
                                *logged_in = false;

                                should_reset = true;
                            }
                        }

                        menu.end();
                    }
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Save states") {
                    if ui.menu_item("Create save state") {
                        Self::create_state(cpu, save_name.replace(".sav", ""));
                    }
                    if let Some(menu) = ui.begin_menu("Load save state") {
                        // load save states from dir
                        Self::process_save_states(save_name.replace(".sav", ""), |file, dir| {
                            if ui.menu_item(file) {
                                let ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

                                let (producer, consumer) = ringbuffer.split();

                                let waveform_ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

                                let (waveform_producer, waveform_consumer) = waveform_ringbuffer.split();

                                Self::load_state(cpu, dir, rom_bytes, producer, waveform_producer);

                                self.wave_consumer = waveform_consumer;
                                self.device.lock().consumer = consumer;
                            }
                        });

                        menu.end();
                    }
                    if let Some(menu) = ui.begin_menu("Delete save state") {
                        Self::process_save_states(save_name.replace(".sav", ""), |file, dir| {
                            if ui.menu_item(file) {
                                self.confirm_delete_dialog = true;
                                self.file_to_delete = Some(dir.clone());
                            }
                        });

                        menu.end();
                    }
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Misc") {
                    if ui.menu_item("DMG Palette picker [F2]") {
                        self.show_palette_picker_popup = true;
                    }
                    if ui.menu_item("Waveform visualizer [F4]") {
                        self.show_waveform = !self.show_waveform;
                        if self.show_waveform {
                            self.waveform_canvas.window_mut().show();
                        } else {
                            self.waveform_canvas.window_mut().hide();
                        }
                    }
                    if ui.menu_item("Controller bindings") {
                        self.show_bindings_popup = true;
                    }
                    menu.end();
                }
            });
        }

        if should_reset {
            let ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

            let (producer, consumer) = ringbuffer.split();

            let waveform_ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

            let (waveform_producer, waveform_consumer) = waveform_ringbuffer.split();

            if !previous_logged_in && *logged_in {
                if let Some(save_bytes) = save_bytes {
                    let new_bytes = self.cloud_service.lock().unwrap().get_file(None);

                    *save_bytes = new_bytes;

                    cpu.bus.cartridge.load_save(save_bytes);
                }
            }

            let new_save_bytes = Self::reload_cpu(
                cpu,
                self.config.current_palette,
                producer,
                waveform_producer,
                rom_bytes,
                save_name.to_string(),
                *logged_in,
                self.cloud_service.clone(),
                !reuse_save
            );

            if let Some(bytes) = save_bytes {
                if reuse_save {
                    cpu.bus.cartridge.load_save(bytes);
                } else if new_save_bytes.len() > 0 {
                    *bytes = new_save_bytes;
                }
            }

            self.wave_consumer = waveform_consumer;
            self.device.lock().consumer = consumer;
        }

        let draw_data = self.imgui.render();

        self.renderer.render(&self.gl, &mut self.textures, draw_data).unwrap();
    }

    // used when the user closes the emulator and the game saves one more time
    fn save_game(&mut self, mbc: &mut MBC, logged_in: bool) {
        match mbc {
            MBC::MBC1(mbc) => {
                if mbc.backup_file.is_dirty {
                    if logged_in {
                        let data = mbc.backup_file.ram.clone();

                        mbc.backup_file.is_dirty = false;
                        mbc.backup_file.last_updated = 0;

                        mbc.backup_file.last_saved = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("an error occurred")
                            .as_millis();

                        let cloud_service = self.cloud_service.clone();
                        thread::spawn(move || {
                            cloud_service.lock().unwrap().upload_file(&data, None);
                        });
                    } else {
                        mbc.backup_file.save_file();
                    }
                }
            }
            MBC::MBC3(mbc) => {
                if mbc.backup_file.is_dirty {
                    if logged_in {
                        let data = mbc.backup_file.ram.clone();

                        mbc.backup_file.is_dirty = false;
                        mbc.backup_file.last_updated = 0;

                        mbc.backup_file.last_saved = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("an error occurred")
                            .as_millis();

                        let cloud_service = self.cloud_service.clone();
                        thread::spawn(move || {
                            cloud_service.lock().unwrap().upload_file(&data, None);
                        });
                    } else {
                        mbc.backup_file.save_file();
                    }
                }
            }
            MBC::MBC5(mbc) => {
                if mbc.backup_file.is_dirty {
                    if logged_in {
                        let data = mbc.backup_file.ram.clone();

                        mbc.backup_file.is_dirty = false;
                        mbc.backup_file.last_updated = 0;

                        mbc.backup_file.last_saved = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("an error occurred")
                            .as_millis();

                        let cloud_service = self.cloud_service.clone();
                        thread::spawn(move || {
                            cloud_service.lock().unwrap().upload_file(&data, None);
                        });
                    } else {
                        mbc.backup_file.save_file();
                    }
                }
            }
            _=> ()
        }
    }

    fn create_quick_state(cpu: &mut CPU, save_name: String) {
        let (bytes, _) = cpu.create_save_state();

        let compressed = zstd::encode_all(&*bytes, 9).unwrap();

        let filename = "quick_save.state";

        let game_path = save_name.replace(".sav", "");

        let mut split: Vec<&str> = game_path.split('/').collect();

        let game_name = split.pop().unwrap();

        let mut dir = data_dir().unwrap();

        dir.push("GBC+");
        dir.push(game_name);

        fs::create_dir_all(&dir).expect("Couldn't create save state directory");

        dir.push(filename);

        fs::write(dir, compressed).unwrap();
    }

    fn get_quick_save_path(save_name: String) -> PathBuf {
        let mut dir = data_dir().unwrap();

        dir.push("GBC+");

        let game_path = save_name.replace(".sav", "");

        let mut split: Vec<&str> = game_path.split('/').collect();

        let game_name = split.pop().unwrap();

        dir.push(game_name);

        fs::create_dir_all(&dir).expect("Couldn't create save state directory");

        dir.push("quick_save.state");

        dir
    }

    fn write_config_file(config: &EmuConfig, config_file: &mut File) {
        let json = match serde_json::to_string(config) {
            Ok(result) => result,
            Err(_) => "".to_string()
        };

        if json != "" {
            config_file.seek(SeekFrom::Start(0)).unwrap();
            config_file.write_all(json.as_bytes()).unwrap();
        }
    }

    pub fn handle_events(&mut self, cpu: &mut CPU, logged_in: bool, save_name: &str, rom_bytes: &[u8]) {
        for event in self.event_pump.poll_iter() {
            self.platform.handle_event(&mut self.imgui, &event);
            match event {
                Event::Quit { .. } => {
                    self.save_game(&mut cpu.bus.cartridge.mbc, logged_in);
                    exit(0);
                }
                Event::Window { win_event, window_id, .. } => {
                    if win_event == WindowEvent::Close {
                        if window_id == 1 {
                            self.save_game(&mut cpu.bus.cartridge.mbc, logged_in);
                            exit(0);
                        } else if window_id == 2 {
                            self.show_waveform = false;
                            self.waveform_canvas.window_mut().hide();
                        }
                    }
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(button) = self.current_input {
                        if let Some(keycode) = keycode {
                            if let Some(old_keycode) = self.button_to_keycode.get(&button) {
                                self.keyboard_map.remove(old_keycode);

                                if let Some(old_button) = self.keyboard_map.get(&keycode) {
                                    let old_button = *old_button;
                                    let old_keycode = *old_keycode;

                                    self.keyboard_map.insert(old_keycode, old_button);
                                    self.button_to_keycode.insert(old_button, old_keycode);
                                }
                            }

                            self.keyboard_map.insert(keycode, button);
                            self.button_to_keycode.insert(button, keycode);

                            if let Some(current_action) = &mut self.current_action {
                                *current_action += 1;
                                if *current_action < ACTIONS.len() {
                                    let input = *self.ctrl_strs_to_buttons.get(ACTIONS[*current_action]).unwrap();
                                    self.current_input = Some(input);

                                } else {
                                    self.current_input = None;
                                    self.current_action = None;
                                }
                            }

                            let mut keyboard_map = HashMap::<String, JoypadButtons>::new();

                            for (key, value) in self.keyboard_map.iter() {
                                keyboard_map.insert(Keycode::name(*key), *value);
                            }

                            self.config.keyboard_map = keyboard_map;

                            let mut button_to_keys = HashMap::<JoypadButtons, String>::new();

                            for (key, value) in self.keyboard_map.iter() {
                                button_to_keys.insert(*value, Keycode::name(*key));
                            }

                            self.config.button_to_keys = button_to_keys;

                            Self::write_config_file(&self.config, &mut self.config_file);
                        }
                    } else {
                        if let Some(keycode) = keycode {
                            if let Some(button) = self.keyboard_map.get(&keycode) {
                                self.display_ui = false;
                                cpu.bus.joypad.press_button(*button);
                            } else if keycode == Keycode::G {

                                cpu.bus.ppu.debug_on = !cpu.bus.ppu.debug_on;
                                cpu.bus.debug_on = !cpu.bus.debug_on;
                                cpu.debug_on = !cpu.debug_on;
                            } else if keycode == Keycode::F2 {
                                cpu.bus.ppu.current_palette = (cpu.bus.ppu.current_palette + 1) % cpu.bus.ppu.palette_colors.len();

                                self.config.current_palette = cpu.bus.ppu.current_palette;

                                // fuck you......... "cannot borrow self as mutable more than once" SHUT THE FUCK UP
                                Self::write_config_file(&self.config, &mut self.config_file);
                            } else if keycode == Keycode::F4 {
                                self.show_waveform = !self.show_waveform;

                                if self.show_waveform {
                                    self.waveform_canvas.window_mut().show();
                                } else {
                                    self.waveform_canvas.window_mut().hide();
                                }
                            } else if keycode == Keycode::F5 {
                                Self::create_quick_state(cpu, save_name.to_string());
                            } else if keycode == Keycode::F7 {
                                let dir = Self::get_quick_save_path(save_name.to_string());

                                let ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

                                let (producer, consumer) = ringbuffer.split();

                                let waveform_ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

                                let (waveform_producer, waveform_consumer) = waveform_ringbuffer.split();

                                Self::load_state(cpu, dir, rom_bytes, producer, waveform_producer);

                                self.wave_consumer = waveform_consumer;
                                self.device.lock().consumer = consumer;


                            } else if keycode == Keycode::Escape {
                                self.display_ui = !self.display_ui;
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
                    if let Some(current_input) = self.current_joy_input {
                        if let Some(old_index) = self.button_to_index.get(&current_input) {
                            let old_index_u8 = *old_index as u8;
                            let old_index = *old_index;
                            self.button_map.remove(&old_index_u8);

                            if let Some(old_button) = self.button_map.get(&button_idx) {
                                let old_button = *old_button;

                                self.button_map.insert(old_index_u8, old_button);
                                self.button_to_index.insert(old_button, old_index);
                            }
                        }

                        self.button_map.insert(button_idx, current_input);
                        self.button_to_index
                            .insert(
                                current_input,
                                ButtonIndex::try_from(button_idx)
                                    .unwrap_or_else(|_| panic!("unknown button given: {button_idx}"))
                            );

                        if let Some(current_action) = &mut self.current_action {
                            *current_action += 1;
                            if *current_action < JOY_ACTIONS.len() {
                                let input = *self.ctrl_strs_to_buttons.get(JOY_ACTIONS[*current_action]).unwrap();
                                self.current_joy_input = Some(input);

                            } else {
                                self.current_joy_input = None;
                                self.current_action = None;
                            }
                        }

                        self.config.button_map = self.button_map.clone();
                        self.config.button_to_index = self.button_to_index.clone();

                        Self::write_config_file(&self.config, &mut self.config_file);
                    } else {
                        if let Some(button) = self.button_map.get(&button_idx) {
                            self.display_ui = false;
                            cpu.bus.joypad.press_button(*button);
                        } else if button_idx == ButtonIndex::LeftThumbstick as u8 {
                            Self::create_quick_state(cpu, save_name.to_string());
                        } else if button_idx == ButtonIndex::RightThumbstick as u8 {
                            let dir = Self::get_quick_save_path(save_name.to_string());

                            let ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

                            let (producer, consumer) = ringbuffer.split();

                            let waveform_ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

                            let (waveform_producer, waveform_consumer) = waveform_ringbuffer.split();

                            Self::load_state(cpu, dir, rom_bytes, producer, waveform_producer);

                            self.wave_consumer = waveform_consumer;
                            self.device.lock().consumer = consumer;
                        }
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