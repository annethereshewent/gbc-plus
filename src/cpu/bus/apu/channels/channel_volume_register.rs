use serde::{Deserialize, Serialize};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EnvelopeDirection {
    Decrease = 0,
    Increase = 1
}

#[derive(Serialize, Deserialize)]
pub struct ChannelVolumeRegister {
    pub sweep_pace: u8,
    pub env_dir: EnvelopeDirection,
    pub initial_volume: u8
}

impl ChannelVolumeRegister {
    pub fn new() -> Self {
        Self {
            sweep_pace: 0,
            env_dir: EnvelopeDirection::Decrease,
            initial_volume: 0
        }
    }

    pub fn write(&mut self, value: u8) {
        self.sweep_pace = value & 0x7;
        self.env_dir = match (value >> 3) & 0b1 {
            0 => EnvelopeDirection::Decrease,
            1 => EnvelopeDirection::Increase,
            _ => unreachable!()
        };

        self.initial_volume = (value >> 4) & 0xf;
    }

    pub fn read(&self) -> u8 {
        self.sweep_pace | (self.env_dir as u8) << 3 | self.initial_volume << 4
    }
}