
use std::{collections::{HashMap, VecDeque}, panic, sync::{Arc, Mutex}};

use gbc_plus::cpu::{bus::joypad::JoypadButtons, CPU};
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
    audio_buffer: Arc<Mutex<VecDeque<f32>>>
}

#[wasm_bindgen]
impl WebEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let joypad_map = HashMap::<usize, JoypadButtons>::new();

        let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));
        Self {
            cpu: CPU::new(audio_buffer.clone(), None),
            joypad_map,
            audio_buffer
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

    pub fn modify_samples(&self, left: &mut [f32], right: &mut [f32]) {
        let mut samples = self.audio_buffer.lock().unwrap();
        let mut left_sample = 0.0;
        let mut right_sample = 0.0;
        if samples.len() > 1 {
            left_sample = samples[samples.len() - 2];
            right_sample = samples[samples.len() - 1];
        }

        let mut is_left_sample = false;

        let mut left_index = 0;
        let mut right_index = 0;

        while let Some(sample) = samples.pop_back() {
            if is_left_sample {
                left[left_index] = sample;
                left_index += 1;
            }  else {
                right[right_index] = sample;
                right_index += 1;
            }
            is_left_sample = !is_left_sample;
        }

        console_log!("this shit still sucks");

        while left_index < left.len() || right_index < right.len() {
            if is_left_sample {
                left[left_index] = left_sample;
                left_index += 1;
            } else {
                right[right_index] = right_sample;
                right_index += 1;
            }

            is_left_sample = !is_left_sample;
        }
    }

    pub fn update_input(&mut self, button: usize, pressed: bool) {
        // if pressed {
        //     self.cpu.bus.joypad.press_button(button);
        // } else {
        //     self.cpu.bus.joypad.release_button(button);
        // }
    }
}