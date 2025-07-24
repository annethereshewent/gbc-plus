use std::sync::Arc;

use apu::{sound_panning_register::SoundPanningRegister, APU};
use cartridge::{mbc::MBC, Cartridge};
use joypad::Joypad;
use ppu::PPU;
use interrupt_register::InterruptRegister;
use ringbuf::{storage::Heap, wrap::caching::Caching, SharedRb};
use serde::{Deserialize, Serialize};
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

#[derive(Copy, Clone, PartialEq)]
pub enum HdmaMode {
    General,
    Hblank
}

#[derive(Serialize, Deserialize)]
pub struct Bus {
    pub cartridge: Cartridge,
    wram: [Box<[u8]>; 8],
    hram: Box<[u8]>,
    pub ime: bool,
    pub IF: InterruptRegister,
    pub ie: InterruptRegister,
    pub ppu: PPU,
    pub apu: APU,
    pub joypad: Joypad,
    pub timer: Timer,
    wram_bank: usize,
    pub double_speed: bool,
    pub vram_dma_source: u16,
    pub vram_dma_destination: u16,
    hdma_hblank: bool,
    hdma_length: isize,
    curr_dma_source: u16,
    curr_dma_dest: u16,
    hdma_finished: bool,
    pub debug_on: bool
}

impl Bus {
    pub fn new(
        producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
        waveform_producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
        rom_path: Option<String>,
        is_ios: bool
    ) -> Self {
        Self {
            cartridge: Cartridge::new(rom_path),
            wram: [
                vec![0; 0x1000].into_boxed_slice(),
                vec![0; 0x1000].into_boxed_slice(),
                vec![0; 0x1000].into_boxed_slice(),
                vec![0; 0x1000].into_boxed_slice(),
                vec![0; 0x1000].into_boxed_slice(),
                vec![0; 0x1000].into_boxed_slice(),
                vec![0; 0x1000].into_boxed_slice(),
                vec![0; 0x1000].into_boxed_slice()
            ],
            hram: vec![0; 0x7f].into_boxed_slice(),
            IF: InterruptRegister::from_bits_retain(0),
            ie: InterruptRegister::from_bits_retain(0),
            ime: true,
            ppu: PPU::new(),
            apu: APU::new(producer, waveform_producer, is_ios),
            joypad: Joypad::new(),
            timer: Timer::new(),
            wram_bank: 1,
            double_speed: false,
            vram_dma_destination: 0,
            vram_dma_source: 0,
            hdma_hblank: false,
            hdma_length: 0,
            curr_dma_source: 0,
            curr_dma_dest: 0,
            hdma_finished: false,
            debug_on: false
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        let actual_cycles = if self.double_speed { cycles / 2 } else { cycles };

        if self.hdma_hblank {
            self.ppu.hdma_init = true;
        }

        let hdma_cycles = if self.hdma_hblank && self.ppu.entering_hblank(actual_cycles) {
            self.do_hdma_hblank()
        } else {
            0
        };

        self.timer.tick(cycles + hdma_cycles, &mut self.IF);
        self.ppu.tick(actual_cycles + hdma_cycles, &mut self.IF);
        self.apu.tick(actual_cycles + hdma_cycles);
    }

    fn do_hdma_hblank(&mut self) -> usize {
        let actual_destination = self.curr_dma_dest | 0x8000;

        for i in 0..0x10 {
            let value = self.mem_read8(self.curr_dma_source + i as u16);
            self.mem_write8(actual_destination + i as u16, value);
        }

        self.curr_dma_source += 0x10;
        self.curr_dma_dest += 0x10;

        self.hdma_length -= 0x10;

        self.ppu.hdma_init = false;

        if self.hdma_length == 0 {
            self.hdma_length = 0;
            self.hdma_hblank = false;
            self.hdma_finished = true;
        }
        32
    }

    pub fn mem_read8(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x7fff => match self.cartridge.mbc {
                MBC::None => self.cartridge.rom[address as usize],
                _ => self.cartridge.mbc_read8(address)
            }
            0xa000..=0xbfff => self.cartridge.mbc_read8(address),
            0x8000..=0x9fff => if self.ppu.cgb_mode {
                 if self.ppu.vram_enabled {
                    self.ppu.vram[self.ppu.vram_bank as usize][(address - 0x8000) as usize]
                } else {
                    0xff
                }
            } else {
                if self.ppu.vram_enabled {
                    self.ppu.vram[0][(address - 0x8000) as usize]
                } else {
                    0xff
                }
            },
            0xc000..=0xcfff => self.wram[0][(address - 0xc000) as usize],
            0xd000..=0xdfff => if self.ppu.cgb_mode {
                self.wram[self.wram_bank][(address - 0xd000) as usize]
            } else {
                self.wram[1][(address - 0xd000) as usize]
            }
            // echo ram, for some reason zelda oracle of seasons tries to access it.
            // TODO: properly emulate? probably not worth it haha
            0xe000..=0xfdff => 0xff,
            0xff00 => self.joypad.read(),
            0xff01..=0xff02 => 0, // serial ports, can safely ignore (hopefully!)
            0xff04 => self.timer.div,
            0xff05 => self.timer.tima,
            0xff06 => self.timer.tma,
            0xff07 => self.timer.tac.bits(),
            0xff0f => self.IF.bits(),
            0xff10 => self.apu.channel1.nrx0.as_ref().unwrap().read(),
            0xff11 => self.apu.channel1.read_length(),
            0xff12 => self.apu.channel1.read_volume(),
            0xff13 =>  0xff, // write only
            0xff14 => self.apu.channel1.nrx4.read(),
            0xff15 => 0xff, // i have no idea what this does, but Pokemon Gold seems to use it despite it not being an official register.
            0xff16 => self.apu.channel2.read_length(),
            0xff17 => self.apu.channel2.read_volume(),
            0xff18 => 0xff, // write only
            0xff19 => self.apu.channel2.nrx4.read(),
            0xff1a => 0x7f | (self.apu.channel3.dac_enable as u8) << 7,
            0xff1b => 0xff, // write only
            0xff1c => {
                if let Some(output) = self.apu.channel3.output {
                    let value = match output {
                        0 => 1,
                        1 => 2,
                        2 => 3,
                        _ => unreachable!()
                    };

                    // value << 5

                    1 << 7 | value << 5 | 0x1f
                } else {
                    1 << 7 | 0x1f
                }
            }
            0xff1d => 0xff, // write only register
            0xff1e => self.apu.channel3.nr34.read(),
            0xff1f => 0xff, // see above comment
            0xff20 => 0xff, // write only register
            0xff21 => self.apu.channel4.nr42.read(),
            0xff22 => self.apu.channel4.nr43.read(),
            0xff23 => self.apu.channel4.nr44.read(),
            0xff24 => self.apu.nr50.read(),
            0xff25 => self.apu.nr51.bits(),
            0xff26 => self.apu.read_channel_status(),
            0xff27..=0xff2f => 0xff,
            0xff30..=0xff3f => self.apu.channel3.wave_ram[address as usize - 0xff30],
            0xff40 => self.ppu.lcdc.bits(),
            0xff41 => self.ppu.read_stat(),
            0xff42 => self.ppu.scy,
            0xff43 => self.ppu.scx,
            0xff44 => self.ppu.line_y,
            0xff45 => self.ppu.lyc,
            0xff47 => self.ppu.bgp.read(),
            0xff48 => self.ppu.obp0.read(),
            0xff49 => self.ppu.obp1.read(),
            0xff4a => self.ppu.wy,
            0xff4b => self.ppu.wx,
            0xff4d => (self.double_speed as u8) << 7,
            0xff4f => self.ppu.vram_bank as u8,
            0xff55 => if self.hdma_length == 0 && self.hdma_finished { 0xff } else { ((self.hdma_length - 1) / 0x10) as u8 },
            0xff68 => self.ppu.bgpi.read(),
            0xff69 => self.ppu.bgpd_byte,
            0xff6a => self.ppu.obpi.read(),
            0xff6b => self.ppu.obpd_byte,
            0xff70 => self.wram_bank as u8,
            0xff80..=0xfffe => self.hram[(address - 0xff80) as usize],
            0xffff => self.ie.bits(),
            _ => {
                println!("[WARN](mem_read8): invalid address given: 0x{:x}", address);
                0xff
            }
        }
    }

