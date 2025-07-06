pub struct Bus {
    pub rom: Vec<u8>
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Vec::new()
        }
    }

    pub fn mem_read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.rom[address as usize],
            _ => panic!("invalid address given!")
        }
    }
}