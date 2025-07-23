use serde::{Deserialize, Serialize};


#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum LFSRWidth {
    Bit15 = 0,
    Bit7 = 1
}

const DIVIDERS: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

#[derive(Serialize, Deserialize)]
pub struct ChannelFreqRandomRegister {
    pub clock_divider: u8,
    pub clock_shift: u8,
    pub lfsr_width: LFSRWidth,
    clock_div_index: u8
}

impl ChannelFreqRandomRegister {
    pub fn new() -> Self {
        Self {
            clock_divider: 0,
            clock_shift: 0,
            lfsr_width: LFSRWidth::Bit15,
            clock_div_index: 0
        }
    }

    pub fn write(&mut self, value: u8) {
        let div = value & 0x7;
        self.clock_divider = DIVIDERS[div as usize];
        self.clock_div_index = div;
        self.lfsr_width = match (value >> 3) & 0x1 {
            0 => LFSRWidth::Bit15,
            1 => LFSRWidth::Bit7,
            _ => unreachable!()
        };

        self.clock_shift = (value >> 4) & 0xf;
    }

    pub fn read(&self) -> u8 {
        self.clock_div_index | (self.lfsr_width as u8) << 3 | self.clock_shift << 4
    }
}