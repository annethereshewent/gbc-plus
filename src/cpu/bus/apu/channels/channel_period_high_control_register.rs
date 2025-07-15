pub struct ChannelPeriodHighControlRegister {
    pub length_enable: bool,
    pub trigger: bool
}

impl ChannelPeriodHighControlRegister {
    pub fn new() -> Self {
        Self {
            length_enable: false,
            trigger: false
        }
    }

    pub fn write(&mut self, value: u8) -> bool {
        self.length_enable = (value >> 6) & 0x1 == 1;
        self.trigger = (value >> 7) & 0x1 == 1;

        self.trigger
    }

    pub fn read(&self) -> u8 {
        0x3f | (self.length_enable as u8) << 6 | 1 << 7
    }
}