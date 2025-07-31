use serde::{Deserialize, Serialize};

use super::{
    channel4_control_register::Channel4ControlRegister,
    channel_freq_random_register::{
        ChannelFreqRandomRegister,
        LFSRWidth
    },
    channel_volume_register::{
        ChannelVolumeRegister,
        EnvelopeDirection
    }
};

#[derive(Serialize, Deserialize)]
pub struct Channel4 {
    pub enabled: bool,
    pub nr42: ChannelVolumeRegister,
    pub nr44: Channel4ControlRegister,
    pub nr43: ChannelFreqRandomRegister,
    pub length: u8,
    frequency_timer: isize,
    envelope_timer: usize,
    current_timer: usize,
    current_volume: usize,
    lfsr: u16,
    output: u16
}

impl Channel4 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr42: ChannelVolumeRegister::new(),
            nr44: Channel4ControlRegister::new(),
            nr43: ChannelFreqRandomRegister::new(),
            length: 0,
            frequency_timer: 0,
            envelope_timer: 0,
            current_timer: 0,
            current_volume: 0,
            lfsr: 1,
            output: 0
        }
    }

    pub fn write_length(&mut self, value: u8) {
        self.length = value & 0x3f;

        self.current_timer = self.length as usize;
    }

    pub fn write_control(&mut self, value: u8) {
        self.nr44.write(value);
    }

    fn restart_channel(&mut self) {
        self.nr44.trigger = false;

        self.enabled = true;
        self.frequency_timer = self.get_frequency_timer();

        if self.current_timer >= 64 {
            self.current_timer = 0;
        }

        self.envelope_timer = self.nr42.sweep_pace as usize;
        self.current_volume = self.nr42.initial_volume as usize;
        self.lfsr = 1;
    }

    pub fn write_volume(&mut self, value: u8) {
        self.nr42.write(value);
    }

    pub fn tick_length(&mut self) {
        if self.nr44.length_enable {
            self.current_timer += 1;

            if self.current_timer >= 64 {
                self.enabled = false;
            }
        }
    }

    pub fn generate_sample(&self) -> f32 {
        if self.enabled {
            self.output as f32
        } else {
            0.0
        }
    }

    pub fn tick_envelope(&mut self) {
        self.envelope_timer -= 1;

        if self.envelope_timer == 0 {
            self.envelope_timer = self.nr42.sweep_pace as usize;

            if self.nr42.env_dir == EnvelopeDirection::Decrease {
                if self.current_volume != 0 {
                    self.current_volume -= 1;
                }
            } else {
                if self.current_volume != 15 {
                    self.current_volume += 1;
                }
            }
        }
    }

    fn get_frequency_timer(&self) -> isize {
        (self.nr43.clock_divider as isize) << self.nr43.clock_shift as isize
    }

    pub fn tick(&mut self, cycles: usize) {
        if self.nr44.trigger {
            self.restart_channel();
        }

        self.frequency_timer -= cycles as isize;

        if self.frequency_timer <= 0 {
            self.frequency_timer = self.get_frequency_timer();

            let result = (self.lfsr & 0x1) ^ ((self.lfsr >> 1) & 0x1);

            self.lfsr &= !(1 << 15);

            self.lfsr |= result << 15;

            if self.nr43.lfsr_width == LFSRWidth::Bit7 {
                self.lfsr &= !(1 << 7);
                self.lfsr |= result << 7;
            }

            self.lfsr >>= 1;

            self.output = (self.lfsr & 0x1) * self.current_volume as u16;
        }
    }
}