use std::{fs::{File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, time::{SystemTime, UNIX_EPOCH}};

pub struct BackupFile {
    file: Option<File>,
    pub is_dirty: bool,
    ram: Box<[u8]>,
    pub last_updated: u128
}

impl BackupFile {
    pub fn new(rom_path: Option<String>, ram_size: usize, has_backup: bool) -> Self {
         let mut ram = vec![0; ram_size];
        let file = if let Some(rom_path) = rom_path {
            let mut split_vec: Vec<&str> = rom_path.split('.').collect();

            // remove the extension
            split_vec.pop();

            let filename = format!("{}.sav", split_vec.join("."));

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
            last_updated: 0
        }
    }

    pub fn write8(&mut self, address: usize, value: u8) {
        self.ram[address] = value;
        self.is_dirty = true;

        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();
    }

    pub fn write16(&mut self, address: usize, value: u16) {
        unsafe { *(&mut self.ram[address] as *mut u8 as *mut u16) = value };
        if self.file.is_some() {
            self.is_dirty = true;
        }
    }

    pub fn read8(&self, address: usize) -> u8 {
        self.ram[address]
    }

    pub fn read16(&self, address: usize) -> u16 {
        unsafe { *(&self.ram[address] as *const u8 as *const u16) }
    }

    pub fn save_file(&mut self) {
        self.is_dirty = false;
        self.last_updated = 0;
        if let Some(file) = &mut self.file {
            file.seek(SeekFrom::Start(0)).unwrap();
            file.write_all(&self.ram).unwrap();
        }
    }
}

impl Drop for BackupFile {
    fn drop(&mut self) {
        if self.is_dirty {
            self.save_file();
        }
    }
}