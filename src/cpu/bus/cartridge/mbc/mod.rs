use super::backup_file::BackupFile;

pub mod mbc1;
pub mod mbc3;
pub mod mbc5;

pub trait MBC {
    fn read(&mut self, address: u16, rom: &[u8]) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn read16(&mut self, address: u16, rom: &[u8]) -> u16;
    fn write16(&mut self, address: u16, value: u16);
    fn backup_file(&self) -> &BackupFile;
    fn save(&mut self);
    fn save_rtc(&mut self);
    fn clear_is_dirty(&mut self);
    fn save_web_mobile(&self) -> *const u8;
    fn save_rtc_web_mobile(&self) -> String;
    fn load_rtc(&mut self, json: String);
    fn load_save(&mut self, buf: &[u8]);
    fn has_timer(&self) -> bool;
}