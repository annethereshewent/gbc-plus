use std::sync::Arc;

use audio_master_register::AudioMasterRegister;
use channels::{
    pulse_channel::PulseChannel,
    channel3::Channel3,
    channel4::Channel4,
};
use master_volume_vin_register::MasterVolumeVinRegister;
use ringbuf::{storage::Heap, traits::Producer, wrap::caching::Caching, SharedRb};
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
    pub producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
    sequencer_cycles: usize,
    sequencer_step: usize,
    is_ios: bool
}

impl APU {
    pub fn new(producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>, is_ios: bool) -> Self {
        Self {
            nr52: AudioMasterRegister::new(),
            nr51: SoundPanningRegister::from_bits_retain(0),
            nr50: MasterVolumeVinRegister::new(),
            channel1: PulseChannel::new(),
            channel2: PulseChannel::new(),
            channel3: Channel3::new(),
            channel4: Channel4::new(),
            cycles: 0,
            sequencer_cycles: 0,
            sequencer_step: 0,
            producer,
            is_ios
        }
    }

    fn generate_samples(&mut self) {
        let mut ch1_left_sample = self.channel1.generate_sample() * self.nr51.contains(SoundPanningRegister::CH1_LEFT) as i16 as f32;
        let mut ch1_right_sample = self.channel1.generate_sample() * self.nr51.contains(SoundPanningRegister::CH1_RIGHT) as i16 as f32;

        ch1_left_sample = (ch1_left_sample / 7.5) - 1.0;
        ch1_right_sample = (ch1_right_sample / 7.5) - 1.0;

        let mut ch2_left_sample = self.channel2.generate_sample() * self.nr51.contains(SoundPanningRegister::CH2_LEFT) as i16 as f32;
        let mut ch2_right_sample = self.channel2.generate_sample() * self.nr51.contains(SoundPanningRegister::CH2_RIGHT) as i16 as f32;

        ch2_left_sample = (ch2_left_sample / 7.5) - 1.0;
        ch2_right_sample = (ch2_right_sample / 7.5) - 1.0;

        let mut ch3_left_sample = self.channel3.generate_sample() * self.nr51.contains(SoundPanningRegister::CH3_LEFT) as i16 as f32;
        let mut ch3_right_sample = self.channel3.generate_sample() * self.nr51.contains(SoundPanningRegister::CH3_RIGHT) as i16 as f32;

        ch3_left_sample = (ch3_left_sample / 7.5) - 1.0;
        ch3_right_sample = (ch3_right_sample / 7.5) - 1.0;

        let mut ch4_left_sample = self.channel4.generate_sample() * self.nr51.contains(SoundPanningRegister::CH4_LEFT) as i16 as f32;
        let mut ch4_right_sample = self.channel4.generate_sample() * self.nr51.contains(SoundPanningRegister::CH4_RIGHT) as i16 as f32;

        ch4_left_sample = (ch4_left_sample / 7.5) - 1.0;
        ch4_right_sample = (ch4_right_sample / 7.5) - 1.0;

        let mut left_sample = (ch1_left_sample + ch2_left_sample + ch3_left_sample + ch4_left_sample) / 4.0;
        let mut right_sample = (ch1_right_sample + ch2_right_sample + ch3_right_sample + ch4_right_sample) / 4.0;

        left_sample *= self.nr50.left_volume as f32 / 7.0;
        right_sample *= self.nr50.right_volume as f32 / 7.0;

        left_sample = left_sample.clamp(-1.0, 1.0);
        right_sample = right_sample.clamp(-1.0, 1.0);

        self.push_ringbuffer(left_sample, right_sample);
    }

