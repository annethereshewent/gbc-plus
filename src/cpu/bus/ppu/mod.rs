use std::{thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}};

use bg_palette_register::{BGColor, BGPaletteRegister};
use lcd_control_register::LCDControlRegister;
use lcd_status_register::LCDStatusRegister;
use oam_entry::OAMEntry;
use obj_palette_register::ObjPaletteRegister;
use picture::{Color, Picture};

use super::interrupt_register::InterruptRegister;

pub mod lcd_status_register;
pub mod lcd_control_register;
pub mod bg_palette_register;
pub mod obj_palette_register;
pub mod oam_entry;
pub mod picture;

const MODE2_CYCLES: usize = 80;
const MODE3_CYCLES: usize = 172;
const MODE0_CYCLES: usize = 204;
const MODE1_CYCLES: usize = 456;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const FPS_INTERVAL: u128 = 1000 / 60;


#[derive(Copy, Clone, PartialEq)]
pub enum OamPriority {
    None,
    Background
}

#[derive(Copy, Clone)]
pub struct OamAttributes {
    pub priority: OamPriority,
    pub y_flip: bool,
    pub x_flip: bool,
    pub dmg_palette: u8
}

impl OamAttributes {
    pub fn new(attributes: u8) -> Self {
        Self {
            priority: match (attributes >> 7) & 0x1 {
                0 => OamPriority::None,
                1 => OamPriority::Background,
                _ => unreachable!()
            },
            dmg_palette: (attributes >> 4) & 0x1,
            x_flip: (attributes >> 5) & 0x1 == 1,
            y_flip: (attributes >> 6) & 0x1 == 1,
        }
    }
}


#[derive(Copy, Clone, PartialEq)]
pub enum LCDMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    HDraw = 3
}

pub struct PPU {
    pub scy: u8,
    pub scx: u8,
    pub wx: u8,
    pub wy: u8,
    pub stat: LCDStatusRegister,
    pub lcdc: LCDControlRegister,
    pub line_y: u8,
    pub vram: Box<[u8]>,
    pub cycles: usize,
    pub bgp: BGPaletteRegister,
    mode: LCDMode,
    pub obp0: ObjPaletteRegister,
    pub obp1: ObjPaletteRegister,
    pub oam: [OAMEntry; 0xa0],
    pub frame_finished: bool,
    pub picture: Picture,
    previous_time: u128,
    prev_background_indexes: [usize; SCREEN_WIDTH]
}

impl PPU {
    pub fn new() -> Self {
        Self {
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            stat: LCDStatusRegister::from_bits_truncate(0),
            lcdc: LCDControlRegister::from_bits_retain(0),
            mode: LCDMode::OAMScan,
            line_y: 0,
            vram: vec![0; 0x2000].into_boxed_slice(),
            cycles: 0,
            bgp: BGPaletteRegister::new(),
            obp0: ObjPaletteRegister::new(),
            obp1: ObjPaletteRegister::new(),
            oam: [OAMEntry::new(); 0xa0],
            frame_finished: false,
            picture: Picture::new(),
            previous_time: 0,
            prev_background_indexes: [0; SCREEN_WIDTH]
        }
    }

    pub fn tick(&mut self, cycles: usize, interrupt_register: &mut InterruptRegister) {
        if self.lcdc.contains(LCDControlRegister::LCD_AND_PPU_ENABLE) {
            self.cycles += cycles;

            match self.mode {
                LCDMode::HBlank => self.handle_hblank(interrupt_register),
                LCDMode::VBlank => self.handle_vblank(interrupt_register),
                LCDMode::OAMScan => self.handle_oam_scan(),
                LCDMode::HDraw => self.handle_hdraw(interrupt_register),
            }
        }
    }

    fn handle_hblank(&mut self, interrupt_register: &mut InterruptRegister) {
        if self.cycles >= MODE0_CYCLES {
            self.cycles -= MODE0_CYCLES;
            self.line_y += 1;

            self.mode = if self.line_y == 144 {
                if self.stat.contains(LCDStatusRegister::MODE1) {
                    interrupt_register.set(InterruptRegister::LCD, true);
                }

                interrupt_register.set(InterruptRegister::VBLANK, true);
                LCDMode::VBlank
            } else {
                if self.stat.contains(LCDStatusRegister::MODE2) {
                    interrupt_register.set(InterruptRegister::LCD, true);
                }
                LCDMode::OAMScan
            };
        }
    }

