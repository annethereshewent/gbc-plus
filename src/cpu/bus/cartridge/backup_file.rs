use std::{collections::HashSet, fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, time::{SystemTime, UNIX_EPOCH}};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BackupFile {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub file: Option<File>,
    pub is_dirty: bool,
    pub ram: Box<[u8]>,
    pub last_updated: u128,
    pub is_desktop: bool,
    pub last_saved: u128,
    pub dirty_reads: HashSet<u16>,
    pub dirty_writes: HashSet<u16>
}

impl BackupFile {
    pub fn new(save_path: Option<String>, ram_size: usize, has_backup: bool, is_desktop: bool) -> Self {
         let mut ram = vec![0; ram_size];
        let file = if let Some(filename) = save_path {
            let file = if has_backup {
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(filename)
                    .unwrap();

                let file_length = file.metadata().unwrap().len();

                if file_length == ram_size as u64 {
                    file.read_exact(&mut ram).unwrap();
                    file.seek(SeekFrom::Start(0)).unwrap();
                } else {
                    file.set_len(ram_size as u64).unwrap();
                }

                Some(file)
            } else {
                None
            };

            file
        } else {
            None
        };

        Self {
            is_dirty: false,
            file,
            ram: ram.into_boxed_slice(),
            last_updated: 0,
            last_saved: 0,
            is_desktop,
            dirty_reads: HashSet::new(),
            dirty_writes: HashSet::new()
        }
    }

    pub fn clear_is_dirty(&mut self) {
        self.is_dirty = false;
    }


    pub fn write8(&mut self, address: usize, value: u8) {
        self.dirty_writes.insert(address as u16);
        self.ram[address] = value;


        if self.dirty_reads.contains(&(address as u16)) {
            if self.is_desktop {
                self.last_updated = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("an error occurred")
                    .as_millis();
            }

            self.is_dirty = true;
        }
    }

    pub fn write16(&mut self, address: usize, value: u16) {
        self.dirty_writes.insert(address as u16);
        unsafe { *(&mut self.ram[address] as *mut u8 as *mut u16) = value };

        if self.dirty_reads.contains(&(address as u16)) {
            if self.is_desktop {
                self.last_updated = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("an error occurred")
                    .as_millis();
            }

            self.is_dirty = true;
        }
    }

    pub fn read8(&mut self, address: usize) -> u8 {
        if !self.dirty_writes.contains(&(address as u16)) {
            self.dirty_reads.insert(address as u16);
        }

        self.ram[address]
    }

    pub fn read16(&mut self, address: usize) -> u16 {
         if !self.dirty_writes.contains(&(address as u16)) {
            self.dirty_reads.insert(address as u16);
        }
        unsafe { *(&self.ram[address] as *const u8 as *const u16) }
    }

    pub fn save_file(&mut self) {
        self.is_dirty = false;
        self.last_updated = 0;
        if self.is_desktop {
            self.last_saved = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("an error occurred")
                .as_millis();
        }

        if let Some(file) = &mut self.file {
            file.seek(SeekFrom::Start(0)).unwrap();
            file.write_all(&self.ram).unwrap();
        }
    }

    pub fn load_save(&mut self, buf: &[u8]) {
        self.ram = buf.to_vec().into_boxed_slice();
    }
}

impl Drop for BackupFile {
    fn drop(&mut self) {
        if self.is_dirty {
            self.save_file();
        }
    }
}