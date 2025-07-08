use crate::cpu::CLOCK_SPEED;

use super::{channel_length_duty_register::ChannelLengthDutyRegister, channel_period_high_control_register::ChannelPeriodHighControlRegister, channel_sweep_register::{ChannelSweepRegister, SweepDirection}, channel_volume_register::{ChannelVolumeRegister, EnvelopeDirection}};


const TICKS_PER_ITERATION: usize = CLOCK_SPEED / 128;
pub const LENGTH_CYCLES_NEEDED: usize = CLOCK_SPEED / 256;

pub struct PulseChannel<const IS_CHANNEL1: bool>  {
    pub enabled: bool,
    pub nrx2: ChannelVolumeRegister,
    pub nrx4: ChannelPeriodHighControlRegister,
    pub nrx0: Option<ChannelSweepRegister>,
    pub nrx1: ChannelLengthDutyRegister,
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

const DUTY_PATTERNS: [[usize; 8]; 4] = [
    [0,0,0,0,0,0,0,1],
    [0,0,0,0,0,0,1,1],
    [0,0,0,0,1,1,1,1],
    [1,1,1,1,1,1,0,0]
];


impl<const IS_CHANNEL1: bool> PulseChannel<IS_CHANNEL1> {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nrx0: if IS_CHANNEL1 { Some(ChannelSweepRegister::new()) } else { None },
            nrx1: ChannelLengthDutyRegister::new(),
            nrx2: ChannelVolumeRegister::new(),
            nrx4: ChannelPeriodHighControlRegister::new(),
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
        self.nrx2.write(value);
    }

    pub fn write_period_high_control(&mut self, value: u8) {
        self.period &= 0xff;
        self.period |= ((value & 0x7) as u16) << 8;

        self.nrx4.write(value);
    }

    pub fn write_length_register(&mut self, value: u8) {
        self.nrx1.write(value);

        self.current_timer = self.nrx1.initial_timer as usize;
    }

    pub fn write_sweep(&mut self, value: u8) {
        if let Some(nrx0) = &mut self.nrx0 {
            nrx0.write(value);

            if nrx0.pace == 0 {
                self.enabled = false;
                self.sweep_cycles = 0;
            }
        }
    }

    pub fn generate_sample(&mut self) -> f32 {
        if self.enabled {
            let bit = DUTY_PATTERNS[self.nrx1.wave_duty as usize][self.duty_step];

            (((bit * self.current_volume) as f32) / 15.0) - 1.0
        } else {
            0.0
        }
    }

    fn tick_envelope(&mut self, cycles: usize) {
        let envelope_cycles_needed = CLOCK_SPEED / 64;

        self.envelope_cycles += cycles;

        if self.nrx2.sweep_pace > 0 && self.envelope_cycles >= envelope_cycles_needed {
            self.envelope_cycles -= envelope_cycles_needed;

            self.envelope_timer -= 1;

            if self.envelope_timer == 0 {
                self.envelope_timer = self.nrx2.sweep_pace as usize;
                if self.nrx2.env_dir == EnvelopeDirection::Decrease {
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

    fn tick_length(&mut self, cycles: usize) {
        self.length_cycles += cycles;

        if self.nrx4.length_enable {
            if self.length_cycles >= LENGTH_CYCLES_NEEDED {
                self.length_cycles -= LENGTH_CYCLES_NEEDED;

                self.current_timer += 1;

                if self.current_timer >= 64 {
                    self.enabled = false;
                    self.current_timer = 0;
                    self.length_cycles = 0;
                }
            }
        }
    }

    fn tick_sweep(&mut self, cycles: usize) {
        if let Some(nrx0) = &mut self.nrx0 {
            let sweep_cycles_needed = nrx0.pace as usize * TICKS_PER_ITERATION;
            self.sweep_cycles += cycles;

            if self.sweep_cycles >= sweep_cycles_needed {
                self.sweep_cycles -= sweep_cycles_needed;

                if self.sweep_enabled {
                    let operand = self.period >> nrx0.step;

                    if nrx0.direction == SweepDirection::Addition {
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
        }
    }

    fn restart_channel(&mut self) {
        self.nrx4.trigger = false;

        self.enabled = true;

        self.frequency_timer = (2048 - self.period as isize) * 4;
        self.current_timer = self.nrx1.initial_timer as usize;
        self.envelope_timer = self.nrx2.sweep_pace as usize;

        self.current_volume = self.nrx2.initial_volume as usize;

        if let Some(nrx0) = &mut self.nrx0 {
            self.sweep_cycles = if nrx0.pace == 0 { 8 } else { nrx0.pace as usize };
            if nrx0.pace > 0 || nrx0.step > 0 {
                self.sweep_enabled = true;
            }
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.frequency_timer -= cycles as isize;

        if self.nrx4.trigger {
            self.restart_channel();
        }

        self.tick_envelope(cycles);
        self.tick_length(cycles);
        if self.nrx0.is_some() {
            self.tick_sweep(cycles);
        }

        if self.frequency_timer <= 0 {
            self.frequency_timer = (2048 - self.period as isize) * 4;

            self.duty_step = (self.duty_step + 1) & 0x7;
        }
    }
}