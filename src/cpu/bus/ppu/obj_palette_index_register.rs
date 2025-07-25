use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ObjPaletteIndexRegister {
    pub address: u8,
    pub auto_increment: bool
}

impl ObjPaletteIndexRegister {
    pub fn new() -> Self {
        Self {
            address: 0,
            auto_increment: false
        }
    }
    pub fn write(&mut self, value: u8) {
        self.address = value & 0x3f;
        self.auto_increment = (value >> 7) & 0x1 == 1;
    }

    pub fn read(&self) -> u8 {
        self.address | (self.auto_increment as u8) << 7
    }
}