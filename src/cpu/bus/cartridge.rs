use std::{cell::RefCell, rc::Rc};

use mbc::MBC;
use mbc1::MBC1;

pub mod mbc;
pub mod mbc1;

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub rom_size: usize,
    pub ram_size: usize,
    pub mbc: Option<Box<dyn MBC>>
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            rom_size: 0,
            ram_size: 0,
            mbc: None
        }
    }

    pub fn set_mbc1(&mut self, ram: bool, battery: bool) {
        self.mbc = Some(Box::new(MBC1::new(ram, battery, self.rom_size, self.ram_size)));
    }

    pub fn mbc_write8(&mut self, address: u16, value: u8) {
        if let Some(mbc) = &mut self.mbc {
            mbc.write(address, value)
        }
    }

    pub fn mbc_read8(&mut self, address: u16) -> u8 {
        if let Some(mbc) = &self.mbc {
            mbc.read(address, &self.rom)
        } else {
            0xff
        }
    }

    pub fn mbc_read16(&self, address: u16) -> u16 {
        if let Some(mbc) = &self.mbc {
            mbc.read16(address, &self.rom)
        } else {
            0xff
        }
    }

    pub fn mbc_write16(&mut self, address: u16, value: u16) {
        if let Some(mbc) = &mut self.mbc {
            mbc.write16(address, value)
        }
    }
}