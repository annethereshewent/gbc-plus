

#[derive(Copy, Clone)]
pub enum BGColor {
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    Black = 3
}

pub struct BGPaletteRegister {
    pub indexes: [BGColor; 4]
}


impl BGPaletteRegister {
    pub fn write(&mut self, value: u8) {
        for i in 0..self.indexes.len() {
            self.indexes[i] = Self::get_id_color((value >> (i * 2)) & 0x3);
        }
    }

    pub fn new() -> Self {
        Self {
            indexes: [BGColor::White; 4]
        }
    }

    pub fn read(&self) -> u8 {
        let mut value = 0;

        for i in 0..self.indexes.len() {
            value |= (self.indexes[i] as u8) << (i * 2);
        }

        value
    }

    pub fn get_id_color(value: u8) -> BGColor {
        match value {
            0 => BGColor::White,
            1 => BGColor::LightGray,
            2 => BGColor::DarkGray,
            3 => BGColor::Black,
            _ => unreachable!()
        }
    }
}