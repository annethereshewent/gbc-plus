use audio_master_register::AudioMasterRegister;
use channels::{channel4_control_register::Channel4ControlRegister, channel_period_high_control_register::ChannelPeriodHighControlRegister, channel_sweep_register::ChannelSweepRegister, channel_volume_register::ChannelVolumeRegister};
use master_volume_vin_register::MasterVolumeVinRegister;
use sound_panning_register::SoundPanningRegister;

pub mod audio_master_register;
pub mod sound_panning_register;
pub mod master_volume_vin_register;
pub mod channels;

pub struct APU {
    pub nr52: AudioMasterRegister,
    pub nr51: SoundPanningRegister,
    pub nr50: MasterVolumeVinRegister,
    pub nr12: ChannelVolumeRegister,
    pub nr22: ChannelVolumeRegister,
    pub nr42: ChannelVolumeRegister,
    pub nr14: ChannelPeriodHighControlRegister,
    pub nr24: ChannelPeriodHighControlRegister,
    pub nr44: Channel4ControlRegister,
    pub nr10: ChannelSweepRegister
}

impl APU {
    pub fn new() -> Self {
        Self {
            nr52: AudioMasterRegister::new(),
            nr51: SoundPanningRegister::from_bits_retain(0),
            nr50: MasterVolumeVinRegister::new(),
            nr12: ChannelVolumeRegister::new(),
            nr22: ChannelVolumeRegister::new(),
            nr42: ChannelVolumeRegister::new(),
            nr14: ChannelPeriodHighControlRegister::new(),
            nr24: ChannelPeriodHighControlRegister::new(),
            nr44: Channel4ControlRegister::new(),
            nr10: ChannelSweepRegister::new()
        }
    }
}