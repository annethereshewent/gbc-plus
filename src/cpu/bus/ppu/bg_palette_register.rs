
pub enum BGColor {
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    Black = 3
}

pub struct BGPaletteRegister {
    pub id0: BGColor,
    pub id1: BGColor,
    pub id2: BGColor,
    pub id3: BGColor
}


impl BGPaletteRegister {
    pub fn write(&mut self, value: u8) {
        self.id0 = Self::get_id_color(value & 0x3);
        self.id1 = Self::get_id_color((value >> 2) & 0x3);
        self.id2 = Self::get_id_color((value >> 4) & 0x3);
        self.id3 = Self::get_id_color((value >> 6) & 0x3);
    }

    pub fn new() -> Self {
        Self {
            id0: BGColor::White,
            id1: BGColor::White,
            id2: BGColor::White,
            id3: BGColor::White
        }
    }

    fn get_id_color(value: u8) -> BGColor {
        match value {
            0 => BGColor::White,
            1 => BGColor::LightGray,
            2 => BGColor::DarkGray,
            3 => BGColor::Black,
            _ => unreachable!()
        }
    }
}