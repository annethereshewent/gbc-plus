use std::{cmp, fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, time::{SystemTime, UNIX_EPOCH}};

use chrono::{DateTime, Duration, Local, TimeDelta, TimeZone};
use serde::{Deserialize, Serialize};

use crate::cpu::bus::cartridge::backup_file::BackupFile;

#[derive(Serialize, Deserialize)]
pub struct RtcFile {
    timestamp: usize,
    carry_bit: bool,
    halted: bool,
    num_wraps: usize
}

impl RtcFile {
    pub fn new(
        timestamp: usize,
        halted: bool,
        carry_bit: bool,
        num_wraps: usize
    ) -> Self
    {
        Self {
            timestamp,
            carry_bit,
            halted,
            num_wraps
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct MBC3 {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub start: DateTime<Local>,
    rom_bank: u8,
    ram_bank: u8,
    timer_ram_enable: bool,
    latch_clock: ClockRegister,
    pub backup_file: BackupFile,
    _rom_size: usize,
    has_ram: bool,
    pub has_timer: bool,
    latch_value: u8,
    clock_latched: bool,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub rtc_file: Option<File>,
    pub carry_bit: bool,
    previous_wrapped_days: u16,
    pub halted: bool,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    halted_elapsed: TimeDelta,
    pub num_wraps: usize,
    pub is_dirty: bool
}

impl MBC3 {
    pub fn check_save(&mut self, is_cloud: bool) -> bool {
        let min_diff = if is_cloud { 1500 } else { 500 };
        // let min_last_saved = if is_cloud { 20000 } else { 10000 };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();

        // let last_saved = self.backup_file.last_saved;

        let last_updated = self.backup_file.last_updated;

        if self.backup_file.is_dirty &&
            current_time > last_updated &&
            last_updated != 0 // ||
            // (last_saved == 0 || (current_time - last_saved) >= min_last_saved)
        {
            let diff = current_time - last_updated;
            if diff >= min_diff {
                self.backup_file.last_updated = 0;
                return true;
            }
        }

        false
    }

    pub fn has_saved(&mut self) -> bool {
        let return_val = self.backup_file.is_dirty;

        self.backup_file.is_dirty = false;

        return_val
    }

    pub fn save_rtc_web_mobile(&self) -> String {
        if self.has_timer {
            let rtc_json = RtcFile::new(
                self.start.timestamp() as usize,
                self.halted,
                self.carry_bit,
                self.num_wraps
            );
            serde_json::to_string::<RtcFile>(&rtc_json).unwrap_or("".to_string())
        } else {
            "".to_string()
        }
    }

    pub fn load_rtc(&mut self, json: String) {
        if self.has_timer {
            match serde_json::from_str::<RtcFile>(&json) {
                Ok(result) => {
                    let start = Local.timestamp_opt(result.timestamp as i64, 0).unwrap();
                    let halted_elapsed = TimeDelta::new(0, 0).unwrap();

                    self.carry_bit = result.carry_bit;
                    self.halted = result.halted;
                    self.start = start;
                    self.halted_elapsed = halted_elapsed;
                }
                Err(_) => ()
            }
        }
    }


    pub fn save_rtc(&mut self) {
        let rtc_json = RtcFile::new(
            self.start.timestamp() as usize,
            self.halted,
            self.carry_bit,
            self.num_wraps
        );


        if let Some(file) = &mut self.rtc_file {
            match serde_json::to_string::<RtcFile>(&rtc_json) {
                Ok(result) => {
                    file.set_len(0).unwrap();
                    file.seek(SeekFrom::Start(0)).unwrap();
                    file.write_all(result.as_bytes()).unwrap();
                }
                Err(_) => ()
            }
        }
    }

    pub fn read(&mut self, address: u16, rom: &[u8]) -> u8 {
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

    pub fn read16(&mut self, address: u16, rom: &[u8]) -> u16 {
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

    pub fn write(&mut self, address: u16, value: u8) {
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

    pub fn write16(&mut self, address: u16, value: u16) {
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

        let previous_carry = self.carry_bit;

        self.carry_bit = ((self.latch_clock.rtc_dh >> 7) & 0x1) == 1;

        if previous_carry != self.carry_bit {
            self.is_dirty = true;
        }

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

        let delta = cmp::max(now.signed_duration_since(self.start), Duration::zero());

        let seconds = delta.num_seconds() % 60;
        let minutes = (delta.num_seconds() / 60) % 60;
        let hours = (delta.num_seconds() / 60 / 60) % 24;

        let days = delta.num_seconds() / 60 / 60 / 24;

        let num_wraps = days / 0x1ff;

        if num_wraps as usize > self.num_wraps {
            self.carry_bit = true;
            self.num_wraps = num_wraps as usize;
            self.is_dirty = true;
        }

        let new_wrapped_days = days & 0x1ff;

        self.latch_clock.rtc_dh = ((new_wrapped_days >> 8) & 0x1) as u8 | (self.carry_bit as u8) << 7;
        self.latch_clock.rtc_dl = new_wrapped_days as u8;
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

    pub fn new(
        has_ram: bool,
        has_battery: bool,
        has_timer: bool,
        rom_size: usize,
        ram_size: usize,
        save_path: Option<String>,
        is_desktop: bool,
        logged_in: bool
    ) -> Self {
        let (start, carry_bit, halted, halted_elapsed, rtc_file, num_wraps) = if let Some(save_path) = &save_path {
            if has_timer && !logged_in {
                let mut split_str: Vec<&str> = save_path.split('.').collect();

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

                let (start, carry_bit, halted, halted_elapsed, num_wraps) = match serde_json::from_str::<RtcFile>(&str) {
                    Ok(result) => {
                        let start = Local.timestamp_opt(result.timestamp as i64, 0).unwrap();
                        let halted_elapsed = TimeDelta::new(0, 0).unwrap();

                        (start, result.carry_bit, result.halted, halted_elapsed, result.num_wraps)
                    }
                    Err(_) => (Local::now(), false, false, Duration::seconds(0), 0)
                };

                (start, carry_bit, halted, halted_elapsed, Some(rtc_file), num_wraps)
            } else {
                (Local::now(), false, false, TimeDelta::new(0, 0).unwrap(), None, 0)
            }
        } else {
            (Local::now(), false, false, TimeDelta::new(0, 0).unwrap(), None, 0)
        };

        Self {
            rom_bank: 1,
            ram_bank: 0,
            timer_ram_enable: false,
            latch_clock: ClockRegister::new(),
            backup_file: BackupFile::new(save_path.clone(), ram_size, has_battery && has_ram, is_desktop),
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
            halted_elapsed,
            num_wraps,
            is_dirty: false
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