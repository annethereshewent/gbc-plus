use super::channel_period_high_control_register::ChannelPeriodHighControlRegister;

pub struct Channel3 {
    pub enabled: bool,
    pub nr34: ChannelPeriodHighControlRegister,
    pub period_lo: u8,
    pub length: u8,
    pub output: u8,
}

impl Channel3 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr34: ChannelPeriodHighControlRegister::new(),
            period_lo: 0,
            length: 0,
            output: 0
        }
    }
}