    fn get_pixel(bg_color: BGColor) -> Color {
        match bg_color {
            BGColor::White => Color::new(0x9b, 0xbc, 0x0f),
            BGColor::LightGray => Color::new(0x8b, 0xac, 0x0f),
            BGColor::DarkGray => Color::new(0x48, 0x98, 0x48),
            BGColor::Black => Color::new(0x15, 0x56, 0x15)
        }
    }

    pub fn cap_fps(&mut self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();

        if self.previous_time != 0 {
            let diff = current_time - self.previous_time;

            if diff < FPS_INTERVAL {
                sleep(Duration::from_millis((FPS_INTERVAL - diff) as u64));
            }
        }

        self.previous_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();
    }

    fn draw_line(&mut self) {
        self.draw_background();
        self.draw_objects();
    }

    fn draw_background(&mut self) {
        let base_tilemap_address: usize = if !self.lcdc.contains(LCDControlRegister::BG_TILEMAP) {
            0x9800
        } else {
            0x9c00
        };

        let base_tile_address: usize = if !self.lcdc.contains(LCDControlRegister::BG_AND_WINDOW_TILES) {
            0x9000
        } else {
            0x8000
        };

        let scroll_x = self.scx;
        let scroll_y = self.scy;
        let y = self.line_y;

        let base_tile = ((scroll_y as usize + y as usize) / 8) * 32 + scroll_x as usize / 8;

        // println!("base_tile = {base_tile}");

        for x in 0..SCREEN_WIDTH {
            if self.lcdc.contains(LCDControlRegister::BG_WINDOW_ENABLE_PRIORITY) {
                let tile_number = base_tile + x / 8;

                let tilemap_address = base_tilemap_address as usize + tile_number;

                let tile_id = self.vram_read8(tilemap_address);

                let x_in_tile = (x as usize + scroll_x as usize) % 8;
                let y_in_tile = (y as usize + scroll_y as usize) % 8;

                let tile_address = if base_tile_address == 0x8000 {
                    base_tile_address + tile_id as usize * 16 + y_in_tile * 2
                } else {
                    let offset = tile_id as i8 as i32 * 16 + y_in_tile as i32 * 2;

                    (base_tile_address as i32 + offset) as usize
                };

                let lower_byte = self.vram_read8(tile_address);
                let upper_byte = self.vram_read8(tile_address + 1);


                let palette_index = (upper_byte >> (7 - x_in_tile) & 0x1) << 1 | lower_byte >> (7 - x_in_tile) & 0x1;

                let color = self.bgp.indexes[palette_index as usize];

                let pixel = Self::get_pixel(color);

                self.picture.set_pixel(x, y as usize, pixel);

                self.prev_background_indexes[x as usize] = color as usize;
            } else {
                let color = self.bgp.indexes[0];

                let pixel = match color {
                    BGColor::White => Color::new(0x9b, 0xbc, 0x0f),
                    BGColor::LightGray => Color::new(0x8b, 0xac, 0x0f),
                    BGColor::DarkGray => Color::new(0x48, 0x98, 0x48),
                    BGColor::Black => Color::new(0x15, 0x56, 0x15)
                };

                self.picture.set_pixel(x, y as usize, pixel);

                self.prev_background_indexes[x as usize] = 0;
            }
        }
    }

    fn vram_read8(&self, address: usize) -> u8 {
        self.vram[address - 0x8000]
    }

