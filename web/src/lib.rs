
use std::{collections::{HashMap, VecDeque}, panic, sync::{Arc, Mutex}};

use gbc_plus::cpu::{bus::joypad::JoypadButtons, CPU};
use ringbuf::traits::Consumer;
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
    sample_buffer: Vec<f32>
}

const BUTTON_CROSS: usize = 0;
const BUTTON_SQUARE: usize = 2;
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

        Self {
            cpu: CPU::new(None, None, false),
            joypad_map,
            sample_buffer: Vec::new()
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

    pub fn get_buffer_len(&self) -> usize {
        self.sample_buffer.len()
    }

    pub fn update_input(&mut self, button: usize, pressed: bool) {
        console_log!("hello!");
        if pressed {
            self.cpu.bus.joypad.press_button(*self.joypad_map.get(&button).unwrap());
        } else {
            self.cpu.bus.joypad.release_button(*self.joypad_map.get(&button).unwrap());
        }
    }
}