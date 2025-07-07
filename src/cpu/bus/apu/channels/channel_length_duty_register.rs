pub struct ChannelLengthDutyRegister {
    initial_timer: u8,
    wave_duty: u8
}

impl ChannelLengthDutyRegister {
    pub fn new() -> Self {
        Self {
            initial_timer: 0,
            wave_duty: 0
        }
    }

    pub fn write(&mut self, value: u8) {
        self.wave_duty = value & 0x3f;
        self.initial_timer = (value >> 6) & 0x3;
    }
}