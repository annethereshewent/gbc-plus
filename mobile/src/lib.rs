use std::collections::HashMap;

use gbc_plus::cpu::{bus::joypad::JoypadButtons, CPU};
use ringbuf::traits::Consumer;

const BUTTON_CROSS: usize = 0;
const BUTTON_SQUARE: usize = 2;
const SELECT: usize = 8;
const START: usize = 9;
const UP: usize = 12;
const DOWN: usize = 13;
const LEFT: usize = 14;
const RIGHT: usize = 15;

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type GBCMobileEmulator;
    }
}

pub struct GBCMobileEmulator {
    cpu: CPU,
    joypad_map: HashMap<usize, JoypadButtons>,
    sample_buffer: Vec<f32>
}

impl GBCMobileEmulator {
        pub fn new() -> Self {

        let joypad_map = HashMap::<usize, JoypadButtons>::from([
            (BUTTON_CROSS, JoypadButtons::A),
            (BUTTON_SQUARE, JoypadButtons::B),
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
            sample_buffer: Vec::new()
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
        self.cpu.step_frame();

        self.cpu.bus.ppu.frame_finished = false;
    }

    pub fn get_screen(&self) -> *const u8 {
        self.cpu.bus.ppu.picture.data.as_ptr()
    }

    pub fn get_screen_length(&self) -> usize {
        self.cpu.bus.ppu.picture.data.len()
    }

    pub fn read_ringbuffer(&mut self) -> *mut f32 {
        self.sample_buffer = Vec::new();
        if let Some(ring_buffer) = &mut self.cpu.bus.apu.ring_buffer {
            for sample in ring_buffer.pop_iter() {
                self.sample_buffer.push(sample);
            }
        }

        self.sample_buffer.as_mut_ptr()
    }

    pub fn pop_sample(&mut self) -> Option<f32> {
        if let Some(ring_buffer) = &mut self.cpu.bus.apu.ring_buffer {
            return ring_buffer.try_pop()
        }

        None
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

    pub fn get_buffer_len(&self) -> usize {
        self.sample_buffer.len()
    }

    pub fn update_input(&mut self, button: usize, pressed: bool) {
        if pressed {
            self.cpu.bus.joypad.press_button(*self.joypad_map.get(&button).unwrap());
        } else {
            self.cpu.bus.joypad.release_button(*self.joypad_map.get(&button).unwrap());
        }
    }
}