use std::{
    collections::{HashMap, VecDeque},
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
    sync::Arc,
    time::{
        SystemTime,
        UNIX_EPOCH
    }
};
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
        apu::NUM_SAMPLES, cartridge::mbc::MBC, joypad::JoypadButtons, ppu::{
            SCREEN_HEIGHT,
            SCREEN_WIDTH
        }
    },
    CPU
};
use imgui_sdl2_support::SdlPlatform;
use ringbuf::{
    storage::Heap, traits::{
        Consumer, Observer, Producer, Split
    }, wrap::caching::Caching, HeapRb, SharedRb
};
use sdl2::{
    audio::{
        AudioCallback,
        AudioDevice,
        AudioSpecDesired
    },
    controller::GameController,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    render::Canvas,
    video::{GLContext, GLProfile, Window},
    EventPump,
    GameControllerSubsystem
};
use serde::{Deserialize, Serialize};
use imgui::{Context, Textures};

const BUTTON_CROSS: u8 = 0;
const BUTTON_SQUARE: u8 = 2;
const BUTTON_SELECT: u8 = 4;
const BUTTON_START: u8 = 6;
const BUTTON_UP: u8 = 11;
const BUTTON_DOWN: u8 = 12;
const BUTTON_LEFT: u8 = 13;
const BUTTON_RIGHT: u8 = 14;

const WAVEFORM_LENGTH: usize = 683;
const WAVEFORM_HEIGHT: usize = 256;

pub enum UIAction {
    None,
    Waveform
}

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
    _device: AudioDevice<GbcAudioCallback>,
    event_pump: EventPump,
    button_map: HashMap<u8, JoypadButtons>,
    keyboard_map: HashMap<Keycode, JoypadButtons>,
    controller_id: Option<u32>,
    game_controller_subsystem: GameControllerSubsystem,
    retry_attempts: usize,
    config: EmuConfig,
    config_file: File,
    last_check: Option<u128>,
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
    samples: Vec<f32>
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
        wave_consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>
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

        let _device = audio_subsystem.open_playback(
            None,
            &spec,
            |_| GbcAudioCallback { consumer }
        ).unwrap();

        _device.resume();

        let window = video_subsystem
            .window("GBC+", (SCREEN_WIDTH * 3) as u32, (SCREEN_HEIGHT * 3) as u32)
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let mut waveform_window = video_subsystem
            .window("Waveform Viewer", 683, 256)
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
            _device,
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
            samples: Vec::with_capacity(NUM_SAMPLES)
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

    pub fn update_rtc(&mut self, cpu: &mut CPU) {

        match &mut cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc3) => {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("an error occurred")
                    .as_millis();
                if let Some(last_check) = self.last_check {
                    if current_time - last_check >= 1500 {
                        mbc3.save_rtc();
                        self.last_check = None;
                    }
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

    pub fn check_saves(&mut self, cpu: &mut CPU) {
        match &mut cpu.bus.cartridge.mbc {
            MBC::MBC1(mbc) => mbc.check_save(),
            MBC::MBC3(mbc) => mbc.check_save(),
            MBC::MBC5(mbc) => mbc.check_save(),
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

    pub fn render_ui(&mut self) -> UIAction {
        self.platform.prepare_frame(&mut self.imgui, &mut self.window, &self.event_pump);

        let mut action = UIAction::None;

        let ui = self.imgui.new_frame();

        ui.main_menu_bar(|| {
            if let Some(menu) = ui.begin_menu("Misc") {
                if ui.menu_item("Waveform visualizer") {
                    self.show_waveform = !self.show_waveform;
                    if self.show_waveform {
                        self.waveform_canvas.window_mut().show();
                    } else {
                        self.waveform_canvas.window_mut().hide();
                    }
                }
            }
        });

        let draw_data = self.imgui.render();

        self.renderer.render(&self.gl, &mut self.textures, draw_data).unwrap();

        action
    }

    pub fn handle_events(&mut self, cpu: &mut CPU) {
        for event in self.event_pump.poll_iter() {
            self.platform.handle_event(&mut self.imgui, &event);
            match event {
                Event::Window { win_event, window_id, .. } => {
                    if win_event == WindowEvent::Close {
                        if window_id == 1 {
                            match &mut cpu.bus.cartridge.mbc {
                                MBC::MBC1(mbc) => {
                                    if mbc.backup_file.is_dirty {
                                        mbc.backup_file.save_file();
                                    }
                                }
                                MBC::MBC3(mbc) => {
                                    if mbc.backup_file.is_dirty {
                                        mbc.backup_file.save_file();
                                    }
                                }
                                MBC::MBC5(mbc) => {
                                    if mbc.backup_file.is_dirty {
                                        mbc.backup_file.save_file();
                                    }
                                }
                                _=> ()
                            }
                            exit(0);
                        } else if window_id == 2 {
                            self.show_waveform = false;
                            self.waveform_canvas.window_mut().hide();
                        }
                    }
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