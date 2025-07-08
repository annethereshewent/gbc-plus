use super::{
    channel_length_duty_register::ChannelLengthDutyRegister,
    channel_period_high_control_register::ChannelPeriodHighControlRegister,
    channel_volume_register::ChannelVolumeRegister
};

pub struct Channel2 {
    pub enabled: bool,
    pub nr22: ChannelVolumeRegister,
    pub nr24: ChannelPeriodHighControlRegister,
    pub nr21: ChannelLengthDutyRegister,
    pub period_lo: u8
}

impl Channel2 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr22: ChannelVolumeRegister::new(),
            nr24: ChannelPeriodHighControlRegister::new(),
            nr21: ChannelLengthDutyRegister::new(),
            period_lo: 0
        }
    }
}