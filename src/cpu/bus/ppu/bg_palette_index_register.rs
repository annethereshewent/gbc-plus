pub struct BgPaletteIndexRegister {
    pub address: u8,
    pub auto_increment: bool
}

impl BgPaletteIndexRegister {
    pub fn new() -> Self {
        Self {
            address: 0,
            auto_increment: false
        }
    }
    pub fn write(&mut self, value: u8) {
        self.address = value & 0x3f;
        self.auto_increment = (value >> 7) & 0x1 == 1;

    }
}