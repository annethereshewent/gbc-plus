use std::{collections::HashMap, thread::sleep, time::Duration};

use gbc_plus::cpu::{bus::joypad::JoypadButtons, CPU};
use ringbuf::traits::{Consumer, Observer};

const BUTTON_CROSS: usize = 0;
const BUTTON_CIRCLE: usize = 1;
const BUTTON_SQUARE: usize = 2;
const BUTTON_TRIANGLE: usize = 3;
const SELECT: usize = 4;
const START: usize = 6;
const BUTTON_L: usize = 9;
const BUTTON_R: usize = 10;
const UP: usize = 12;
const DOWN: usize = 13;
const LEFT: usize = 14;
const RIGHT: usize = 15;
const BUTTON_HOME: usize = 16;
const GAME_MENU: usize = 17;
const QUICK_LOAD: usize = 18;
const QUICK_SAVE: usize = 19;

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
    }
}

pub struct GBCMobileEmulator {
    cpu: CPU,
    joypad_map: HashMap<usize, JoypadButtons>,
    sample_buffer: Vec<f32>,
    paused: bool
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

        Self {
            cpu: CPU::new(None, None, false),
            joypad_map,
            sample_buffer: Vec::new(),
            paused: false
        }
    }

    pub fn has_timer(&self) -> bool {
        if let Some(mbc) = &self.cpu.bus.cartridge.mbc {
            mbc.has_timer()
        } else {
            false
        }
    }

    pub fn fetch_rtc(&self) -> String {
        if let Some(mbc) = &self.cpu.bus.cartridge.mbc {
            mbc.save_rtc_web_mobile()
        } else {
            "".to_string()
        }
    }

    pub fn load_rtc(&mut self, json: String) {
        if let Some(mbc) = &mut self.cpu.bus.cartridge.mbc {
            mbc.load_rtc(json);
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
        if let Some(mbc) = &mut self.cpu.bus.cartridge.mbc {
            mbc.load_save(buf);
        }
    }

    pub fn has_saved(&mut self) -> bool {
        let return_val = if let Some(mbc) = &mut self.cpu.bus.cartridge.mbc {
            let return_val = mbc.backup_file().is_dirty;

            mbc.clear_is_dirty();

            return_val
        } else {
            false
        };

        return_val
    }

    pub fn get_save_length(&self) -> usize {
        if let Some(mbc) = &self.cpu.bus.cartridge.mbc {
            mbc.backup_file().ram.len()
        } else {
            0
        }
    }

    pub fn save_game(&mut self) -> *const u8 {
        if let Some(mbc) = &mut self.cpu.bus.cartridge.mbc {
            mbc.save_web_mobile()
        } else {
            let vec = Vec::new();

            vec.as_ptr()
        }
    }

    pub fn read_ringbuffer(&mut self) -> *const f32 {
        self.sample_buffer = Vec::new();
        if let Some(ring_buffer) = &mut self.cpu.bus.apu.ring_buffer {
            for sample in ring_buffer.pop_iter() {
                self.sample_buffer.push(sample);
            }
        }

        self.sample_buffer.as_ptr()
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