use mbc::{mbc1::MBC1, mbc3::MBC3, mbc5::MBC5, MBC};
use serde::{Deserialize, Serialize};

pub mod backup_file;
pub mod mbc;

#[derive(Serialize, Deserialize)]
pub struct Cartridge {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub rom: Vec<u8>,
    pub rom_size: usize,
    pub ram_size: usize,
    pub mbc: MBC,
    pub rom_path: Option<String>
}

impl Cartridge {
    pub fn new(rom_path: Option<String>) -> Self {
        Self {
            rom: Vec::new(),
            rom_size: 0,
            ram_size: 0,
            mbc: MBC::None,
            rom_path
        }
    }

    pub fn set_mbc1(&mut self, ram: bool, battery: bool) {
        self.mbc = MBC::MBC1(
            MBC1::new(
                ram,
                battery,
                self.rom_size,
                self.ram_size,
                self.rom_path.clone()
            )
        );
    }

    pub fn set_mbc3(&mut self, ram: bool, battery: bool, timer: bool) {
        self.mbc = MBC::MBC3(
            MBC3::new(
                ram,
                battery,
                timer,
                self.rom_size,
                self.ram_size,
                self.rom_path.clone()
            )
        );
    }

    pub fn set_mbc5(&mut self, ram: bool, battery: bool, rumble: bool) {
        self.mbc = MBC::MBC5(
            MBC5::new(
                ram,
                battery,
                rumble,
                self.rom_size,
                self.ram_size,
                self.rom_path.clone()
            )
        );
    }

    pub fn mbc_write8(&mut self, address: u16, value: u8) {
        // if let Some(mbc) = &mut self.mbc {
        //     mbc.write(address, value)
        // }
        match &mut self.mbc {
            MBC::MBC1(mbc1) => mbc1.write(address, value),
            MBC::MBC3(mbc3) => mbc3.write(address, value),
            MBC::MBC5(mbc5) => mbc5.write(address, value),
            _ => ()
        }
    }

    pub fn mbc_read8(&mut self, address: u16) -> u8 {
        match &mut self.mbc {
            MBC::MBC1(mbc1) => mbc1.read(address, &self.rom),
            MBC::MBC3(mbc3) => mbc3.read(address, &self.rom),
            MBC::MBC5(mbc5) => mbc5.read(address, &self.rom),
            _ => 0xff
        }
    }

    pub fn mbc_read16(&mut self, address: u16) -> u16 {
        match &mut self.mbc {
            MBC::MBC1(mbc1) => mbc1.read16(address, &self.rom),
            MBC::MBC3(mbc3) => mbc3.read16(address, &self.rom),
            MBC::MBC5(mbc5) => mbc5.read16(address, &self.rom),
            _ => 0xff
        }
    }

    pub fn mbc_write16(&mut self, address: u16, value: u16) {
        // if let Some(mbc) = &mut self.mbc {
        //     mbc.write(address, value)
        // }
        match &mut self.mbc {
            MBC::MBC1(mbc1) => mbc1.write16(address, value),
            MBC::MBC3(mbc3) => mbc3.write16(address, value),
            MBC::MBC5(mbc5) => mbc5.write16(address, value),
            _ => ()
        }
    }
}