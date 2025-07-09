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

    pub fn new() -> Self {
        Self {
            indexes: [BGColor::White; 4]
        }
    }
}