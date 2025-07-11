use std::{collections::VecDeque, sync::{Arc, Mutex}};

use apu::{sound_panning_register::SoundPanningRegister, APU};
use cartridge::Cartridge;
use joypad::Joypad;
use ppu::{lcd_status_register::LCDStatusRegister, PPU};
use interrupt_register::InterruptRegister;
use timer::Timer;

pub mod interrupt_register;
pub mod ppu;
pub mod cartridge;
pub mod apu;
pub mod timer;
pub mod joypad;

const CARTRIDGE_TYPE_ADDR: usize = 0x147;
const ROM_SIZE_ADDR: usize = 0x148;
const RAM_SIZE_ADDR: usize = 0x149;
const CGB_ADDR: usize = 0x143;

pub struct Bus {
    pub cartridge: Cartridge,
    wram: Box<[u8]>,
    hram: Box<[u8]>,
    pub ime: bool,
    pub IF: InterruptRegister,
    pub ie: InterruptRegister,
    pub ppu: PPU,
    pub apu: APU,
    pub joypad: Joypad,
    pub timer: Timer,
}

impl Bus {
    pub fn new(audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
        Self {
            cartridge: Cartridge::new(),
            wram: vec![0; 0x2000].into_boxed_slice(),
            hram: vec![0; 0x7f].into_boxed_slice(),
            IF: InterruptRegister::from_bits_retain(0),
            ie: InterruptRegister::from_bits_retain(0),
            ime: true,
            ppu: PPU::new(),
            apu: APU::new(audio_buffer),
            joypad: Joypad::new(),
            timer: Timer::new()
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.timer.tick(cycles, &mut self.IF);
        self.ppu.tick(cycles, &mut self.IF);
        self.apu.tick(cycles);
    }


    pub fn mem_read8(&mut self, address: u16) -> u8 {
        match address {
            // 0x0000..=0x3fff => self.cartridge.rom[address as usize], // TODO: implement banks
            0x0000..=0x7fff => if self.cartridge.mbc.is_some() {
                self.cartridge.mbc_read8(address)
            } else {
                self.cartridge.rom[address as usize]
            }
            0xa000..=0xbfff => self.cartridge.mbc_read8(address),
            0x8000..=0x9fff => self.ppu.vram[(address - 0x8000) as usize],
            0xc000..=0xdfff => self.wram[(address - 0xc000) as usize],
            0xff00 => self.joypad.read(),
            0xff04 => self.timer.div,
            0xff05 => self.timer.tima,
            0xff0f => self.IF.bits(),
            0xff1a => self.apu.channel3.dac_enable as u8,
            0xff24 => self.apu.nr50.read(),
            0xff25 => self.apu.nr51.bits(),
            0xff26 => self.apu.read_channel_status(),
            0xff40 => self.ppu.lcdc.bits(),
            0xff41 => self.ppu.read_stat(),
            0xff44 => self.ppu.line_y,
            0xff47 => self.ppu.bgp.read(),
            0xff48 => self.ppu.obp0.read(),
            0xff4a => self.ppu.wy,
            // 0xff4d => 0, // GBC, TODO
            0xff80..=0xfffe => self.hram[(address - 0xff80) as usize],
            0xffff => self.ie.bits(),
            _ => panic!("(mem_read8): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_read16(&self, address: u16) -> u16 {
        match address {
            0x0000..=0x7fff => if self.cartridge.mbc.is_some() {
                self.cartridge.mbc_read16(address)
            } else {
                unsafe { *(&self.cartridge.rom[address as usize] as *const u8 as *const u16) }
            }
            0xa000..=0xbfff => self.cartridge.mbc_read16(address),
            0xc000..=0xdfff => unsafe { *(&self.wram[(address - 0xc000) as usize] as *const u8 as *const u16) },
            0xff80..=0xfffe => unsafe { *(&self.hram[(address - 0xff80) as usize] as *const u8 as *const u16) },
            _ => panic!("(mem_read16): invalid address given: 0x{:x}", address)
        }
    }

    pub fn mem_write16(&mut self, address: u16, value: u16) {
        match address {
            0x8000..=0x9fff => unsafe { *(&mut self.ppu.vram[(address - 0x8000) as usize] as *mut u8 as *mut u16) = value },
            0xa000..=0xbfff | 0x0000..=0x7fff => self.cartridge.mbc_write16(address, value),
            0xc000..=0xdfff => unsafe { *(&mut self.wram[(address - 0xc000) as usize] as *mut u8 as *mut u16) = value },
            0xff7f => (),
            0xff80..=0xfffe => unsafe { *(&mut self.hram[(address - 0xff80) as usize] as *mut u8 as *mut u16) = value },
            0xffff => self.ie = InterruptRegister::from_bits_retain(value as u8),
            _ => panic!("(mem_write16): invalid address given: 0x{:x}", address)
        }
    }

    pub fn handle_dma(&mut self, value: u8) {
        let address = (value as u16) << 8;

        for i in 0..0xa0 {
            let value = self.mem_read8(address + i);
            self.mem_write8(0xfe00 + i, value);
        }

        self.tick(640);
    }

    pub fn check_header(&mut self) {
        let cartridge_type = self.cartridge.rom[CARTRIDGE_TYPE_ADDR];

        let cgb_flag = self.cartridge.rom[CGB_ADDR];

        if [0x80, 0xc0].contains(&cgb_flag) {
            // todo!("GBC mode");
        }

        let rom_size_header = self.cartridge.rom[ROM_SIZE_ADDR];

        self.cartridge.rom_size = match rom_size_header {
            0 => 0x8000,
            1 => 0x10000,
            2 => 0x20000,
            3 => 0x40000,
            4 => 0x80000,
            5 => 0x100000,
            6 => 0x200000,
            7 => 0x400000,
            8 => 0x800000,
            _ => panic!("unsupported rom size: {rom_size_header}")
        };

        let ram_size_header = self.cartridge.rom[RAM_SIZE_ADDR];

        self.cartridge.ram_size = match ram_size_header {
            0 => 0,
            2 => 0x2000,
            3 => 0x8000,
            4 => 0x20000,
            5 => 0x10000,
            _ => panic!("unsupported option received: {ram_size_header}")
        };

        match cartridge_type {
            0 => (),
            1 => self.set_mbc1(false, false),
            2 => self.set_mbc1(true, false),
            3 => self.set_mbc1(true, true),
            _ => panic!("unsupported mbc type: {cartridge_type}")
        }
    }

    fn set_mbc1(&mut self, ram: bool, battery: bool) {
        self.cartridge.set_mbc1(ram, battery);
    }

    pub fn mem_write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7fff | 0xa000..=0xbfff => self.cartridge.mbc_write8(address, value),
            0x8000..=0x9fff => self.ppu.vram[(address - 0x8000) as usize] = value,
            0xc000..=0xdfff => self.wram[(address - 0xc000) as usize] = value,
            0xfe00..=0xfe9f => self.ppu.write_oam(address, value),
            0xfea0..=0xfeff => (), // ignore, this area is restricted but some games may still write to it
            0xff00 => self.joypad.write(value),
            0xff01..=0xff02 => (), // Serial ports, ignore!
            0xff04 => self.timer.div = 0,
            0xff05 => self.timer.write_tima(value),
            0xff06 => self.timer.tma = value,
            0xff07 => self.timer.update_tac(value),
            0xff0f => self.IF = InterruptRegister::from_bits_retain(value),
            0xff10 => self.apu.channel1.write_sweep(value),
            0xff11 => self.apu.channel1.write_length_register(value),
            0xff12 => self.apu.channel1.write_volume_register(value),
            0xff13 => {
                self.apu.channel1.period &= 0x700;
                self.apu.channel1.period |= value as u16;
            }
            0xff14 => self.apu.channel1.write_period_high_control(value),
            0xff16 => self.apu.channel2.write_length_register(value),
            0xff17 => self.apu.channel2.write_volume_register(value),
            0xff18 => {
                self.apu.channel2.period &= 0x700;
                self.apu.channel2.period |= value as u16;
            }
            0xff19 => self.apu.channel2.write_period_high_control(value),
            0xff1a => self.apu.channel3.write_dac_enable(value),
            0xff1b => self.apu.channel3.length = value,
            0xff1c => self.apu.channel3.output = match (value >> 5) & 0x3 {
                0 => None,
                1 => Some(0),
                2 => Some(1),
                3 => Some(2),
                _ => unreachable!()
            },
            0xff1d => {
                self.apu.channel3.period &= 0x700;
                self.apu.channel3.period |= value as u16;
            }
            0xff1e => self.apu.channel3.write_period_high_control(value),
            0xff20 => self.apu.channel4.length = value & 0x3f,
            0xff21 => self.apu.channel4.nr42.write(value),
            0xff22 => self.apu.channel4.nr43.write(value),
            0xff23 => self.apu.channel4.write_control(value),
            0xff24 => self.apu.nr50.write(value),
            0xff25 => self.apu.nr51 = SoundPanningRegister::from_bits_retain(value),
            0xff26 => self.apu.nr52.write(value),
            0xff30..=0xff3f => self.apu.channel3.wave_ram[(address - 0xff30) as usize] = value,
            0xff40 => self.ppu.update_lcdc(value),
            0xff41 => self.ppu.stat = LCDStatusRegister::from_bits_truncate(value),
            0xff42 => self.ppu.scy = value,
            0xff43 => self.ppu.scx = value,
            0xff44 => (),
            0xff45 => self.ppu.lyc = value,
            0xff46 => self.handle_dma(value),
            0xff47 => self.ppu.bgp.write(value),
            0xff48 => self.ppu.obp0.write(value),
            0xff49 => self.ppu.obp1.write(value),
            0xff4a => self.ppu.wy = value,
            0xff4b => self.ppu.wx = value,
            0xff68..=0xff69 => (), // ignore this for now lmao
            0xff4d => (), // GBC, TODO
            0xff4f => (), // GBC, TODO
            0xff7f => (), // ignore this one, tetris tries to write to here for some reason.
            0xff80..=0xfffe => self.hram[(address - 0xff80) as usize] = value,
            0xffff => self.ie = InterruptRegister::from_bits_retain(value),
            _ => panic!("(mem_write8): invalid address given: 0x{:x}", address)
        }
    }
}