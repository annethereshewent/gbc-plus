pub struct Channel4ControlRegister {
    pub length_enable: bool,
    pub trigger: bool
}

impl Channel4ControlRegister {
    pub fn new() -> Self {
        Self {
            length_enable: false,
            trigger: false
        }
    }

    pub fn write(&mut self, value: u8) {
        self.length_enable = (value >> 6) & 0x1 == 1;
        self.trigger = (value >> 7) & 0x1 == 1;
    }

    pub fn read(&self) -> u8 {
        (self.length_enable as u8) << 6
    }
}