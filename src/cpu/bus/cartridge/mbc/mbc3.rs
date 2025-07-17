use std::{cmp, fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}};

use chrono::{DateTime, Duration, Local, TimeDelta, TimeZone};
use serde::{Deserialize, Serialize};

use crate::cpu::bus::cartridge::backup_file::BackupFile;

use super::MBC;

#[derive(Serialize, Deserialize)]
struct RtcFile {
    timestamp: usize,
    carry_bit: bool,
    halted: bool
}

impl RtcFile {
    pub fn new(
        timestamp: usize,
        halted: bool,
        carry_bit: bool,
    ) -> Self
    {
        Self {
            timestamp,
            carry_bit,
            halted
        }
    }
}

#[derive(Copy, Clone, Debug)]
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

pub struct MBC3 {
    start: DateTime<Local>,
    rom_bank: u8,
    ram_bank: u8,
    timer_ram_enable: bool,
    latch_clock: ClockRegister,
    backup_file: BackupFile,
    _rom_size: usize,
    has_ram: bool,
    has_timer: bool,
    latch_value: u8,
    clock_latched: bool,
    rtc_file: Option<File>,
    carry_bit: bool,
    previous_wrapped_days: u16,
    halted: bool,
    halted_elapsed: TimeDelta
}

impl MBC for MBC3 {
    fn backup_file(&self) -> &BackupFile {
        &self.backup_file
    }

    fn load_save(&mut self, buf: &[u8]) {
        self.backup_file.load_save(buf);
    }

    fn save_web_mobile(&self) -> *const u8 {
        self.backup_file.ram.as_ptr()
    }

    fn clear_is_dirty(&mut self) {
        self.backup_file.is_dirty = false;
    }

    fn save_rtc(&mut self) {
        let rtc_json = RtcFile::new(
            self.start.timestamp() as usize,
            self.halted,
            self.carry_bit
        );


        if let Some(file) = &mut self.rtc_file {
            match serde_json::to_string::<RtcFile>(&rtc_json) {
                Ok(result) => {
                    file.seek(SeekFrom::Start(0)).unwrap();
                    file.write_all(result.as_bytes()).unwrap();
                }
                Err(_) => ()
            }
        }
    }