    pub fn mem_read16(&mut self, address: u16) -> u16 {
        match address {
            0x0000..=0x7fff => match self.cartridge.mbc {
                MBC::None => unsafe { *(&self.cartridge.rom[address as usize] as *const u8 as *const u16) },
                _ => self.cartridge.mbc_read16(address)
            }
            0x8000..=0x9fff => if self.ppu.cgb_mode {
                if self.ppu.vram_enabled { unsafe { *(&self.ppu.vram[self.ppu.vram_bank as usize][(address - 0x8000) as usize] as *const u8 as *const u16) } } else { 0xff }
            } else {
                if self.ppu.vram_enabled { unsafe { *(&self.ppu.vram[0][(address - 0x8000) as usize] as *const u8 as *const u16) } } else { 0xff }
            },
            0xa000..=0xbfff => self.cartridge.mbc_read16(address),
            0xc000..=0xcfff => unsafe { *(&self.wram[0][(address - 0xc000) as usize] as *const u8 as *const u16) },
            0xd000..=0xdfff => if self.ppu.cgb_mode {
                unsafe { *(&self.wram[self.wram_bank][(address - 0xd000) as usize] as *const u8 as *const u16) }
            } else {
                unsafe { *(&self.wram[1][(address - 0xd000) as usize] as *const u8 as *const u16) }
            }
            0xff80..=0xfffe => unsafe { *(&self.hram[(address - 0xff80) as usize] as *const u8 as *const u16) },
            _ => {
                println!("(mem_read16): invalid address given: 0x{:x}", address);
                0xff
            }
        }
    }

