use super::{channel4_control_register::Channel4ControlRegister, channel_freq_random_register::ChannelFreqRandomRegister, channel_volume_register::ChannelVolumeRegister};

pub struct Channel4 {
    pub enabled: bool,
    pub nr42: ChannelVolumeRegister,
    pub nr44: Channel4ControlRegister,
    pub nr43: ChannelFreqRandomRegister,
    pub length: u8,
}

impl Channel4 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr42: ChannelVolumeRegister::new(),
            nr44: Channel4ControlRegister::new(),
            nr43: ChannelFreqRandomRegister::new(),
            length: 0
        }
    }
}