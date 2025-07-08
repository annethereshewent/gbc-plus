use crate::cpu::CLOCK_SPEED;

use super::{channel_length_duty_register::ChannelLengthDutyRegister, channel_period_high_control_register::ChannelPeriodHighControlRegister, channel_sweep_register::{ChannelSweepRegister, SweepDirection}, channel_volume_register::{ChannelVolumeRegister, EnvelopeDirection}};


const TICKS_PER_ITERATION: usize = CLOCK_SPEED / 128;

pub struct Channel1 {
    pub enabled: bool,
    pub nr12: ChannelVolumeRegister,
    pub nr14: ChannelPeriodHighControlRegister,
    pub nr10: ChannelSweepRegister,
    pub nr11: ChannelLengthDutyRegister,
    pub period: u16,
    sweep_cycles: usize,
    length_cycles: usize,
    envelope_cycles: usize,
    current_timer: usize,
    current_volume: usize,
    frequency_timer: isize,
    duty_step: usize,
    envelope_timer: usize,
    sweep_enabled: bool
}

pub const DUTY_PATTERNS: [[usize; 8]; 4] = [
    [0,0,0,0,0,0,0,1],
    [0,0,0,0,0,0,1,1],
    [0,0,0,0,1,1,1,1],
    [1,1,1,1,1,1,0,0]
];


impl Channel1 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr10: ChannelSweepRegister::new(),
            nr11: ChannelLengthDutyRegister::new(),
            nr12: ChannelVolumeRegister::new(),
            nr14: ChannelPeriodHighControlRegister::new(),
            period: 0,
            sweep_cycles: 0,
            length_cycles: 0,
            envelope_cycles: 0,
            current_timer: 0,
            current_volume: 0,
            frequency_timer: 0,
            duty_step: 0,
            envelope_timer: 0,
            sweep_enabled: false
        }
    }

    pub fn write_volume_register(&mut self, value: u8) {
        self.nr12.write(value);
    }

    pub fn write_period_high_control(&mut self, value: u8) {
        self.period &= 0xff;
        self.period |= ((value & 0x7) as u16) << 8;

        self.nr14.write(value);
    }

    pub fn write_length_register(&mut self, value: u8) {
        self.nr11.write(value);

        self.current_timer = self.nr11.initial_timer as usize;
    }

    pub fn write_sweep(&mut self, value: u8) {
        self.nr10.write(value);

        if self.nr10.pace == 0 {
            self.enabled = false;
            self.sweep_cycles = 0;
        }
    }

    pub fn generate_sample(&mut self) -> f32 {
        if self.enabled {
            let bit = DUTY_PATTERNS[self.nr11.wave_duty as usize][self.duty_step];

            (((bit * self.current_volume) as f32) / 7.5) - 1.0
        } else {
            -1.0
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.frequency_timer -= cycles as isize;

        if self.nr14.trigger {
            self.nr14.trigger = false;

            self.enabled = true;

            self.frequency_timer = (2048 - self.period as isize) * 4;
            self.current_timer = self.nr11.initial_timer as usize;
            self.envelope_timer = self.nr12.sweep_pace as usize;

            self.current_volume = self.nr12.initial_volume as usize;

            self.sweep_cycles = if self.nr10.pace == 0 { 8 } else { self.nr10.pace as usize };

            if self.nr10.pace > 0 || self.nr10.step > 0 {
                self.sweep_enabled = true;
            }
        }

        let sweep_cycles_needed = self.nr10.pace as usize * TICKS_PER_ITERATION;
        let envelope_cycles_needed = CLOCK_SPEED / 64;

        self.sweep_cycles += cycles;
        self.envelope_cycles += cycles;
        self.length_cycles += cycles;

        if self.frequency_timer <= 0 {
            self.frequency_timer = (2048 - self.period as isize) * 4;

            self.duty_step = (self.duty_step + 1) & 0x7;
        }

        if self.sweep_cycles >= sweep_cycles_needed {
            self.sweep_cycles -= sweep_cycles_needed;

            if self.sweep_enabled {
                let operand = self.period >> self.nr10.step;

                if self.nr10.direction == SweepDirection::Addition {
                    let new_period = self.period + operand;

                    if new_period < 0x7ff {
                        self.period = new_period
                    } else {
                        self.enabled = false;
                    }
                } else {
                    if self.period > 0 {
                        if operand <= self.period {
                            self.period = self.period - operand
                        } else {
                            self.period = 0;
                        }
                    }
                }
            }
        }

        if self.nr14.length_enable {
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

        if self.nr12.sweep_pace > 0 && self.envelope_cycles >= envelope_cycles_needed {
            self.envelope_cycles -= envelope_cycles_needed;

            self.envelope_timer -= 1;

            if self.envelope_timer == 0 {
                self.envelope_timer = self.nr12.sweep_pace as usize;
                if self.nr12.env_dir == EnvelopeDirection::Decrease {
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