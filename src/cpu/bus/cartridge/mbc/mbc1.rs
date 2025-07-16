use crate::cpu::bus::cartridge::backup_file::BackupFile;

use super::MBC;

#[derive(Copy, Clone, PartialEq)]
enum BankingMode {
    Simple = 0,
    Advanced = 1
}

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

impl MBC for MBC1 {
    fn backup_file(&self) -> &BackupFile {
        &self.backup_file
    }

    // Do nothing; MBC1 has no RTC
    fn save_rtc(&mut self) {}

    fn save(&mut self) {
        self.backup_file.save_file();
    }

    fn read(&mut self, address: u16, rom: &[u8]) -> u8 {
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
    fn write(&mut self, address: u16, value: u8) {
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

    fn write16(&mut self, address: u16, value: u16) {
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

    fn read16(&mut self, address: u16, rom: &[u8]) -> u16 {
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
}

impl MBC1 {
    pub fn new(has_ram: bool, has_battery: bool, rom_size: usize, ram_size: usize, rom_path: Option<String>) -> Self {
        Self {
            _ram_size: ram_size,
            rom_size: rom_size,
            has_ram,
            ram_enable: false,
            banking_mode: BankingMode::Simple,
            rom_bank: 1,
            ram_bank: 0,
            backup_file: BackupFile::new(rom_path, ram_size, has_battery && has_ram)
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