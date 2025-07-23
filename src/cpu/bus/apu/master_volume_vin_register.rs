use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MasterVolumeVinRegister {
    pub right_volume: u8,
    pub left_volume: u8,
    pub vin_left: bool,
    pub vin_right: bool
}

impl MasterVolumeVinRegister {
    pub fn new() -> Self {
        Self {
            right_volume: 0,
            left_volume: 0,
            vin_left: false,
            vin_right: false
        }
    }

    pub fn write(&mut self, value: u8) {
        self.right_volume = value & 0x7;
        self.vin_right = (value >> 3) & 0x1 == 1;
        self.left_volume = (value >> 4) & 0x7;
        self.vin_left = (value >> 7) & 0x1 == 1;
    }

    pub fn read(&self) -> u8 {
        self.right_volume | (self.vin_right as u8) << 3 | self.left_volume << 4 | (self.vin_left as u8) << 7
    }
}