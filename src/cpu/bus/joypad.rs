use std::collections::HashMap;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Copy, Clone, Serialize, Deserialize)]
    pub struct JoypadRegister: u8 {
        const A_RIGHT = 1;
        const B_LEFT = 1 << 1;
        const SELECT_UP = 1 << 2;
        const START_DOWN = 1 << 3;
    }
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Serialize,
    Deserialize
)]
pub enum JoypadButtons {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
    None
}

#[derive(Serialize, Deserialize)]
pub struct Joypad {
    pub select_buttons: bool,
    pub select_dpad: bool,
    pub joypad_register: JoypadRegister,
    pressed_buttons: HashMap<JoypadButtons, bool>
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            select_buttons: false,
            select_dpad: false,
            pressed_buttons: HashMap::new(),
            joypad_register: JoypadRegister::from_bits_retain(0xf)
        }
    }

    pub fn write(&mut self, value: u8) {
        self.select_buttons = (value >> 5) & 0b1 == 0;
        self.select_dpad = (value >> 4) & 0b1 == 0;
    }

    pub fn read(&mut self) -> u8 {
        if self.select_buttons {
            self.joypad_register.set(JoypadRegister::A_RIGHT, !*self.pressed_buttons.get(&JoypadButtons::A).unwrap_or(&false));
            self.joypad_register.set(JoypadRegister::B_LEFT, !*self.pressed_buttons.get(&JoypadButtons::B).unwrap_or(&false));
            self.joypad_register.set(JoypadRegister::START_DOWN, !*self.pressed_buttons.get(&JoypadButtons::Start).unwrap_or(&false));
            self.joypad_register.set(JoypadRegister::SELECT_UP, !*self.pressed_buttons.get(&JoypadButtons::Select).unwrap_or(&false));
        } else {
            self.joypad_register.set(JoypadRegister::A_RIGHT, !*self.pressed_buttons.get(&JoypadButtons::Right).unwrap_or(&false));
            self.joypad_register.set(JoypadRegister::B_LEFT, !*self.pressed_buttons.get(&JoypadButtons::Left).unwrap_or(&false));
            self.joypad_register.set(JoypadRegister::START_DOWN, !*self.pressed_buttons.get(&JoypadButtons::Down).unwrap_or(&false));
            self.joypad_register.set(JoypadRegister::SELECT_UP, !*self.pressed_buttons.get(&JoypadButtons::Up).unwrap_or(&false));
        }

        self.joypad_register.bits() | (self.select_dpad as u8) << 4 | (self.select_buttons as u8) << 5
    }

    pub fn press_button(&mut self, button: JoypadButtons) {
        self.pressed_buttons.insert(button, true);
    }

    pub fn release_button(&mut self, button: JoypadButtons) {
        self.pressed_buttons.remove(&button);
    }
}