    fn read(&mut self, address: u16, rom: &[u8]) -> u8 {
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
                    self.read_rtc()
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

    fn read16(&mut self, address: u16, rom: &[u8]) -> u16 {
        match address {
            0x0000..=0x3fff => {
                unsafe { *(&rom[address as usize] as *const u8 as *const u16) }

            }
            0x4000..=0x7fff => {
                let actual_address = self.get_rom_address(address);

                unsafe { *(&rom[actual_address] as *const u8 as *const u16) }
            }
            0xa000..=0xbfff => if self.timer_ram_enable {
                if self.ram_bank > 0x7 {
                    self.read_rtc() as u16
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
        self.backup_file.save_file();
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
                   self.write_rtc_latch(value);
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
                    self.write_rtc_latch(value as u8);
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

    fn write_rtc_latch(&mut self, value: u8) {
        match self.ram_bank {
            0xc => self.latch_clock.rtc_dh = value,
            0xb => self.latch_clock.rtc_dl = value,
            0xa => self.latch_clock.rtc_h = value,
            0x9 => self.latch_clock.rtc_m = value,
            0x8 => self.latch_clock.rtc_s = value,
            _ => panic!("invalid option given for rtc register")
        }

        let previous_halted = self.halted;

        self.carry_bit = ((self.latch_clock.rtc_dh >> 7) & 0x1) == 1;
        self.halted = ((self.latch_clock.rtc_dh >> 6) & 0x1) == 1;

        if !previous_halted && self.halted {
            self.halted_elapsed = Local::now().signed_duration_since(self.start);


        } else if previous_halted && !self.halted {
            self.start = Local::now() - self.halted_elapsed;
        }
    }
    fn read_rtc(&self) -> u8 {
        match self.ram_bank {
            0xc => self.latch_clock.rtc_dh,
            0xb => self.latch_clock.rtc_dl,
            0xa => self.latch_clock.rtc_h,
            0x9 => self.latch_clock.rtc_m,
            0x8 => self.latch_clock.rtc_s,
            _ => panic!("invalid option given for rtc register")
        }
    }
    fn update_rtc_latch(&mut self) {
        let now = Local::now();

        let delta = if self.halted { self.halted_elapsed } else { cmp::max(now.signed_duration_since(self.start), Duration::zero()) };

        let seconds = delta.num_seconds() % 60;
        let minutes = (delta.num_seconds() / 60) % 60;
        let hours = (delta.num_seconds() / 60 / 60) % 24;

        let days = hours / 24;

        let new_wrapped_days = days & 0x1ff;

        if new_wrapped_days < self.previous_wrapped_days as i64 {
            self.carry_bit = true;
        }
        self.previous_wrapped_days = new_wrapped_days as u16;

        self.latch_clock.rtc_dh = ((days >> 8) & 0x1) as u8 | (self.carry_bit as u8) << 7;
        self.latch_clock.rtc_dl = days as u8;
        self.latch_clock.rtc_h = hours as u8;
        self.latch_clock.rtc_m = minutes as u8;
        self.latch_clock.rtc_s = seconds as u8;
    }

    fn get_ram_address(&self, address: u16) -> usize {
        (address & 0x1fff) as usize | (self.ram_bank as usize) << 13
    }

    fn get_rom_address(&self, address: u16) -> usize {
        (address as usize) & 0x3fff | (self.rom_bank as usize) << 14
    }

    pub fn new(has_ram: bool, has_battery: bool, has_timer: bool, rom_size: usize, ram_size: usize, rom_path: Option<String>) -> Self {
        let (start, carry_bit, halted, halted_elapsed, rtc_file) = if let Some(rom_path) = &rom_path {
            let mut split_str: Vec<&str> = rom_path.split('.').collect();

            split_str.pop();

            split_str.push("rtc");

            let rtc_path = split_str.join(".");

            let mut rtc_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(rtc_path)
                .unwrap();

            let mut str = "".to_string();

            rtc_file.read_to_string( &mut str).unwrap();
            rtc_file.seek(SeekFrom::Start(0)).unwrap();

            let (start, carry_bit, halted, halted_elapsed) = match serde_json::from_str::<RtcFile>(&str) {
                Ok(result) => {
                    let start = Local.timestamp_opt(result.timestamp as i64, 0).unwrap();
                    let halted_elapsed = TimeDelta::new(0, 0).unwrap();

                    (start, result.carry_bit, result.halted, halted_elapsed)
                }
                Err(_) => (Local::now(), false, false, Duration::seconds(0))
            };

            (start, carry_bit, halted, halted_elapsed, Some(rtc_file))
        } else {
            // TODO: parse some json sent by web emulator
            (Local::now(), false, false, TimeDelta::new(0, 0).unwrap(), None)
        };

        Self {
            rom_bank: 1,
            ram_bank: 0,
            timer_ram_enable: false,
            latch_clock: ClockRegister::new(),
            backup_file: BackupFile::new(rom_path.clone(), ram_size, has_battery && has_ram),
            _rom_size: rom_size,
            has_ram,
            has_timer,
            start,
            rtc_file,
            latch_value: 0,
            clock_latched: false,
            carry_bit,
            previous_wrapped_days: 0,
            halted,
            halted_elapsed
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

        self.ram_bank = id;
    }

    fn latch_clock_value(&mut self, value: u8) {
        if self.has_timer {
            if self.latch_value == 0 && value == 1 {
                self.clock_latched = !self.clock_latched;
                if self.clock_latched {
                    self.update_rtc_latch();
                }
            }
            self.latch_value = value;
        }
    }
}