use std::{collections::HashMap, sync::Arc, thread::sleep, time::Duration};

use gbc_plus::cpu::{bus::{cartridge::mbc::MBC, joypad::JoypadButtons}, CPU};
use ringbuf::{
    storage::Heap,
    traits::{
        Consumer,
        Observer,
        Split
    },
    wrap::caching::Caching,
    HeapRb,
    SharedRb
};

const BUTTON_CROSS: usize = 0;
const BUTTON_CIRCLE: usize = 1;
// const BUTTON_SQUARE: usize = 2;
// const BUTTON_TRIANGLE: usize = 3;
const SELECT: usize = 4;
const START: usize = 6;
// const LEFT_STICK: usize = 7;
// const RIGHT_STICK: usize = 8;
// const BUTTON_L: usize = 9;
// const BUTTON_R: usize = 10;
const UP: usize = 12;
const DOWN: usize = 13;
const LEFT: usize = 14;
const RIGHT: usize = 15;
// const LEFT_TRIGGER: usize = 16;
// const RIGHT_TRIGGER: usize = 17;

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type GBCMobileEmulator;

        #[swift_bridge(init)]
        fn new() -> GBCMobileEmulator;

        #[swift_bridge(swift_name="hasTimer")]
        fn has_timer(&self) -> bool;

        #[swift_bridge(swift_name="fetchRtc")]
        fn fetch_rtc(&self) -> String;

        #[swift_bridge(swift_name="loadRtc")]
        fn load_rtc(&mut self, json: String);

        #[swift_bridge(swift_name="loadRom")]
        fn load_rom(&mut self, data: &[u8]);

        #[swift_bridge(swift_name="stepFrame")]
        fn step_frame(&mut self);

        #[swift_bridge(swift_name="getScreen")]
        fn get_screen(&self) -> *const u8;

        #[swift_bridge(swift_name="getScreenLength")]
        fn get_screen_length(&self) -> usize;

        #[swift_bridge(swift_name="loadSave")]
        fn load_save(&mut self, buf: &[u8]);

        #[swift_bridge(swift_name="hasSaved")]
        fn has_saved(&mut self) -> bool;

        #[swift_bridge(swift_name="getSaveLength")]
        fn get_save_length(&self) -> usize;

        #[swift_bridge(swift_name="saveGame")]
        fn save_game(&mut self) -> *const u8;

        #[swift_bridge(swift_name="getBufferLength")]
        fn get_buffer_len(&self) -> usize;

        #[swift_bridge(swift_name="updateInput")]
        fn update_input(&mut self, button: usize, pressed: bool);

        #[swift_bridge(swift_name="setPaused")]
        fn set_paused(&mut self, val: bool);

        #[swift_bridge(swift_name="readRingBuffer")]
        fn read_ringbuffer(&mut self) -> *const f32;

        #[swift_bridge(swift_name="hasSamples")]
        fn has_samples(&self) -> bool;

        #[swift_bridge(swift_name="popSample")]
        fn pop_sample(&mut self) -> f32;

        #[swift_bridge(swift_name="createSaveState")]
        fn create_save_state(&mut self) -> *const u8;

        #[swift_bridge(swift_name="saveStateLength")]
        fn save_state_len(&self) -> usize;

        #[swift_bridge(swift_name="loadSaveState")]
        fn load_save_state(&mut self, data: &[u8]);

        #[swift_bridge(swift_name="reloadRom")]
        fn reload_rom(&mut self, bytes: &[u8]);

        #[swift_bridge(swift_name="setPausedAudio")]
        fn set_paused_audio(&mut self, value: bool);
    }
}

pub struct GBCMobileEmulator {
    cpu: CPU,
    joypad_map: HashMap<usize, JoypadButtons>,
    sample_buffer: Vec<f32>,
    paused: bool,
    consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
    state_data: Vec<u8>
}

impl GBCMobileEmulator {
        pub fn new() -> Self {

        let joypad_map = HashMap::<usize, JoypadButtons>::from([
            (BUTTON_CIRCLE, JoypadButtons::A),
            (BUTTON_CROSS, JoypadButtons::B),
            (SELECT, JoypadButtons::Select),
            (START, JoypadButtons::Start),
            (UP, JoypadButtons::Up),
            (DOWN, JoypadButtons::Down),
            (LEFT, JoypadButtons::Left),
            (RIGHT, JoypadButtons::Right)
        ]);

        let ringbuffer = HeapRb::<f32>::new(4096 * 2);

        let (producer, consumer) = ringbuffer.split();

        Self {
            cpu: CPU::new(producer, None, None, true),
            joypad_map,
            sample_buffer: Vec::new(),
            paused: false,
            consumer,
            state_data: Vec::new()
        }
    }

