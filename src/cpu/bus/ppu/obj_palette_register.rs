use super::bg_palette_register::{BGColor, BGPaletteRegister};

pub struct ObjPaletteRegister {
    pub indexes: [BGColor; 4]
}

impl ObjPaletteRegister {
    pub fn write(&mut self, value: u8) {
        for i in 0..self.indexes.len() {
            self.indexes[i] = BGPaletteRegister::get_id_color(value >> (2 * i) & 0x3);
        }
    }

    pub fn read(&self) -> u8 {
        let mut val = 0;
        for i in 1..self.indexes.len() {
            val |= (self.indexes[i] as u8) << (2 * i);
        }

        val
    }

    pub fn new() -> Self {
        Self {
            indexes: [BGColor::White; 4]
        }
    }
}