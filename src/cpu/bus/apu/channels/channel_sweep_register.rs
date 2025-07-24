use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum SweepDirection {
    Addition = 0,
    Subtraction = 1
}

#[derive(Serialize, Deserialize)]
pub struct ChannelSweepRegister {
    pub step: u8,
    pub direction: SweepDirection,
    pub pace: u8
}

impl ChannelSweepRegister {
    pub fn new() -> Self {
        Self {
            step: 0,
            direction: SweepDirection::Addition,
            pace: 0
        }
    }

    pub fn write(&mut self, value: u8) {
        self.step = value & 0x7;
        self.pace = (value >> 4) & 0x7;
        self.direction = match (value >> 3) & 0x1 {
            0 => SweepDirection::Addition,
            1 => SweepDirection::Subtraction,
            _ => unreachable!()
        }
    }

    pub fn read(&self) -> u8 {
        self.step | (self.direction as u8) << 3 | self.pace << 4 | 1 << 7
    }
}