use apu::{sound_panning_register::SoundPanningRegister, APU};
use cartridge::Cartridge;
use ppu::{lcd_status_register::LCDStatusRegister, PPU};
use interrupt_register::InterruptRegister;

pub mod interrupt_register;
pub mod ppu;
pub mod cartridge;
pub mod apu;

pub struct Bus {
    pub cartridge: Cartridge,
    wram: Box<[u8]>,
    hram: Box<[u8]>,
    pub ime: bool,
    pub IF: InterruptRegister,
    pub ie: InterruptRegister,
    pub ppu: PPU,
    pub apu: APU
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
            ppu: PPU::new(),
            apu: APU::new()
        }
    }

    pub fn mem_read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7fff => self.cartridge.rom[address as usize], // TODO: implement banks
            0xff44 => self.ppu.line_y,
            _ => panic!("(mem_read8): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_read16(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x7fff => unsafe { *(&self.cartridge.rom[address as usize] as *const u8 as *const u16) }, // TODO: implement banks
            0xc000..=0xdfff => unsafe { *(&self.wram[(address - 0xc000) as usize] as *const u8 as *const u16) },
            _ => panic!("(mem_read16): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_write16(&mut self, address: u16, value: u16) {
        match address {
            0xc000..=0xdfff => unsafe { *(&mut self.wram[(address - 0xc000) as usize] as *mut u8 as *mut u16) = value },
            _ => panic!("(mem_write16): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3fff => (), // TODO: ROM bank switching
            0x4000..=0x7fff => (), // TODO: ROM bank switching
            0x8000..=0x9fff => self.ppu.vram[(address - 0x8000) as usize] = value,
            0xa000..=0xbfff => self.cartridge.write_ram(address - 0xa000, value),
            0xc000..=0xdfff => self.wram[(address - 0xc000) as usize] = value,
            0xfe00..=0xfe9f => self.ppu.write_oam(address, value),
            0xfea0..=0xfeff => (), // ignore, this area is restricted but some games may still write to it
            0xff01..=0xff02 => (), // Serial ports, ignore!
            0xff0f => self.IF = InterruptRegister::from_bits_retain(value),
            0xff10 => self.apu.nr10.write(value),
            0xff12 => self.apu.nr12.write(value),
            0xff14 => self.apu.nr14.write(value),
            0xff17 => self.apu.nr22.write(value),
            0xff19 => self.apu.nr24.write(value),
            0xff1a => self.apu.write_dac_enable(value),
            0xff21 => self.apu.nr42.write(value),
            0xff23 => self.apu.nr44.write(value),
            0xff24 => self.apu.nr50.write(value),
            0xff25 => self.apu.nr51 = SoundPanningRegister::from_bits_retain(value),
            0xff26 => self.apu.nr52.write(value),
            0xff40 => self.ppu.update_lcdc(value),
            0xff41 => self.ppu.stat = LCDStatusRegister::from_bits_truncate(value),
            0xff42 => self.ppu.scy = value,
            0xff43 => self.ppu.scx = value,
            0xff47 => self.ppu.bgp.write(value),
            0xff48 => self.ppu.obp0.write(value),
            0xff49 => self.ppu.obp1.write(value),
            0xff7f => (), // ignore this one, tetris tries to write to here for some reason.
            0xff80..=0xfffe => self.hram[(address - 0xff80) as usize] = value,
            0xffff => self.ie = InterruptRegister::from_bits_retain(value),
            _ => panic!("(mem_write8): invalid address given: 0x{:x}", address)
        }
    }
}