pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Box<[u8]>,
    pub rom_size: usize,
    pub ram_size: usize
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            ram: vec![0; 0x2000].into_boxed_slice(),
            rom_size: 0,
            ram_size: 0
        }
    }

    pub fn write_ram(&mut self, address: u16, val: u8) {
        // TODO: save RAM to disk
        self.ram[address as usize] = val;
    }
}