    /*
     *  This is a hacky method that only outputs positive sample values.
     *  This is because iOS does *not* like dealing with nonzero samples
     *  that are all the same value, instead of treating it like silence
     *  iOS will produce a ton of consistent pops. however apple seems to
     *  like this function well enough and audio sounds mostly fine,
     *  so sticking with this for now.
     *  TODO: find a better way to do this.
     */
    pub fn generate_ios_samples(&mut self) {
        let ch1_left_sample = self.channel1.generate_sample() * self.nr51.contains(SoundPanningRegister::CH1_LEFT) as i16 as f32;
        let ch1_right_sample = self.channel1.generate_sample() * self.nr51.contains(SoundPanningRegister::CH1_RIGHT) as i16 as f32;

        let ch2_left_sample = self.channel2.generate_sample() * self.nr51.contains(SoundPanningRegister::CH2_LEFT) as i16 as f32;
        let ch2_right_sample = self.channel2.generate_sample() * self.nr51.contains(SoundPanningRegister::CH2_RIGHT) as i16 as f32;

        let ch3_left_sample = self.channel3.generate_sample() * self.nr51.contains(SoundPanningRegister::CH3_LEFT) as i16 as f32;
        let ch3_right_sample = self.channel3.generate_sample() * self.nr51.contains(SoundPanningRegister::CH3_RIGHT) as i16 as f32;

        let ch4_left_sample = self.channel4.generate_sample() * self.nr51.contains(SoundPanningRegister::CH4_LEFT) as i16 as f32;
        let ch4_right_sample = self.channel4.generate_sample() * self.nr51.contains(SoundPanningRegister::CH4_RIGHT) as i16 as f32;


        let mut left_sample = (ch1_left_sample + ch2_left_sample + ch3_left_sample + ch4_left_sample) / 4.0;
        let mut right_sample =(ch1_right_sample + ch2_right_sample + ch3_right_sample + ch4_right_sample) / 4.0;

        left_sample /= 15.0;
        right_sample /= 15.0;

        left_sample *= self.nr50.left_volume as f32 / 7.0;
        right_sample *= self.nr50.right_volume as f32 / 7.0;

        left_sample = left_sample.clamp(0.0, 1.0);
        right_sample = right_sample.clamp(0.0, 1.0);

        self.push_ringbuffer(left_sample, right_sample);

    }

    fn push_ringbuffer(&mut self, left_sample: f32, right_sample: f32) {
        self.producer.try_push(left_sample).unwrap_or(());
        self.producer.try_push(right_sample).unwrap_or(());
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

    pub fn read_channel_status(&self) -> u8 {
        let audio_on = self.nr52.read();

        self.channel1.enabled as u8 | (self.channel2.enabled as u8) << 1 | (self.channel3.enabled as u8) << 2 | (self.channel3.enabled as u8) << 3 | 0x7 << 4 | audio_on << 7
    }

    pub fn write_audio_master(&mut self, value: u8) {
        let previous_audio_on = self.nr52.audio_on;
        self.nr52.write(value);

        if previous_audio_on && !self.nr52.audio_on {
            self.channel1.enabled = false;
            self.channel2.enabled = false;
            self.channel3.enabled = false;
            self.channel4.enabled = false;

            self.reset_registers();
        }
    }

    fn reset_registers(&mut self) {
        self.nr50.write(0);
        self.nr51 = SoundPanningRegister::from_bits_truncate(0);
        self.nr52.write(0);

        if let Some(nrx0) = &mut self.channel1.nrx0 {
            nrx0.write(0);
        }

        self.channel1.nrx1.write(0);
        self.channel1.nrx2.write(0);
        self.channel1.nrx4.write(0);
        self.channel1.period = 0;

        self.channel2.nrx1.write(0);
        self.channel2.nrx2.write(0);
        self.channel2.nrx4.write(0);
        self.channel2.period = 0;

        self.channel3.dac_enable = false;
        self.channel3.nr34.write(0);
        self.channel3.period = 0;
        self.channel3.length = 0;
        self.channel3.output = None;

        self.channel4.nr42.write(0);
        self.channel4.nr43.write(0);
        self.channel4.nr44.write(0);
    }

    // i wanted to dry up all these tick_length and tick_envelope methods, but rust literally will *not* let me.
    // it complains constantly about the damn borrow checker. so shitty inefficient code it is! thanks rust.
    fn clock_lengths(&mut self) {
        self.channel1.tick_length();
        self.channel2.tick_length();
        self.channel3.tick_length();
        self.channel4.tick_length();
    }

    fn clock_sweep(&mut self) {
        self.channel1.tick_sweep();
    }

    fn clock_envelopes(&mut self) {
        self.channel1.tick_envelope();
        self.channel2.tick_envelope();
        self.channel4.tick_envelope();
    }

    pub fn tick(&mut self, cycles: usize) {
        self.cycles += cycles;
        self.sequencer_cycles += cycles;

        self.channel1.tick(cycles);
        self.channel2.tick(cycles);
        self.channel3.tick(cycles);
        self.channel4.tick(cycles);

        if self.sequencer_cycles >= HZ_512 {
            self.sequencer_cycles -= HZ_512;

            self.update_frame_sequencer();
        }

        if self.cycles >= TICKS_PER_SAMPLE {
            self.cycles -= TICKS_PER_SAMPLE;
            if self.is_ios { self.generate_ios_samples(); } else { self.generate_samples(); }
        }
    }
}