use interrupt_register::InterruptRegister;



pub mod interrupt_register;

pub struct Bus {
    pub rom: Vec<u8>,
    wram: Box<[u8]>,
    pub ime: bool,
    pub IF: InterruptRegister,
    pub ie: InterruptRegister
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            wram: vec![0; 0x2000].into_boxed_slice(),
            IF: InterruptRegister::from_bits_retain(0),
            ie: InterruptRegister::from_bits_retain(0),
            ime: true
        }
    }

    pub fn mem_read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.rom[address as usize],
            _ => panic!("(mem_read8): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_read16(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x3fff => unsafe { *(&self.rom[address as usize] as *const u8 as *const u16) },
            _ => panic!("(mem_read16): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_write8(&mut self, address: u16, value: u8) {
        match address {
            0xc000..=0xdfff => self.wram[(address - 0xc000) as usize] = value,
            0xff0f => self.IF = InterruptRegister::from_bits_retain(value),
            0xffff => self.ie = InterruptRegister::from_bits_retain(value),
            _ => panic!("(mem_write8): invalid address given: 0x{:x}", address)
        }
    }
}