use std::{collections::VecDeque, sync::{Arc, Mutex}};

use audio_master_register::AudioMasterRegister;
use channels::{
    pulse_channel::PulseChannel,
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
pub const HZ_512: usize = CLOCK_SPEED / 512;

pub struct APU {
    pub nr52: AudioMasterRegister,
    pub nr51: SoundPanningRegister,
    pub nr50: MasterVolumeVinRegister,
    pub channel1: PulseChannel<true>,
    pub channel2: PulseChannel<false>,
    pub channel3: Channel3,
    pub channel4: Channel4,
    cycles: usize,
    pub audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    sequencer_cycles: usize,
    sequencer_step: usize
}

impl APU {
    pub fn new(audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
        Self {
            nr52: AudioMasterRegister::new(),
            nr51: SoundPanningRegister::from_bits_retain(0),
            nr50: MasterVolumeVinRegister::new(),
            channel1: PulseChannel::new(),
            channel2: PulseChannel::new(),
            channel3: Channel3::new(),
            channel4: Channel4::new(),
            cycles: 0,
            audio_buffer,
            sequencer_cycles: 0,
            sequencer_step: 0
        }
    }

    fn generate_samples(&mut self) {
        let ch1_sample = self.channel1.generate_sample();
        let ch2_sample = self.channel2.generate_sample();
        let ch3_sample = self.channel3.generate_sample();

        let sample = (ch1_sample + ch2_sample + ch3_sample) / 4.0;

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
    // https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html
    // Step   Length Ctr  Vol Env     Sweep
    // ---------------------------------------
    // 0      Clock       -           -
    // 1      -           -           -
    // 2      Clock       -           Clock
    // 3      -           -           -
    // 4      Clock       -           -
    // 5      -           -           -
    // 6      Clock       -           Clock
    // 7      -           Clock       -
    // ---------------------------------------
    // Rate   256 Hz      64 Hz       128 Hz
    fn update_frame_sequencer(&mut self) {
        match self.sequencer_step {
            0 => self.clock_lengths(),
            1 => (),
            2 => {
                self.clock_lengths();
                self.clock_sweep();
            }
            3 => (),
            4 => self.clock_lengths(),
            5 => (),
            6 => {
                self.clock_lengths();
                self.clock_sweep();
            }
            7 => self.clock_envelopes(),
            _ => unreachable!()
        }

        self.sequencer_step = (self.sequencer_step + 1) & 0x7;
    }

    fn clock_lengths(&mut self) {
        self.channel1.tick_length();
        self.channel2.tick_length();
        self.channel3.tick_length();
    }

    fn clock_sweep(&mut self) {
        self.channel1.tick_sweep();
    }

    fn clock_envelopes(&mut self) {
        self.channel1.tick_envelope();
        self.channel2.tick_envelope();
    }

    pub fn tick(&mut self, cycles: usize) {
        self.channel1.tick(cycles);
        self.channel2.tick(cycles);
        self.channel3.tick(cycles);
        self.channel4.tick(cycles);

        self.cycles += cycles;
        self.sequencer_cycles += cycles;

        if self.cycles >= TICKS_PER_SAMPLE {
            self.cycles -= TICKS_PER_SAMPLE;

            if self.nr52.audio_on {
                self.generate_samples();
            }
        }

        if self.sequencer_cycles >= HZ_512 {
            self.sequencer_cycles -= HZ_512;

            self.update_frame_sequencer();
        }
    }
}