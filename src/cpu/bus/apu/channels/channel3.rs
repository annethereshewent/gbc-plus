use super::channel_period_high_control_register::ChannelPeriodHighControlRegister;

pub struct Channel3 {
    pub enabled: bool,
    pub nr34: ChannelPeriodHighControlRegister,
    pub period: u16,
    pub length: u8,
    pub output: u8,
    pub dac_enable: bool,
}

impl Channel3 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr34: ChannelPeriodHighControlRegister::new(),
            period: 0,
            length: 0,
            output: 0,
            dac_enable: false
        }
    }

    pub fn write_period_high_control(&mut self, value: u8) {
        self.period &= 0xff;
        self.period |= ((value as u16) & 0x7) << 8;

        self.nr34.write(value);
    }

    pub fn write_dac_enable(&mut self, value: u8) {
        let previous_enable = self.dac_enable;
        self.dac_enable = (value >> 7) & 0x1 == 1;

        if previous_enable && !self.dac_enable {
            self.enabled = false;
        } else if !previous_enable && self.dac_enable {
            self.enabled = true;
        }
    }

    pub fn tick(&mut self, cycles: usize) {

    }
}