use chrono::{Datelike, Local, Timelike};

use crate::cpu::bus::cartridge::backup_file::BackupFile;

use super::MBC;

#[derive(Copy, Clone)]
pub struct ClockRegister {
    rtc_s: u8,
    rtc_m: u8,
    rtc_h: u8,
    rtc_dl: u8,
    rtc_dh: u8
}

impl ClockRegister {
    fn new() -> Self {
        Self {
            rtc_dh: 0,
            rtc_dl: 0,
            rtc_h: 0,
            rtc_m: 0,
            rtc_s: 0
        }
    }
}

enum SelectedRegister {
    Dh,
    Dl,
    H,
    M,
    S,
    None
}

pub struct MBC3 {
    rom_bank: u8,
    ram_bank: u8,
    timer_ram_enable: bool,
    latch_clock: ClockRegister,
    clock: ClockRegister,
    backup_file: BackupFile,
    _rom_size: usize,
    has_ram: bool,
    has_timer: bool,
    selected_register: SelectedRegister,
    latch_value: Option<u8>
}

impl MBC for MBC3 {
    fn backup_file(&self) -> &BackupFile {
        &self.backup_file
    }

    fn read(&self, address: u16, rom: &[u8]) -> u8 {
        match address {
            0x0000..=0x3fff => {
                rom[address as usize]
            }
            0x4000..=0x7fff => {
                let actual_address = self.get_rom_address(address);

                rom[actual_address]
            }
            0xa000..=0xbfff => if self.timer_ram_enable {
                if self.ram_bank > 0x7 {
                    let local_time = Local::now();

                    match self.selected_register {
                        SelectedRegister::Dh => 0,
                        SelectedRegister::Dl => 0,
                        SelectedRegister::H => local_time.hour() as u8 - 1,
                        SelectedRegister::M => local_time.minute() as u8 - 1,
                        SelectedRegister::S => local_time.second() as u8 - 1,
                        SelectedRegister::None => unreachable!()
                    }
                } else if self.has_ram {
                    let actual_address = self.get_ram_address(address);
                    self.backup_file.read8(actual_address)
                } else {
                    0xff
                }
            } else {
                0xff
            }
            _ => panic!("invalid address to mbc read given: 0x{:x}", address)
        }
    }

    fn read16(&self, address: u16, rom: &[u8]) -> u16 {
        match address {
            0x0000..=0x3fff => {
                unsafe { *(&rom[address as usize] as *const u8 as *const u16) }

            }
            0x4000..=0x7fff => {
                let actual_address = self.get_rom_address(address);

                unsafe { *(&rom[actual_address as usize] as *const u8 as *const u16) }
            }
            0xa000..=0xbfff => if self.timer_ram_enable {
                if self.ram_bank > 0x7 {
                    let local_time = Local::now();

                    match self.selected_register {
                        SelectedRegister::Dh => 0,
                        SelectedRegister::Dl => 0,
                        SelectedRegister::H => local_time.hour() as u16 - 1,
                        SelectedRegister::M => local_time.minute() as u16 - 1,
                        SelectedRegister::S => local_time.second() as u16 - 1,
                        SelectedRegister::None => unreachable!()
                    }
                } else if self.has_ram {
                    let actual_address = self.get_ram_address(address);
                    self.backup_file.read16(actual_address)
                } else {
                    0xff
                }
            } else {
                0xff
            }
            _ => panic!("(mbc_read16): unsupported address given: 0x{:x}", address)
        }
    }

