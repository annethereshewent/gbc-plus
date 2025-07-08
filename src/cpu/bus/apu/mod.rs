use std::{collections::VecDeque, sync::{Arc, Mutex}};

use audio_master_register::AudioMasterRegister;
use channels::{
    channel1::Channel1,
    channel2::Channel2,
    channel3::Channel3,
    channel4::Channel4,
};
use master_volume_vin_register::MasterVolumeVinRegister;
use sound_panning_register::SoundPanningRegister;

use crate::cpu::CLOCK_SPEED;

pub mod audio_master_register;
pub mod sound_panning_register;
pub mod master_volume_vin_register;
pub mod channels;

pub const TICKS_PER_SAMPLE: usize = CLOCK_SPEED / 44100;
pub const NUM_SAMPLES: usize = 8192 * 2;

pub struct APU {
    pub nr52: AudioMasterRegister,
    pub nr51: SoundPanningRegister,
    pub nr50: MasterVolumeVinRegister,
    pub wave_ram: [u8; 16],
    pub channel1: Channel1,
    pub channel2: Channel2,
    pub channel3: Channel3,
    pub channel4: Channel4,
    cycles: usize,
    pub audio_buffer: Arc<Mutex<VecDeque<f32>>>
}

impl APU {
    pub fn new(audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
        Self {
            nr52: AudioMasterRegister::new(),
            nr51: SoundPanningRegister::from_bits_retain(0),
            nr50: MasterVolumeVinRegister::new(),
            wave_ram: [0; 16],
            channel1: Channel1::new(),
            channel2: Channel2::new(),
            channel3: Channel3::new(),
            channel4: Channel4::new(),
            cycles: 0,
            audio_buffer
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.channel1.tick(cycles);
        self.channel2.tick(cycles);
        self.channel3.tick(cycles);
        self.channel4.tick(cycles);

        self.cycles += cycles;

        if self.cycles >= TICKS_PER_SAMPLE {
            self.cycles -= TICKS_PER_SAMPLE;

            let ch1_sample = self.channel1.generate_sample();
            let ch2_sample = self.channel2.generate_sample();

            let sample = (ch1_sample + ch2_sample) / 2.0;

            let left_sample = sample * self.nr51.contains(SoundPanningRegister::CH1_LEFT) as i16 as f32;
            let right_sample = sample * self.nr51.contains(SoundPanningRegister::CH1_RIGHT) as i16 as f32;

            let mut audio_buffer = self.audio_buffer.lock().unwrap();

            if audio_buffer.len() < NUM_SAMPLES {
                audio_buffer.push_back(left_sample);
            }
            if audio_buffer.len() < NUM_SAMPLES {
                audio_buffer.push_back(right_sample);
            }
        }
    }
}