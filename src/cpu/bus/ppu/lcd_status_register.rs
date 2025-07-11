use bitflags::bitflags;

bitflags! {
    pub struct LCDStatusRegister: u8 {
        const MODE0 = 1 << 3;
        const MODE1 = 1 << 4;
        const MODE2 = 1 << 5;
        const LYC_INT = 1 << 6;
    }
}

impl LCDStatusRegister {
    pub fn read(&self) -> u8 {
        self.bits()
    }
}