    fn save(&mut self) {

    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1fff => if value == 0xa {
                self.timer_ram_enable = true;
            } else if value == 0x0 {
                self.timer_ram_enable = false;
            }
            0x2000..=0x3fff => self.update_bank(value),
            0x4000..=0x5fff => self.update_timer_ram_bank(value),
            0x6000..=0x7fff => self.latch_clock_value(value),
            0xa000..=0xbfff => if self.timer_ram_enable {
                if self.ram_bank > 0x7 {
                    match self.selected_register {
                        SelectedRegister::Dh => self.clock.rtc_dh = value,
                        SelectedRegister::Dl => self.clock.rtc_dl = value,
                        SelectedRegister::H => self.clock.rtc_h = value,
                        SelectedRegister::M => self.clock.rtc_m = value,
                        SelectedRegister::S => self.clock.rtc_s = value,
                        SelectedRegister::None => unreachable!()
                    }
                } else if self.has_ram {
                    let actual_address = self.get_ram_address(address);
                    self.backup_file.write8(actual_address, value);
                }
            }
            _ => panic!("unsupported address received: 0x{:x}", address)
        }
    }

    fn write16(&mut self, address: u16, value: u16) {
        match address {
            0x0000..=0x1fff => if value == 0xa {
                self.timer_ram_enable = true;
            } else if value == 0x0 {
                self.timer_ram_enable = false;
            }
            0x2000..=0x3fff => self.update_bank(value as u8),
            0x4000..=0x5fff => self.update_timer_ram_bank(value as u8),
            0x6000..=0x7fff => self.latch_clock_value(value as u8),
            0xa000..=0xbfff => if self.timer_ram_enable {
                if self.ram_bank > 0x7 {
                    match self.selected_register {
                        SelectedRegister::Dh => self.clock.rtc_dh = value as u8,
                        SelectedRegister::Dl => self.clock.rtc_dl = value as u8,
                        SelectedRegister::H => self.clock.rtc_h = value as u8,
                        SelectedRegister::M => self.clock.rtc_m = value as u8,
                        SelectedRegister::S => self.clock.rtc_s = value as u8,
                        SelectedRegister::None => unreachable!()
                    }
                } else {
                    let actual_address = self.get_ram_address(address);
                    self.backup_file.write16(actual_address, value);
                }
            }
            _ => panic!("unsupported address received: 0x{:x}", address)
        }
    }
}

impl MBC3 {
    fn get_ram_address(&self, address: u16) -> usize {
        (address & 0x1fff) as usize | (self.ram_bank as usize) << 13
    }

    fn get_rom_address(&self, address: u16) -> usize {
        (address as usize) & 0x3fff | (self.rom_bank as usize) << 14
    }

    pub fn new(has_ram: bool, has_battery: bool, has_timer: bool, rom_size: usize, ram_size: usize, rom_path: &str) -> Self {
        Self {
            rom_bank: 1,
            ram_bank: 0,
            timer_ram_enable: false,
            latch_clock: ClockRegister::new(),
            clock: ClockRegister::new(),
            backup_file: BackupFile::new(rom_path, ram_size, has_battery && has_ram),
            _rom_size: rom_size,
            has_ram,
            has_timer,
            selected_register: SelectedRegister::None,
            latch_value: None
        }
    }

    fn update_bank(&mut self, value: u8) {
        if value == 0 {
            self.rom_bank = 1;
        } else {
            self.rom_bank = value & 0x7f;
        }
    }

    fn update_timer_ram_bank(&mut self, value: u8) {
        let id = value & 0xf;

        if id < 0xa {
            self.ram_bank = value;
        } else if self.has_timer {
            self.selected_register = match value {
                0x8 => SelectedRegister::S,
                0x9 => SelectedRegister::M,
                0xa => SelectedRegister::H,
                0xb => SelectedRegister::Dl,
                0xc => SelectedRegister::Dh,
                _ => panic!("invalid option received: 0x{:x}", value)
            }
        }
    }

    fn latch_clock_value(&mut self, value: u8) {
        if self.has_timer {
            if self.latch_value.is_none() && value == 0 {
                self.latch_value = Some(0);
            }

            if let Some(latch_value) = &mut self.latch_value {
                if *latch_value == 0 && value == 1 {
                    self.latch_clock = self.clock.clone();
                }
                self.latch_value = None;
            }
        }
    }
}