use crate::cpu::bus::cartridge::backup_file::BackupFile;

use super::MBC;

pub struct MBC5 {
    rom_bank: u16,
    ram_bank: u8,
    ram_enable: bool,
    has_rumble: bool,
    ram_size: usize,
    has_ram: bool,
    backup_file: BackupFile
}

impl MBC for MBC5 {
    fn backup_file(&self) -> &BackupFile {
        &self.backup_file
    }

    fn read(&mut self, address: u16, rom: &[u8]) -> u8 {
       match address {
            0x0000..=0x3fff => {
                rom[address as usize]
            }
            0x4000..=0x7fff => {
                let actual_address = self.get_rom_address(address);

                rom[actual_address % rom.len()]
            }
            0xa000..=0xbfff => if self.ram_enable {
                if self.has_ram && self.ram_enable {
                    let actual_address = self.get_ram_address(address);
                    self.backup_file.read8(actual_address % self.ram_size)
                } else {
                    0xff
                }
            } else {
                0xff
            }
            _ => panic!("invalid address given: 0x{:x}", address)
        }
    }

    fn read16(&mut self, address: u16, rom: &[u8]) -> u16 {

        match address {
            0x0000..=0x3fff => {
                unsafe { *(&rom[address as usize] as *const u8 as *const u16) }
            }
            0x4000..=0x7fff => {
                let actual_address = self.get_rom_address(address);

                unsafe { *(&rom[actual_address % rom.len()] as *const u8 as *const u16) }
            }
            0xa000..=0xbfff => if self.has_ram && self.ram_enable {
                let actual_address = self.get_ram_address(address);
                self.backup_file.read16(actual_address)

            } else {
                0xff
            }
            _ => panic!("invalid address given: 0x{:x}", address)
        }
    }

    fn save(&mut self) {
        self.backup_file.save_file();
    }

    fn save_rtc(&mut self) {
        // do nothing
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => if value == 0xa {
                self.ram_enable = true;
            } else if value == 0 {
                self.ram_enable = false;
            }
            0x2000..=0x2fff => self.rom_bank = (self.rom_bank & 0x100) | value as u16,
            0x3000..=0x3fff => self.rom_bank = (self.rom_bank & 0xff) | (value as u16 & 0x1) << 8,
            0x4000..=0x5fff => self.ram_bank = value & 0xf,
            0xa000..=0xbfff => if self.has_ram && self.ram_enable {
                let actual_address = self.get_ram_address(address) % self.ram_size;
                self.backup_file.write8(actual_address, value);
            }
            _ => () // panic!("invalid address received: 0x{:x}", address)
        }
    }

    fn write16(&mut self, address: u16, value: u16) {
        match address {
            0x0000..=0x1fff => if value == 0xa {
                self.ram_enable = true;
            } else if value == 0 {
                self.ram_enable = false;
            }
            0x2000..=0x2fff => self.rom_bank = (self.rom_bank & 0x100) | value,
            0x3000..=0x3fff => self.rom_bank = (self.rom_bank & 0xff) | (value & 0x1) << 8,
            0x4000..=0x5fff => self.ram_bank = (value as u8) & 0xf,
            0xa000..=0xbfff => if self.has_ram && self.ram_enable {
                let actual_address = self.get_ram_address(address);
                self.backup_file.write16((actual_address % self.ram_size), value);
            }
            _ => ()
        }
    }
}

impl MBC5 {
    pub fn new(has_ram: bool, has_battery: bool, has_rumble: bool, _rom_size: usize, ram_size: usize,  rom_path: &str,) -> Self {
        Self {
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            has_ram,
            ram_size,
            has_rumble,
            backup_file: BackupFile::new(rom_path, ram_size, has_battery)
        }
    }
    fn get_ram_address(&self, address: u16) -> usize {
        (address & 0x1fff) as usize | (self.ram_bank as usize) << 13
    }

    fn get_rom_address(&self, address: u16) -> usize {
        (address as usize) & 0x3fff | (self.rom_bank as usize) << 14
    }
}