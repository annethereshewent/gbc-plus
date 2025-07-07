pub enum LSFRWidth {
    Bit15,
    Bit7
}

pub struct ChannelFreqRandomRegister {
    pub clock_divider: u8,
    pub clock_shift: u8,
    pub lsfr_width: LSFRWidth
}

impl ChannelFreqRandomRegister {
    pub fn new() -> Self {
        Self {
            clock_divider: 0,
            clock_shift: 0,
            lsfr_width: LSFRWidth::Bit15
        }
    }

    pub fn write(&mut self, value: u8) {
        self.clock_divider = value & 0x7;
        self.lsfr_width = match (value >> 3) & 0x1 {
            0 => LSFRWidth::Bit15,
            1 => LSFRWidth::Bit7,
            _ => unreachable!()
        };

        self.clock_shift = (value >> 4) & 0xf;
    }
}