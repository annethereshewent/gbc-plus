
use std::{collections::HashMap, panic, sync::Arc};

use gbc_plus::cpu::{bus::{apu::NUM_SAMPLES, cartridge::mbc::MBC, joypad::JoypadButtons}, CPU};
use ringbuf::{storage::Heap, traits::{Consumer, Split}, wrap::caching::Caching, HeapRb, SharedRb};
use wasm_bindgen::prelude::*;

extern crate gbc_plus;
extern crate console_error_panic_hook;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct WebEmulator {
    cpu: CPU,
    joypad_map: HashMap<usize, JoypadButtons>,
    sample_buffer: Vec<f32>,
    consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
    is_paused: bool,
    save_state: Vec<u8>
}

const BUTTON_A: usize = 0;
const BUTTON_B: usize = 2;
const SELECT: usize = 8;
const START: usize = 9;
const UP: usize = 12;
const DOWN: usize = 13;
const LEFT: usize = 14;
const RIGHT: usize = 15;

#[wasm_bindgen]
impl WebEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

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

        let ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);

        let (producer, consumer) = ringbuffer.split();

        Self {
            cpu: CPU::new(producer, None, None, false, false),
            joypad_map,
            sample_buffer: Vec::new(),
            consumer,
            is_paused: false,
            save_state: Vec::new()
        }
    }

    pub fn set_pause(&mut self, value: bool) {
        self.is_paused = value;
    }

    pub fn change_palette(&mut self, index: usize) {
        self.cpu.bus.ppu.set_dmg_palette(index);
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
        self.cpu.load_rom(data, false);
    }

    pub fn step_frame(&mut self) {
        if !self.is_paused {
            self.cpu.step_frame();

            self.cpu.bus.ppu.frame_finished = false;
        }
    }

    pub fn load_save_state(&mut self, data: &[u8]) {
        self.cpu.load_save_state(&data);

        let ringbuffer = HeapRb::<f32>::new(NUM_SAMPLES);
        let (producer, consumer) = ringbuffer.split();

        self.consumer = consumer;
        self.cpu.bus.apu.producer = Some(producer);
    }

    pub fn create_save_state(&mut self) -> *const u8 {
        (self.save_state, _) = self.cpu.create_save_state();

        self.save_state.as_ptr()
    }

    pub fn save_state_length(&self) -> usize {
        self.save_state.len()
    }

    pub fn reload_rom(&mut self, data: &[u8]) {
        self.cpu.reload_rom(data);
    }

    pub fn get_screen(&self) -> *const u8 {
        self.cpu.bus.ppu.picture.data.as_ptr()
    }

    pub fn get_screen_length(&self) -> usize {
        self.cpu.bus.ppu.picture.data.len()
    }

    pub fn read_ringbuffer(&mut self) -> *mut f32 {
        self.sample_buffer = Vec::new();

        for sample in self.consumer.pop_iter() {
            self.sample_buffer.push(sample);
        }

        self.sample_buffer.as_mut_ptr()
    }

    pub fn pop_sample(&mut self) -> Option<f32> {
        return self.consumer.try_pop()
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

    pub fn is_rtc_dirty(&self) -> bool {
        match &self.cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc3) => mbc3.is_dirty,
            _ => false
        }
    }

    pub fn clear_rtc_dirty(&mut self) {
        match &mut self.cpu.bus.cartridge.mbc {
            MBC::MBC3(mbc3) => mbc3.is_dirty = false,
            _ => ()
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