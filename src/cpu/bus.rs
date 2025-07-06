use cartridge::Cartridge;
use ppu::{lcd_control_register::LCDControlRegister, lcd_status_register::LCDStatusRegister, PPU};
use interrupt_register::InterruptRegister;

pub mod interrupt_register;
pub mod ppu;
pub mod cartridge;

pub struct Bus {
    pub cartridge: Cartridge,
    wram: Box<[u8]>,
    hram: Box<[u8]>,
    pub ime: bool,
    pub IF: InterruptRegister,
    pub ie: InterruptRegister,
    pub ppu: PPU
}

impl Bus {
    pub fn new() -> Self {
        Self {
            cartridge: Cartridge::new(),
            wram: vec![0; 0x2000].into_boxed_slice(),
            hram: vec![0; 0x7f].into_boxed_slice(),
            IF: InterruptRegister::from_bits_retain(0),
            ie: InterruptRegister::from_bits_retain(0),
            ime: true,
            ppu: PPU::new()
        }
    }

    pub fn mem_read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3fff => self.cartridge.rom[address as usize],
            0xff44 => self.ppu.line_y,
            _ => panic!("(mem_read8): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_read16(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x3fff => unsafe { *(&self.cartridge.rom[address as usize] as *const u8 as *const u16) },
            _ => panic!("(mem_read16): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3fff => (), // TODO: ROM bank switching
            0x4000..=0x7fff => (), // TODO: ROM bank switching
            0x8000..=0x9fff => self.ppu.vram[(address - 0x8000) as usize] = value,
            0xa000..=0xbfff => self.cartridge.write_ram(address - 0xa000, value),
            0xc000..=0xdfff => self.wram[(address - 0xc000) as usize] = value,
            0xff01..=0xff02 => (), // Serial ports, ignore!
            0xff0f => self.IF = InterruptRegister::from_bits_retain(value),
            0xff40 => self.ppu.update_lcdc(value),
            0xff41 => self.ppu.stat = LCDStatusRegister::from_bits_truncate(value),
            0xff42 => self.ppu.scy = value,
            0xff43 => self.ppu.scx = value,
            0xff47 => self.ppu.bgp.write(value),
            0xff80..=0xfffe => self.hram[(address - 0xff80) as usize] = value,
            0xffff => self.ie = InterruptRegister::from_bits_retain(value),
            _ => panic!("(mem_write8): invalid address given: 0x{:x}", address)
        }
    }
}