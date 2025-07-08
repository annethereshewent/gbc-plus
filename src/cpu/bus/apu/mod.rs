use audio_master_register::AudioMasterRegister;
use channels::{
    channel1::Channel1,
    channel2::Channel2,
    channel3::Channel3,
    channel4::Channel4,
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
    pub dac_enable: bool,
    pub wave_ram: [u8; 16],
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
            dac_enable: false,
            wave_ram: [0; 16],
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