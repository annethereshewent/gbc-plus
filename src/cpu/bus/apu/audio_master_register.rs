pub struct AudioMasterRegister {
    pub audio_on: bool
}

impl AudioMasterRegister {
    pub fn new() -> Self {
        Self {
            audio_on: false
        }
    }

    pub fn write(&mut self, value: u8) {
        self.audio_on = (value >> 7) & 0x1 == 1;
    }

    pub fn read(&self) -> u8 {
        self.audio_on as u8
    }
}