pub struct AudioMasterRegister {
    ch1_on: bool,
    ch2_on: bool,
    ch3_on: bool,
    ch4_on: bool,
    audio_on: bool
}

impl AudioMasterRegister {
    pub fn new() -> Self {
        Self {
            ch1_on: false,
            ch2_on: false,
            ch3_on: false,
            ch4_on: false,
            audio_on: false
        }
    }

    pub fn write(&mut self, value: u8) {
        self.audio_on = (value >> 7) & 0x1 == 1;
    }

    pub fn read(&self) -> u8 {
        self.ch1_on as u8 | (self.ch2_on as u8) << 1 | (self.ch3_on as u8) << 2 | (self.ch4_on as u8) << 3  | (self.audio_on as u8) << 7
    }
}