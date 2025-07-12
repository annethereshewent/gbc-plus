use super::backup_file::BackupFile;

pub mod mbc1;
pub mod mbc3;

pub trait MBC {
    fn read(&self, address: u16, rom: &[u8]) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn read16(&self, address: u16, rom: &[u8]) -> u16;
    fn write16(&mut self, address: u16, value: u16);
    fn backup_file(&self) -> &BackupFile;
    fn save(&mut self);
}