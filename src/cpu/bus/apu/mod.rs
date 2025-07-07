use audio_master_register::AudioMasterRegister;
use master_volume_vin_register::MasterVolumeVinRegister;
use sound_panning_register::SoundPanningRegister;

pub mod audio_master_register;
pub mod sound_panning_register;
pub mod master_volume_vin_register;

pub struct APU {
    pub nr52: AudioMasterRegister,
    pub nr51: SoundPanningRegister,
    pub nr50: MasterVolumeVinRegister
}

impl APU {
    pub fn new() -> Self {
        Self {
            nr52: AudioMasterRegister::new(),
            nr51: SoundPanningRegister::from_bits_retain(0),
            nr50: MasterVolumeVinRegister::new()
        }
    }
}