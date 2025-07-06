pub struct Bus {
    pub rom: Vec<u8>,
    wram: Box<[u8]>
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            wram: vec![0; 0x2000].into_boxed_slice()
        }
    }

    pub fn mem_read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.rom[address as usize],
            _ => panic!("invalid address given!")
        }
    }

    pub fn mem_read16(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x3fff => unsafe { *(&self.rom[address as usize] as *const u8 as *const u16) },
            _ => panic!("invalid address given!")
        }
    }

    pub fn mem_write8(&mut self, address: u16, value: u8) {
        match address {
            0xc000..=0xdfff => self.wram[(address - 0xc000) as usize] = value,
            _ => panic!("invalid address given: 0x{:x}", address)
        }
    }
}