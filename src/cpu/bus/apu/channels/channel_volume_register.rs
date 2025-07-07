pub enum EnvelopeDirection {
    Decrease,
    Increase
}

pub struct ChannelVolumeRegister {
    pub sweep_space: u8,
    pub env_dir: EnvelopeDirection,
    pub initial_volume: u8
}

impl ChannelVolumeRegister {
    pub fn new() -> Self {
        Self {
            sweep_space: 0,
            env_dir: EnvelopeDirection::Decrease,
            initial_volume: 0
        }
    }

    pub fn write(&mut self, value: u8) {
        self.sweep_space = value & 0x7;
        self.env_dir = match (value >> 3) & 0b1 {
            0 => EnvelopeDirection::Decrease,
            1 => EnvelopeDirection::Increase,
            _ => unreachable!()
        };

        self.initial_volume = (value >> 4) & 0xf;
    }
}