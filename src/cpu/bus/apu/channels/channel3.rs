use serde::{Deserialize, Serialize};

use super::channel_period_high_control_register::ChannelPeriodHighControlRegister;


#[derive(Serialize, Deserialize)]
pub struct Channel3 {
    pub enabled: bool,
    pub nr34: ChannelPeriodHighControlRegister,
    pub period: u16,
    pub length: u8,
    pub output: Option<usize>,
    pub dac_enable: bool,
    pub wave_ram: [u8; 16],
    current_timer: usize,
    frequency_timer: isize,
    current_sample: u8,
    sample_counter: usize
}

impl Channel3 {
    pub fn new() -> Self {
        Self {
            enabled: false,
            nr34: ChannelPeriodHighControlRegister::new(),
            period: 0,
            length: 0,
            output: None,
            dac_enable: false,
            wave_ram: [0; 16],
            current_timer: 0,
            frequency_timer: 0,
            current_sample: 0,
            sample_counter: 0
        }
    }

    pub fn write_length(&mut self, value: u8) {
        self.length = value;

        self.current_timer = self.length as usize;
    }

    pub fn write_period_high_control(&mut self, value: u8) {
        self.period &= 0xff;
        self.period |= ((value as u16) & 0x7) << 8;

        self.nr34.write(value);
    }

    pub fn write_dac_enable(&mut self, value: u8) {
        let previous_enable = self.dac_enable;
        self.dac_enable = (value >> 7) & 0x1 == 1;

        if previous_enable && !self.dac_enable {
            self.enabled = false;
        }
    }

    pub fn generate_sample(&self) -> f32 {
        if self.enabled && self.dac_enable  {
            if let Some(output) = self.output {
                 return (self.current_sample >> output) as f32
            }
        }

        0.0
    }

    fn restart_channel(&mut self) {
        self.nr34.trigger = false;

        self.enabled = self.dac_enable;

        if self.current_timer >= 256 {
            self.current_timer = 0;
        }

        // could also be calculated as CLOCK_SPEED / sample_frequency where sample_frequency = 2097152 / (2048 - period)
        self.frequency_timer = (2048 - self.period as isize) * 2;

        self.sample_counter = 0;
    }

    pub fn tick_length(&mut self) {
        if self.nr34.length_enable {
            self.current_timer += 1;

            if self.current_timer >= 256 {
                self.enabled = false;

            }
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        if self.nr34.trigger && self.dac_enable {
            self.restart_channel();
        }

        self.frequency_timer -= cycles as isize;

        if self.frequency_timer <= 0 {
            self.frequency_timer = (2048 - self.period as isize) * 2;

            let shift = if (self.sample_counter & 1) == 0 { 0 } else { 4 };

            self.current_sample = (self.wave_ram[(self.sample_counter / 2) as usize] >> shift) & 0xf;

            self.sample_counter = (self.sample_counter + 1) & 0x1f;
        }
    }
}