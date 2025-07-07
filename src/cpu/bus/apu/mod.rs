use audio_master_register::AudioMasterRegister;
use channels::{
    channel1::Channel1,
    channel2::Channel2,
    channel3::Channel3,
    channel4::Channel4,
    channel4_control_register::Channel4ControlRegister,
    channel_freq_random_register::ChannelFreqRandomRegister,
    channel_length_duty_register::ChannelLengthDutyRegister,
    channel_period_high_control_register::ChannelPeriodHighControlRegister,
    channel_sweep_register::ChannelSweepRegister,
    channel_volume_register::ChannelVolumeRegister
};
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
    pub nr34: ChannelPeriodHighControlRegister,
    pub nr44: Channel4ControlRegister,
    pub nr10: ChannelSweepRegister,
    pub nr11: ChannelLengthDutyRegister,
    pub nr21: ChannelLengthDutyRegister,
    pub nr43: ChannelFreqRandomRegister,
    pub dac_enable: bool,
    pub channel1_period_lo: u8,
    pub channel2_period_lo: u8,
    pub channel3_period_lo: u8,
    pub wave_ram: [u8; 16],
    pub channel3_length: u8,
    pub channel3_output: u8,
    pub channel4_length: u8,
    pub channel1: Channel1,
    pub channel2: Channel2,
    pub channel3: Channel3,
    pub channel4: Channel4
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
            nr34: ChannelPeriodHighControlRegister::new(),
            nr44: Channel4ControlRegister::new(),
            nr10: ChannelSweepRegister::new(),
            nr11: ChannelLengthDutyRegister::new(),
            nr21: ChannelLengthDutyRegister::new(),
            nr43: ChannelFreqRandomRegister::new(),
            dac_enable: false,
            channel1_period_lo: 0,
            channel2_period_lo: 0,
            channel3_period_lo: 0,
            wave_ram: [0; 16],
            channel3_length: 0,
            channel3_output: 0,
            channel4_length: 0,
            channel1: Channel1::new(),
            channel2: Channel2::new(),
            channel3: Channel3::new(),
            channel4: Channel4::new()
        }
    }

    pub fn write_dac_enable(&mut self, value: u8) {
        let previous_enable = self.dac_enable;
        self.dac_enable = (value >> 7) & 0x1 == 1;

        if previous_enable && !self.dac_enable {
            self.channel3.enabled = false;
        }
    }
}