    fn draw_objects(&mut self) {
        if !self.lcdc.contains(LCDControlRegister::OBJ_ENABLE) {
            return;
        }

        let sprite_height = if self.lcdc.contains(LCDControlRegister::OBJ_SIZE) {
            16
        } else {
            8
        };

        let mut candidates: Vec<OAMEntry> = Vec::new();



        for entry in &mut self.oam {
             let y_diff: i16 = self.line_y as i16 - (entry.y_position as i16 - 16);
            if y_diff >= 0 && y_diff < sprite_height as i16 {
                candidates.push(*entry);

                if candidates.len() == 10 {
                    break;
                }
            }
        }

        let base_tilemap_address: u16 = 0x8000;

        for sprite in candidates {
            for i in 0..8 {
                let x_pos = i + (sprite.x_position as i16 - 8);

                println!("x_pos = {x_pos}");

                if x_pos < 0 {
                    continue;
                }

                let mut y_in_tile = self.line_y - (sprite.y_position - 16);

                if sprite.attributes.y_flip {
                    y_in_tile = sprite_height - 1 - y_in_tile;
                }

                let tile_address = base_tilemap_address + sprite.tile_index as u16 * 16 + y_in_tile as u16 * 2;

                let lower_byte = self.vram_read8(tile_address as usize);
                let upper_byte = self.vram_read8(tile_address as usize + 1);

                let bit_index = if sprite.attributes.x_flip { i } else { 7 - i };

                let lower_bit = (lower_byte >> bit_index) & 1;
                let upper_bit = (upper_byte >> bit_index) & 1;

                let palette_index = lower_bit | (upper_bit << 1);

                if palette_index == 0 {
                    continue;
                }

                let color = if sprite.attributes.dmg_palette == 0 {
                    self.obp0.indexes[palette_index as usize]
                } else {
                    self.obp1.indexes[palette_index as usize]
                };

                if sprite.attributes.priority == OamPriority::None || self.prev_background_indexes[x_pos as usize] == 0 {
                    // draw the pixel!
                    let pixel = Self::get_pixel(color);

                    self.picture.set_pixel(x_pos as usize, self.line_y as usize, pixel);
                } else {
                    continue;
                }
            }

        }
    }

    fn handle_vblank(&mut self, interrupt_register: &mut InterruptRegister) {
        if self.cycles >= MODE1_CYCLES {
            self.cycles -= MODE1_CYCLES;

            self.line_y += 1;

            if self.line_y == 154 {
                if self.stat.contains(LCDStatusRegister::MODE2) {
                    interrupt_register.set(InterruptRegister::LCD, true);
                }
                self.frame_finished = true;

                self.mode = LCDMode::OAMScan;
                self.line_y = 0;
            }
        }
    }

    fn handle_oam_scan(&mut self) {
        if self.cycles >= MODE2_CYCLES {
            self.cycles -= MODE2_CYCLES;

            self.mode = LCDMode::HDraw
        }
    }

    fn handle_hdraw(&mut self, interrupt_register: &mut InterruptRegister) {
        if self.cycles >= MODE3_CYCLES {
            self.cycles -= MODE3_CYCLES;

            if self.stat.contains(LCDStatusRegister::MODE0) {
                interrupt_register.set(InterruptRegister::LCD, true);
            }

            self.draw_line();

            self.mode = LCDMode::HBlank;
        }
    }

    pub fn update_lcdc(&mut self, value: u8) {
        let previous_enable = self.lcdc.contains(LCDControlRegister::LCD_AND_PPU_ENABLE);
        self.lcdc = LCDControlRegister::from_bits_retain(value);

        if previous_enable && !self.lcdc.contains(LCDControlRegister::LCD_AND_PPU_ENABLE) {
            self.line_y = 0;
            // set to mode 0 on disabling the ppu
            self.mode = LCDMode::HBlank;
        } else if !previous_enable && self.lcdc.contains(LCDControlRegister::LCD_AND_PPU_ENABLE) {
            // set to mode 2
            self.mode = LCDMode::OAMScan;
        }
    }

    pub fn write_oam(&mut self, address: u16, value: u8) {
        let oam_index = (address - 0xfe00) / 4;

        let offset = (address - 0xfe00) & 0x3;

        let oam = &mut self.oam[oam_index as  usize];

        match offset {
            0 => oam.y_position = value,
            1 => oam.x_position = value,
            2 => oam.tile_index = value,
            3 => oam.attributes = OamAttributes::new(value),
            _ => unreachable!()
        }
    }
}