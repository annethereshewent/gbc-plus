use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChannelLengthDutyRegister {
    pub initial_timer: u8,
    pub wave_duty: u8
}

impl ChannelLengthDutyRegister {
    pub fn new() -> Self {
        Self {
            initial_timer: 0,
            wave_duty: 0
        }
    }

    pub fn write(&mut self, value: u8) {
        self.wave_duty = (value >> 6) & 0x3;
        self.initial_timer = value & 0x3f;
    }

    pub fn read(&self) -> u8 {
        0x3f | self.wave_duty << 6
    }
}