    pub fn set_paused_audio(&mut self, value: bool) {
        self.cpu.bus.apu.is_paused = value;
    }

    pub fn create_save_state(&mut self) -> *const u8 {
        let (bytes, _) = self.cpu.create_save_state();

        let compressed = zstd::encode_all(&*bytes, 9).unwrap();

        self.state_data = compressed;

        self.state_data.as_ptr()
    }

    pub fn reload_rom(&mut self, bytes: &[u8]) {
        self.cpu.reload_rom(bytes);
    }

    pub fn load_save_state(&mut self, data: &[u8]) {
        let decompressed = zstd::decode_all(&*data).unwrap();
        self.cpu.load_save_state(&decompressed);

        let ringbuffer = HeapRb::<f32>::new(4096 * 2);

        let (producer, consumer) = ringbuffer.split();

        self.consumer = consumer;
        self.cpu.bus.apu.producer = Some(producer);
    }

    pub fn save_state_len(&self) -> usize {
        self.state_data.len()
    }

    pub fn has_timer(&self) -> bool {
        match &self.cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc) => mbc.has_timer,
            _ => false
        }
    }

    pub fn fetch_rtc(&self) -> String {
        match &self.cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc) => mbc.save_rtc_web_mobile(),
            _ => "".to_string()
        }
    }

    pub fn load_rtc(&mut self, json: String) {
        match &mut self.cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc) => mbc.load_rtc(json),
            _ => ()
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.cpu.load_rom(data);
    }

    pub fn step_frame(&mut self) {
        if !self.paused {
            self.cpu.step_frame();
        } else {
            sleep(Duration::from_millis(100));
        }

        self.cpu.bus.ppu.cap_fps();

        self.cpu.bus.ppu.frame_finished = false;
    }

    pub fn set_paused(&mut self, val: bool) {
        self.paused = val;
    }

    pub fn get_screen(&self) -> *const u8 {
        self.cpu.bus.ppu.picture.data.as_ptr()
    }

    pub fn get_screen_length(&self) -> usize {
        self.cpu.bus.ppu.picture.data.len()
    }

    pub fn load_save(&mut self, buf: &[u8]) {
        match &mut self.cpu.bus.cartridge.mbc {
            MBC::MBC1(mbc) => mbc.backup_file.load_save(buf),
            MBC::MBC3(mbc) => mbc.backup_file.load_save(buf),
            MBC::MBC5(mbc) => mbc.backup_file.load_save(buf),
            _ => ()
        }
    }

    pub fn has_saved(&mut self) -> bool {
        match &mut self.cpu.bus.cartridge.mbc {
            MBC::MBC1(mbc) => mbc.has_saved(),
            MBC::MBC3(mbc) => mbc.has_saved(),
            MBC::MBC5(mbc) => mbc.has_saved(),
            _ => false
        }
    }

    pub fn get_save_length(&self) -> usize {
        match &self.cpu.bus.cartridge.mbc {
            MBC::MBC1(mbc) => mbc.backup_file.ram.len(),
            MBC::MBC3(mbc) => mbc.backup_file.ram.len(),
            MBC::MBC5(mbc) => mbc.backup_file.ram.len(),
            _ => 0
        }
    }

    pub fn save_game(&mut self) -> *const u8 {
        match &mut self.cpu.bus.cartridge.mbc {
            MBC::MBC1(mbc) => mbc.backup_file.ram.as_ptr(),
            MBC::MBC3(mbc) => mbc.backup_file.ram.as_ptr(),
            MBC::MBC5(mbc) => mbc.backup_file.ram.as_ptr(),
            _ => {
                let vec = Vec::new();

                vec.as_ptr()
            }
        }
    }

    pub fn read_ringbuffer(&mut self) -> *const f32 {
        self.sample_buffer = Vec::new();

        for sample in self.consumer.pop_iter() {
            self.sample_buffer.push(sample);
        }

        self.sample_buffer.as_ptr()
    }

    pub fn has_samples(&self) -> bool {
        !self.consumer.is_empty()
    }

    pub fn pop_sample(&mut self) -> f32 {
        self.consumer.try_pop().unwrap_or(0.0)
    }

    pub fn get_buffer_len(&self) -> usize {
        self.sample_buffer.len()
    }

    pub fn update_input(&mut self, button: usize, pressed: bool) {
        if pressed {
            if let Some(joypad_button) = self.joypad_map.get(&button) {
                self.cpu.bus.joypad.press_button(*joypad_button);
            }
        } else {
            if let Some(joypad_button) = self.joypad_map.get(&button) {
                self.cpu.bus.joypad.release_button(*joypad_button);
            }
        }
    }
}