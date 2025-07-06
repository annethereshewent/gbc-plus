use super::bg_palette_register::{BGColor, BGPaletteRegister};

pub struct ObjPaletteRegister {
    id1: BGColor,
    id2: BGColor,
    id3: BGColor
}

impl ObjPaletteRegister {
    pub fn write(&mut self, value: u8) {
        self.id1 = BGPaletteRegister::get_id_color((value >> 2) & 0x3);
        self.id2 = BGPaletteRegister::get_id_color((value >> 4) & 0x3);
        self.id3 = BGPaletteRegister::get_id_color((value >> 6) & 0x3);
    }

    pub fn new() -> Self {
        Self {
            id1: BGColor::White,
            id2: BGColor::White,
            id3: BGColor::White
        }
    }
}