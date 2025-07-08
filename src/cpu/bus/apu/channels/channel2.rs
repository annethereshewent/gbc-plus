use crate::cpu::CLOCK_SPEED;

use super::{
    channel1::DUTY_PATTERNS, channel_length_duty_register::ChannelLengthDutyRegister, channel_period_high_control_register::ChannelPeriodHighControlRegister, channel_volume_register::{ChannelVolumeRegister, EnvelopeDirection}
};

pub struct Channel2 {
    pub enabled: bool,
    pub nr22: ChannelVolumeRegister,
    pub nr24: ChannelPeriodHighControlRegister,
    pub nr21: ChannelLengthDutyRegister,
    pub period: u16,
    frequency_timer: isize,
    envelope_cycles: usize,
    duty_step: usize,
    length_cycles: usize,
    current_timer: usize,
    current_volume: usize,
    envelope_timer: usize
}

impl Channel2 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr22: ChannelVolumeRegister::new(),
            nr24: ChannelPeriodHighControlRegister::new(),
            nr21: ChannelLengthDutyRegister::new(),
            period: 0,
            frequency_timer: 0,
            envelope_cycles: 0,
            duty_step: 0,
            length_cycles: 0,
            current_timer: 0,
            current_volume: 0,
            envelope_timer: 0
        }
    }

    pub fn write_volume_register(&mut self, value: u8) {
        self.nr22.write(value);

        self.current_volume = self.nr22.initial_volume as usize;
    }

    pub fn write_period_high_control(&mut self, value: u8) {
        self.period &= 0xff;
        self.period |= ((value & 0x7) as u16) << 8;

        self.nr24.write(value);
    }

    pub fn write_length_register(&mut self, value: u8) {
        self.nr21.write(value);
    }

    pub fn generate_sample(&mut self) -> f32 {
        if self.enabled {
           let bit = DUTY_PATTERNS[self.nr21.wave_duty as usize][self.duty_step];

            (((bit * self.current_volume) as f32) / 7.5) - 1.0
        } else {
            -1.0
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.frequency_timer -= cycles as isize;
        self.envelope_cycles += cycles;
        self.length_cycles += cycles;

        let envelope_cycles_needed = CLOCK_SPEED / 64;

        if self.nr24.trigger {
            self.nr24.trigger = false;

            self.enabled = true;

            self.frequency_timer = (2048 - self.period as isize) * 4;
            self.current_timer = self.nr21.initial_timer as usize;
            self.envelope_timer = self.nr22.sweep_pace as usize;

            self.current_volume = self.nr22.initial_volume as usize;
        }

        if self.frequency_timer <= 0 {
            self.frequency_timer = (2048 - self.period as isize) * 4;

            self.duty_step = (self.duty_step + 1) & 0x7;
        }

        if self.nr24.length_enable {
            let length_cycles_needed = CLOCK_SPEED / 256;


            if self.length_cycles >= length_cycles_needed {
                self.length_cycles -= length_cycles_needed;

                self.current_timer += 1;

                if self.current_timer >= 64 {
                    self.enabled = false;
                    self.current_timer = 0;
                    self.length_cycles = 0;
                }
            }
        }

        if self.nr22.sweep_pace > 0 && self.envelope_cycles >= envelope_cycles_needed {
            self.envelope_cycles -= envelope_cycles_needed;

            self.envelope_timer -= 1;

            if self.envelope_timer == 0 {
                self.envelope_timer = self.nr22.sweep_pace as usize;
                if self.nr22.env_dir == EnvelopeDirection::Decrease {
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

    }
}