    pub fn mem_write16(&mut self, address: u16, value: u16) {
        match address {
            0x8000..=0x9fff => if self.ppu.cgb_mode {
                if self.ppu.vram_enabled {
                    unsafe { *(&mut self.ppu.vram[self.ppu.vram_bank as usize][(address - 0x8000) as usize] as *mut u8 as *mut u16) = value }
                }
            } else {
                if self.ppu.vram_enabled {
                    unsafe { *(&mut self.ppu.vram[0][(address - 0x8000) as usize] as *mut u8 as *mut u16) = value }
                }
            }
            0xa000..=0xbfff | 0x0000..=0x7fff => self.cartridge.mbc_write16(address, value),
            0xc000..=0xcfff => unsafe { *(&mut self.wram[0][(address - 0xc000) as usize] as *mut u8 as *mut u16) = value },
            0xd000..=0xdfff => if self.ppu.cgb_mode {
                unsafe { *(&mut self.wram[self.wram_bank][(address - 0xd000) as usize] as *mut u8 as *mut u16) = value }
            } else {
                unsafe { *(&mut self.wram[1][(address - 0xd000) as usize] as *mut u8 as *mut u16) = value }
            }
            0xff7f => (),
            0xff80..=0xfffe => unsafe { *(&mut self.hram[(address - 0xff80) as usize] as *mut u8 as *mut u16) = value },
            0xffff => self.ie = InterruptRegister::from_bits_retain(value as u8),
            _ => println!("[WARN] (mem_write16): invalid address given: 0x{:x}", address)
        }
    }

    pub fn handle_dma(&mut self, value: u8) {
        let address = (value as u16) << 8;

        for i in 0..0xa0 {
            let value = self.mem_read8(address + i);
            self.mem_write8(0xfe00 + i, value);
        }

        self.tick( 640);
    }

