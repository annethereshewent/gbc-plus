use super::{channel_length_duty_register::ChannelLengthDutyRegister, channel_period_high_control_register::ChannelPeriodHighControlRegister, channel_sweep_register::ChannelSweepRegister, channel_volume_register::ChannelVolumeRegister};

pub struct Channel1 {
    pub enabled: bool,
        pub nr12: ChannelVolumeRegister,
        pub nr14: ChannelPeriodHighControlRegister,
        pub nr10: ChannelSweepRegister,
        pub nr11: ChannelLengthDutyRegister,
        pub period_lo: u8,
}

impl Channel1 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr10: ChannelSweepRegister::new(),
            nr11: ChannelLengthDutyRegister::new(),
            nr12: ChannelVolumeRegister::new(),
            nr14: ChannelPeriodHighControlRegister::new(),
            period_lo: 0
        }
    }
}