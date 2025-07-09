
#[derive(Copy, Clone, PartialEq)]
pub enum LFSRWidth {
    Bit15,
    Bit7
}

const DIVIDERS: [u8; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub struct ChannelFreqRandomRegister {
    pub clock_divider: u8,
    pub clock_shift: u8,
    pub lfsr_width: LFSRWidth
}

impl ChannelFreqRandomRegister {
    pub fn new() -> Self {
        Self {
            clock_divider: 0,
            clock_shift: 0,
            lfsr_width: LFSRWidth::Bit15
        }
    }

    pub fn write(&mut self, value: u8) {
        let div = value & 0x7;
        self.clock_divider = DIVIDERS[div as usize];
        self.lfsr_width = match (value >> 3) & 0x1 {
            0 => LFSRWidth::Bit15,
            1 => LFSRWidth::Bit7,
            _ => unreachable!()
        };

        self.clock_shift = (value >> 4) & 0xf;
    }
}