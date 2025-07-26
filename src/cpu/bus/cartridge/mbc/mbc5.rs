use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::cpu::bus::cartridge::backup_file::BackupFile;

#[derive(Serialize, Deserialize)]
pub struct MBC5 {
    rom_bank: u16,
    ram_bank: u8,
    ram_enable: bool,
    _has_rumble: bool,
    ram_size: usize,
    has_ram: bool,
    pub backup_file: BackupFile
}

impl MBC5 {
    pub fn check_save(&mut self, is_cloud: bool) -> bool {
        let hash = blake3::hash(&self.backup_file.ram).to_string();

        let min_diff = if is_cloud { 3000 } else { 1000 };
        let min_last_saved = if is_cloud { 6000 } else { 3000 };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();

        let last_saved = self.backup_file.last_saved;

        println!("current_time - last_saved = {}, last_saved = {}", current_time - last_saved, last_saved);

        if Some(hash.clone()) != self.backup_file.previous_hash &&
            (last_saved == 0 || (current_time - last_saved) >= min_last_saved)
        {
            let last_updated = self.backup_file.last_updated;

            self.backup_file.previous_hash = Some(hash);

            if self.backup_file.is_dirty &&
                current_time > last_updated &&
                last_updated != 0
            {
                let diff = current_time - last_updated;
                if diff >= min_diff {
                    self.backup_file.last_updated = 0;
                    return true;
                }
            }
        }

        false
    }

     pub fn has_saved(&mut self) -> bool {
        let return_val = self.backup_file.is_dirty;

        self.backup_file.is_dirty = false;

        return_val
    }

    pub fn read(&mut self, address: u16, rom: &[u8]) -> u8 {
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

    pub fn read16(&mut self, address: u16, rom: &[u8]) -> u16 {

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

    pub fn write(&mut self, address: u16, value: u8) {
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

    pub fn write16(&mut self, address: u16, value: u16) {
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
                self.backup_file.write16(actual_address % self.ram_size, value);
            }
            _ => ()
        }
    }

    pub fn new(
        has_ram: bool,
        has_battery: bool,
        _has_rumble: bool,
        _rom_size: usize,
        ram_size: usize,
        save_path: Option<String>,
        is_desktop: bool
    ) -> Self {
        Self {
            rom_bank: 0,
            ram_bank: 0,
            ram_enable: false,
            has_ram,
            ram_size,
            _has_rumble,
            backup_file: BackupFile::new(save_path, ram_size, has_battery, is_desktop)
        }
    }
    fn get_ram_address(&self, address: u16) -> usize {
        (address & 0x1fff) as usize | (self.ram_bank as usize) << 13
    }

    fn get_rom_address(&self, address: u16) -> usize {
        (address as usize) & 0x3fff | (self.rom_bank as usize) << 14
    }
}