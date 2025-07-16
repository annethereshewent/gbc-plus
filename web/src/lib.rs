use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}};

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
    joypad_map: HashMap<usize, JoypadButtons>
}

#[wasm_bindgen]
impl WebEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let joypad_map = HashMap::<usize, JoypadButtons>::new();

        let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));
        Self {
            cpu: CPU::new(audio_buffer, None),
            joypad_map
        }
    }

    pub fn step_frame(&mut self) {
        self.cpu.step_frame();

        self.cpu.bus.ppu.frame_finished = false;
    }

    pub fn get_screen(&self) -> *const u8 {
        self.cpu.bus.ppu.picture.data.as_ptr()
    }

    pub fn modify_samples(&self, left: &mut [f32], right: &mut [f32]) {

    }

    pub fn update_input(&mut self, button: usize, pressed: bool) {
        // if pressed {
        //     self.cpu.bus.joypad.press_button(button);
        // } else {
        //     self.cpu.bus.joypad.release_button(button);
        // }
    }
}