    pub fn check_header(&mut self) {
        let cartridge_type = self.cartridge.rom[CARTRIDGE_TYPE_ADDR];

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
            0x00 => (),
            0x01 => self.cartridge.set_mbc1(false, false),
            0x02 => self.cartridge.set_mbc1(true, false),
            0x03 => self.cartridge.set_mbc1(true, true),
            0x0f => self.cartridge.set_mbc3(false, true, true),
            0x10 => self.cartridge.set_mbc3(true, true, true),
            0x11 => self.cartridge.set_mbc3(false, false, false),
            0x12 => self.cartridge.set_mbc3(true, false, false),
            0x13 => self.cartridge.set_mbc3(true, true, false),
            0x19 => self.cartridge.set_mbc5(false, false, false),
            0x1a => self.cartridge.set_mbc5(true, false, false),
            0x1b => self.cartridge.set_mbc5(true, true, false),
            0x1c => self.cartridge.set_mbc5(false, false, true),
            0x1d => self.cartridge.set_mbc5(true, false, true),
            0x1e => self.cartridge.set_mbc5(true, true, true),
            _ => panic!("unsupported mbc type: 0x{:x}", cartridge_type)
        }
    }

    fn start_hdma(&mut self, value: u8) {
        let length = (((value as u16) & 0x7f) + 1) * 0x10;

        let mode = match (value >> 7) & 0x1 {
            0 => HdmaMode::General,
            1 => HdmaMode::Hblank,
            _ => unreachable!()
        };

        self.vram_dma_destination &= !(0xf);
        self.vram_dma_source &= !(0xf);
        self.vram_dma_destination &= 0x1fff;

        if mode == HdmaMode::General {
            self.do_hdma_general(length);
        } else {
            self.restart_hdma_hblank(length);
        }
    }

    fn do_hdma_general(&mut self, length: u16) {
        self.hdma_hblank = false;
        // do transfer immediately
        let actual_address = self.vram_dma_destination | 0x8000;

        let cycles = (length / 0x10) * 32;

        for i in 0..length {
            let value = self.mem_read8(self.vram_dma_source + i);

            self.mem_write8(actual_address + i, value);
        }

        self.tick(if self.double_speed { cycles * 2 } else { cycles } as usize);
    }

    fn restart_hdma_hblank(&mut self, length: u16) {
        self.hdma_finished = false;
        self.hdma_hblank = true;
        self.ppu.hdma_init = true;
        self.hdma_length = length as isize;

        self.curr_dma_source = self.vram_dma_source;
        self.curr_dma_dest = self.vram_dma_destination;
    }

    pub fn mem_write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7fff | 0xa000..=0xbfff => self.cartridge.mbc_write8(address, value),
            0x8000..=0x9fff => if self.ppu.cgb_mode {
                if self.ppu.vram_enabled {
                    self.ppu.vram[self.ppu.vram_bank as usize][(address - 0x8000) as usize] = value;
                }
            } else {
                if self.ppu.vram_enabled {
                    self.ppu.vram[0][(address - 0x8000) as usize] = value;
                }
            },
            0xc000..=0xcfff => self.wram[0][(address - 0xc000) as usize] = value,
            0xd000..=0xdfff => if self.ppu.cgb_mode {
                self.wram[self.wram_bank][(address - 0xd000) as usize] = value
            } else {
                self.wram[1][(address - 0xd000) as usize] = value
            }
            0xfe00..=0xfe9f => self.ppu.write_oam(address, value),
            0xfea0..=0xfeff => (), // ignore, this area is restricted but some games may still write to it
            0xff00 => self.joypad.write(value),
            0xff01..=0xff02 => (), // Serial ports, ignore!
            0xff04 => self.timer.div = 0,
            0xff05 => self.timer.write_tima(value),
            0xff06 => self.timer.tma = value,
            0xff07 => self.timer.update_tac(value),
            0xff0f => self.IF = InterruptRegister::from_bits_retain(value),
            0xff10 => if self.apu.nr52.audio_on { self.apu.channel1.write_sweep(value) },
            0xff11 => if self.apu.nr52.audio_on { self.apu.channel1.write_length_register(value) },
            0xff12 => if self.apu.nr52.audio_on { self.apu.channel1.write_volume_register(value) },
            0xff13 => {
                if self.apu.nr52.audio_on {
                    self.apu.channel1.period &= 0x700;
                    self.apu.channel1.period |= value as u16;
                }
            }
            0xff14 => if self.apu.nr52.audio_on { self.apu.channel1.write_period_high_control(value) },
            0xff15 => (),
            0xff16 => if self.apu.nr52.audio_on { self.apu.channel2.write_length_register(value) },
            0xff17 => if self.apu.nr52.audio_on { self.apu.channel2.write_volume_register(value) },
            0xff18 => {
                if self.apu.nr52.audio_on {
                    self.apu.channel2.period &= 0x700;
                    self.apu.channel2.period |= value as u16;
                }
            }
            0xff19 => if self.apu.nr52.audio_on { self.apu.channel2.write_period_high_control(value) },
            0xff1a => if self.apu.nr52.audio_on { self.apu.channel3.write_dac_enable(value) },
            0xff1b => if self.apu.nr52.audio_on { self.apu.channel3.write_length(value) },
            0xff1c => if self.apu.nr52.audio_on {
                self.apu.channel3.output = match (value >> 5) & 0x3 {
                    0 => None,
                    1 => Some(0),
                    2 => Some(1),
                    3 => Some(2),
                    _ => unreachable!()
                }
            }
            0xff1d => {
                if self.apu.nr52.audio_on {
                    self.apu.channel3.period &= 0x700;
                    self.apu.channel3.period |= value as u16;
                }
            }
            0xff1e => if self.apu.nr52.audio_on { self.apu.channel3.write_period_high_control(value) },
            0xff1f => (), // used by pokemon gold but doesn't seem to do or be anything.
            0xff20 => if self.apu.nr52.audio_on { self.apu.channel4.write_length(value) },
            0xff21 => if self.apu.nr52.audio_on { self.apu.channel4.nr42.write(value) },
            0xff22 => if self.apu.nr52.audio_on { self.apu.channel4.nr43.write(value) },
            0xff23 => if self.apu.nr52.audio_on { self.apu.channel4.write_control(value) },
            0xff24 => if self.apu.nr52.audio_on { self.apu.nr50.write(value) },
            0xff25 => if self.apu.nr52.audio_on { self.apu.nr51 = SoundPanningRegister::from_bits_truncate(value) },
            0xff26 => self.apu.write_audio_master(value),
            0xff27..=0xff2f => (),
            0xff30..=0xff3f => self.apu.channel3.wave_ram[(address - 0xff30) as usize] = value,
            0xff40 => self.ppu.update_lcdc(value),
            0xff41 => self.ppu.update_stat(value, &mut self.IF),
            0xff42 => self.ppu.scy = value,
            0xff43 => self.ppu.scx = value,
            0xff44 => (),
            0xff45 => self.ppu.update_lyc(value, &mut self.IF),
            0xff46 => self.handle_dma(value),
            0xff47 => self.ppu.bgp.write(value),
            0xff48 => self.ppu.obp0.write(value),
            0xff49 => self.ppu.obp1.write(value),
            0xff4a => self.ppu.wy = value,
            0xff4b => self.ppu.wx = value,
            0xff4d => if value & 0x1 == 1 {
                self.double_speed = !self.double_speed
            }
            0xff4f => self.ppu.set_vram_bank(value & 0x1),
            0xff51 => self.vram_dma_source = (self.vram_dma_source & 0xff) | (value as u16) << 8,
            0xff52 => self.vram_dma_source = (self.vram_dma_source & 0xff00) | value as u16,
            0xff53 => self.vram_dma_destination = (self.vram_dma_destination & 0xff) | (value as u16) << 8,
            0xff54 => self.vram_dma_destination = (self.vram_dma_destination & 0xff00) | value as u16,
            0xff55 => self.start_hdma(value),
            0xff56 => (), // Infrared comms port for GBC, TODO
            0xff68 => self.ppu.bgpi.write(value),
            0xff69 => self.ppu.update_bg_palette_color(value),
            0xff6a => self.ppu.obpi.write(value),
            0xff6b => self.ppu.update_obj_palette_color(value),
            0xff70 => self.wram_bank = if value == 0 { 1 } else { (value & 0x7) as usize },
            0xff7f => (), // ignore this one, tetris tries to write to here for some reason.
            0xff80..=0xfffe => self.hram[(address - 0xff80) as usize] = value,
            0xffff => self.ie = InterruptRegister::from_bits_retain(value),
            _ => println!("[WARN](mem_write8): invalid address given: 0x{:x}", address)
        }
    }
}