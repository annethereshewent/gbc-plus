use bitflags::bitflags;

bitflags! {
    pub struct JoypadButtons: u8 {
        const A_RIGHT = 1;
        const B_LEFT = 1 << 1;
        const SELECT_UP = 1 << 2;
        const START_DOWN = 1 << 3;

    }
}

pub struct Joypad {
    pub select_buttons: bool,
    pub select_dpad: bool,
    pub joypad_buttons: JoypadButtons
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            select_buttons: false,
            select_dpad: false,
            joypad_buttons: JoypadButtons::from_bits_retain(0xf)
        }
    }

    pub fn write(&mut self, value: u8) {
        self.select_buttons = (value >> 5) & 0b1 == 0;
        self.select_dpad = (value >> 4) & 0b1 == 0;
    }

    pub fn read(&self) -> u8 {
        self.joypad_buttons.bits() | (self.select_dpad as u8) << 4 | (self.select_buttons as u8) << 5
    }
}