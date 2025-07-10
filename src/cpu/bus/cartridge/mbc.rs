pub trait MBC {
    fn read(&self, address: u16, rom: &[u8]) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn read16(&self, address: u16, rom: &[u8]) -> u16;
    fn write16(&mut self, address: u16, value: u16);
}