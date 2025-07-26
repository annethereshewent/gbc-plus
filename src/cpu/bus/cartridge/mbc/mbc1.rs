use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::cpu::bus::cartridge::backup_file::BackupFile;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
enum BankingMode {
    Simple = 0,
    Advanced = 1
}

#[derive(Serialize, Deserialize)]
pub struct MBC1 {
    _ram_size: usize,
    rom_size: usize,
    has_ram: bool,
    ram_enable: bool,
    rom_bank: u8,
    ram_bank: u8,
    banking_mode: BankingMode,
    pub backup_file: BackupFile
}

impl MBC1 {
    pub fn check_save(&mut self, is_cloud: bool) -> bool {
        let hash = blake3::hash(&self.backup_file.ram).to_string();

        let min_diff = if is_cloud { 3000 } else { 1000 };
        let min_last_saved = if is_cloud { 30000 } else { 15000 };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();

        let last_saved = self.backup_file.last_saved;

        if Some(hash.clone()) != self.backup_file.previous_hash ||
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
                let actual_address = self.get_rom_address_lower(address);

                rom[actual_address]
            }
            0x4000..=0x7fff => {
                let actual_address = self.get_rom_address_upper(address);

                rom[actual_address]
            }
            0xa000..=0xbfff => if self.has_ram && self.ram_enable {
                let actual_address = self.get_ram_address(address);
                self.backup_file.read8(actual_address)
            } else {
                0xff
            }
            _ => panic!("invalid address to mbc read given: 0x{:x}", address)
        }
    }
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => self.update_ram_enable(value),
            0x2000..=0x3fff => self.update_rom_bank(value),
            0x4000..=0x5fff => self.update_ram_upper_rom_bank(value),
            0x6000..=0x7fff => self.update_banking_mode(value & 0x1),
            0xa000..=0xbfff => if self.has_ram && self.ram_enable {
                let actual_address = self.get_ram_address(address);

                self.backup_file.write8(actual_address, value);
            }
            _ => panic!("unsupported address received for mbc write: 0x{:x}", address)
        }
    }

    pub fn write16(&mut self, address: u16, value: u16) {
        match address {
            0x0000..=0x1fff => self.update_ram_enable(value as u8),
            0x2000..=0x3fff => self.update_rom_bank(value as u8),
            0x4000..=0x5fff => self.update_ram_upper_rom_bank(value as u8),
            0x6000..=0x7fff => self.update_banking_mode((value & 0x1) as u8),
            0xa000..=0xbfff => if self.has_ram && self.ram_enable {
                let actual_address = self.get_ram_address(address);

                self.backup_file.write16(actual_address, value);
            },
            _ => panic!("unsupported address received: 0x{:x}", address)
        }
    }

    pub fn read16(&mut self, address: u16, rom: &[u8]) -> u16 {
        match address {
            0x0000..=0x3fff => {
                let actual_address = self.get_rom_address_lower(address);
                unsafe { *(&rom[actual_address] as *const u8 as *const u16) }

            }
            0x4000..=0x7fff => {
                let actual_address = self.get_rom_address_upper(address);

                unsafe { *(&rom[actual_address as usize] as *const u8 as *const u16) }
            }
             0xa000..=0xbfff => if self.has_ram && self.ram_enable {
                let actual_address = self.get_ram_address(address);
                self.backup_file.read16(actual_address)
            } else {
                0xff
            }
            _ => panic!("(mbc_read16): unsupported address given: 0x{:x}", address)
        }
    }

    pub fn new(
        has_ram: bool,
        has_battery: bool,
        rom_size: usize,
        ram_size: usize,
        save_path: Option<String>,
        is_desktop: bool
    ) -> Self {
        Self {
            _ram_size: ram_size,
            rom_size: rom_size,
            has_ram,
            ram_enable: false,
            banking_mode: BankingMode::Simple,
            rom_bank: 1,
            ram_bank: 0,
            backup_file: BackupFile::new(save_path, ram_size, has_battery && has_ram, is_desktop)
        }
    }

    fn get_ram_address(&self, address: u16) -> usize {
        if self.banking_mode == BankingMode::Simple {
            (address & 0x1fff) as usize
        } else {
            (address & 0x1fff) as usize | (self.ram_bank as usize) << 13
        }
    }

    fn get_rom_address_lower(&self, address: u16) -> usize {
        if self.banking_mode == BankingMode::Simple {
            (address as usize) & 0x3fff
        } else {
            (address as usize) & 0x3fff | (((self.rom_bank as usize) >> 5) & 0x3) << 19
        }
    }

    fn get_rom_address_upper(&self, address: u16) -> usize {
        (address as usize) & 0x3fff | (self.rom_bank as usize) << 14
    }

    fn update_rom_bank(&mut self, value: u8) {
        let rom_bank = if value == 0 { 1 } else { value & 0x1f };

        self.rom_bank = rom_bank;
    }

    fn update_banking_mode(&mut self, value: u8) {
        self.banking_mode = match value {
            0 => BankingMode::Simple,
            1 => BankingMode::Advanced,
            _ => unreachable!()
        }
    }

    fn update_ram_upper_rom_bank(&mut self, value: u8) {
        if self.rom_size >= 0x100000 {
            self.rom_bank &= 0x1f;
            self.rom_bank |= (value & 0x3) << 5;
        } else if self.has_ram {
            self.ram_bank = value & 0x3;
        }
    }

    fn update_ram_enable(&mut self, value: u8) {
        if self.has_ram {
            if value == 0xa {
                self.ram_enable = true;
            } else {
                self.ram_enable = false;
            }
        }
    }


}