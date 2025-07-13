use std::{thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}};

use bg_palette_register::{BGColor, BGPaletteRegister};
use bg_palette_index_register::BgPaletteIndexRegister;
use lcd_control_register::LCDControlRegister;
use lcd_status_register::LCDStatusRegister;
use oam_entry::OAMEntry;
use obj_palette_index_register::ObjPaletteIndexRegister;
use obj_palette_register::ObjPaletteRegister;
use picture::{Color, Picture};

use super::interrupt_register::InterruptRegister;

pub mod lcd_status_register;
pub mod lcd_control_register;
pub mod bg_palette_register;
pub mod obj_palette_register;
pub mod oam_entry;
pub mod picture;
pub mod bg_palette_index_register;
pub mod obj_palette_index_register;

const MODE2_CYCLES: usize = 80;
const MODE3_CYCLES: usize = 172;
const MODE0_CYCLES: usize = 204;
const MODE1_CYCLES: usize = 456;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const FPS_INTERVAL: u128 = 1000 / 60;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum OamPriority {
    None,
    Background
}

#[derive(Copy, Clone, Debug)]
pub struct OamAttributes {
    pub priority: OamPriority,
    pub y_flip: bool,
    pub x_flip: bool,
    pub dmg_palette: u8,
    pub gbc_palette: u8,
    pub bank: u8
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
            gbc_palette: attributes & 0x7,
            bank: (attributes >> 3) & 0x1
        }
    }
}

pub const CLASSIC_GREEN: [Color; 4] = [
    Color { r: 0x9b, g: 0xbc, b: 0x0f },
    Color { r: 0x8b, g: 0xac, b: 0x0f },
    Color { r: 0x48, g: 0x98, b: 0x48 },
    Color { r: 0x15, g: 0x56, b: 0x15 }
];

pub const GRAYSCALE: [Color; 4] = [
    Color { r: 0xff, g: 0xff, b: 0xff },
    Color { r: 0xaa, g: 0xaa, b: 0xaa },
    Color { r: 0x55, g: 0x55, b: 0x55 },
    Color { r: 0x00, g: 0x00, b: 0x00 }
];

pub const SOLARIZED: [Color; 4] = [
    Color { r: 0xee, g: 0xe8, b: 0xd5 },
    Color { r: 0x83, g: 0x94, b: 0x96 },
    Color { r: 0x58, g: 0x6e, b: 0x75 },
    Color { r: 0x00, g: 0x2b, b: 0x36 }
];

pub const MAVERICK: [Color; 4] = [
    Color { r: 0xe0, g: 0xf8, b: 0xcf },
    Color { r: 0x86, g: 0xc0, b: 0x6c },
    Color { r: 0x30, g: 0x68, b: 0x50 },
    Color { r: 0x07, g: 0x18, b: 0x21 }
];

pub const OCEANIC: [Color; 4] = [
    Color { r: 0xe0, g: 0xff, b: 0xff },
    Color { r: 0x7f, g: 0xdb, b: 0xff },
    Color { r: 0x00, g: 0x74, b: 0xd9 },
    Color { r: 0x00, g: 0x1f, b: 0x3f }
];

pub const BURNT_PEACH: [Color; 4] = [
    Color { r: 0xff, g: 0xee, b: 0xd8 },
    Color { r: 0xd9, g: 0x72, b: 0x5e },
    Color { r: 0x80, g: 0x3d, b: 0x26 },
    Color { r: 0x2f, g: 0x0e, b: 0x00 }
];

pub const GRAPE_SODA: [Color; 4] = [
    Color { r: 0xdc, g: 0xc6, b: 0xf8 },
    Color { r: 0x8e, g: 0x7c, b: 0xc3 },
    Color { r: 0x5c, g: 0x25, b: 0x8d },
    Color { r: 0x1f, g: 0x00, b: 0x37 }
];

pub const STRAWBERRY_MILK: [Color; 4] = [
    Color { r: 0xff, g: 0xf1, b: 0xf5 },
    Color { r: 0xff, g: 0xc2, b: 0xd7 },
    Color { r: 0xe1, g: 0x75, b: 0xa4 },
    Color { r: 0x8c, g: 0x1c, b: 0x3b }
];

pub const WITCHING_HOUR: [Color; 4] = [
    Color { r: 0xe6, g: 0xe6, b: 0xfa },
    Color { r: 0x94, g: 0x7e, b: 0xc3 },
    Color { r: 0x4b, g: 0x00, b: 0x82 },
    Color { r: 0x1a, g: 0x00, b: 0x2e }
];

pub const VOID_DREAM: [Color; 4] = [
    Color { r: 0xe0, g: 0xf7, b: 0xfa },
    Color { r: 0x81, g: 0xd4, b: 0xfa },
    Color { r: 0x4f, g: 0x86, b: 0xf7 },
    Color { r: 0x0d, g: 0x47, b: 0xa1 }
];


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
    pub vram: [Box<[u8]>; 2],
    pub cycles: usize,
    pub bgp: BGPaletteRegister,
    mode: LCDMode,
    pub obp0: ObjPaletteRegister,
    pub obp1: ObjPaletteRegister,
    pub oam: [OAMEntry; 0xa0],
    pub frame_finished: bool,
    pub picture: Picture,
    previous_time: u128,
    prev_background_indexes: [usize; SCREEN_WIDTH],
    prev_window_indexes: [usize; SCREEN_WIDTH],
    current_window_line: isize,
    previous_objs: [Option<OAMEntry>; SCREEN_WIDTH],
    pub lyc: u8,
    pub palette_colors: [[Color; 4]; 10],
    pub current_palette: usize,
    pub vram_bank: u8,
    pub bgpi: BgPaletteIndexRegister,
    pub palette_ram: [u8; 64],
    pub obj_palette_ram: [u8; 64],
    pub obpi: ObjPaletteIndexRegister,
    pub cgb_mode: bool
}

impl PPU {
    pub fn new() -> Self {
        Self {
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            stat: LCDStatusRegister::from_bits_truncate(0),
            lcdc: LCDControlRegister::from_bits_retain(0x91),
            mode: LCDMode::OAMScan,
            line_y: 0,
            vram: [vec![0; 0x2000].into_boxed_slice(), vec![0; 0x2000].into_boxed_slice()],
            cycles: 0,
            bgp: BGPaletteRegister::new(),
            obp0: ObjPaletteRegister::new(),
            obp1: ObjPaletteRegister::new(),
            oam: [OAMEntry::new(); 0xa0],
            frame_finished: false,
            picture: Picture::new(),
            previous_time: 0,
            prev_background_indexes: [0; SCREEN_WIDTH],
            prev_window_indexes: [0; SCREEN_WIDTH],
            current_window_line: -1,
            previous_objs: [None; SCREEN_WIDTH],
            lyc: 0,
            palette_colors: [
                CLASSIC_GREEN,
                GRAYSCALE,
                SOLARIZED,
                MAVERICK,
                OCEANIC,
                BURNT_PEACH,
                GRAPE_SODA,
                STRAWBERRY_MILK,
                WITCHING_HOUR,
                VOID_DREAM
            ],
            current_palette: 1,
            vram_bank: 0,
            bgpi: BgPaletteIndexRegister::new(),
            obpi: ObjPaletteIndexRegister::new(),
            palette_ram: [0; 64],
            obj_palette_ram: [0; 64],
            cgb_mode: false
        }
    }

    pub fn tick(&mut self, cycles: usize, interrupt_register: &mut InterruptRegister) {
        if self.lcdc.contains(LCDControlRegister::LCD_AND_PPU_ENABLE) {
            self.cycles += cycles;

            match self.mode {
                LCDMode::HBlank => self.handle_hblank(interrupt_register),
                LCDMode::VBlank => self.handle_vblank(interrupt_register),
                LCDMode::OAMScan => self.handle_oam_scan(interrupt_register),
                LCDMode::HDraw => self.handle_hdraw(),
            }
        }
    }

    pub fn update_bg_palette_color(&mut self, value: u8) {
        self.palette_ram[self.bgpi.address as usize] = value;
        if self.bgpi.auto_increment {
            self.bgpi.address += 1;
        }
    }

    pub fn update_obj_palette_color(&mut self, value: u8) {
        self.obj_palette_ram[self.obpi.address as usize] = value;
        if self.obpi.auto_increment {
            self.obpi.address += 1;
        }
    }

    pub fn set_vram_bank(&mut self, value: u8) {
        self.vram_bank = value & 0x1;
    }

    pub fn update_lyc(&mut self, value: u8, interrupt_register: &mut InterruptRegister) {
        self.lyc = value;

        if self.stat.contains(LCDStatusRegister::LYC_INT) && self.line_y == self.lyc {
            interrupt_register.set(InterruptRegister::LCD, true);
        }
    }

    fn handle_hblank(&mut self, interrupt_register: &mut InterruptRegister) {
        if self.cycles >= MODE0_CYCLES {
            self.cycles -= MODE0_CYCLES;

            self.line_y += 1;

             if self.stat.contains(LCDStatusRegister::LYC_INT) && self.line_y == self.lyc {
                interrupt_register.set(InterruptRegister::LCD, true);
            }

            if self.stat.contains(LCDStatusRegister::MODE0) {
                interrupt_register.set(InterruptRegister::LCD, true);
            }

            self.mode = if self.line_y == 144 {
                interrupt_register.set(InterruptRegister::VBLANK, true);
                LCDMode::VBlank
            } else {
                LCDMode::OAMScan
            };
        }
    }

    pub fn set_dmg_palette(&mut self, palette_id: usize) {
        self.current_palette = palette_id;
    }

    fn get_pixel(&self, bg_color: BGColor) -> Color {
        self.palette_colors[self.current_palette][bg_color as usize]
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
        if self.cgb_mode {
            self.draw_gbc_background();
            // self.draw_window();
            self.draw_gbc_objects();
        } else {
            self.draw_background();
            self.draw_window();
            self.draw_objects();
        }
    }

    fn draw_window(&mut self) {
        if self.line_y < self.wy ||
            !self.lcdc.contains(LCDControlRegister::WINDOW_ENABLE) ||
            !self.lcdc.contains(LCDControlRegister::BG_WINDOW_ENABLE_PRIORITY) ||
            (self.wx as isize - 7) >= SCREEN_WIDTH as isize ||
            self.current_window_line as usize >= SCREEN_HEIGHT
        {
            return;
        }

        let base_tilemap_address = if !self.lcdc.contains(LCDControlRegister::WINDOW_TILEMAP) {
            0x9800
        } else {
            0x9c00
        };

        let base_tile_address = if !self.lcdc.contains(LCDControlRegister::BG_AND_WINDOW_TILES) {
            0x9000
        } else {
            0x8000
        };

        let mut x_pos = self.wx as i16 - 7;

        if x_pos < 0 {
            x_pos = 0;
        }

        for index in self.prev_window_indexes.iter_mut() { *index = 0; }

        for x in ((x_pos as usize)..SCREEN_WIDTH).step_by(8) {
            let x_pos = (x - (self.wx as usize - 7)) & 0xff;

            let tile_number = (x_pos as usize / 8) + (self.current_window_line as usize / 8) * 32;

            let tilemap_address = base_tilemap_address + tile_number;
            let tile_index = self.vram_read8(tilemap_address, 0);

            let y_pos_in_tile = (self.current_window_line & 0x7) as usize;

            let tile_address = if base_tile_address == 0x9000 {
                (base_tile_address as i32 + (tile_index as i8 as i32) * 16) as usize + y_pos_in_tile * 2
            } else {
                base_tile_address as usize + tile_index as usize * 16 + y_pos_in_tile * 2
            };

            let lower = self.vram_read8(tile_address, 0);
            let upper = self.vram_read8(tile_address + 1, 0);

            for i in 0..8 {
                if x + i >= SCREEN_WIDTH {
                    break;
                }
                let shift = 7 - i;

                let lower_bit = (lower >> shift) & 0x1;
                let upper_bit = (upper >> shift) & 0x1;

                let palette_index = lower_bit | (upper_bit << 1);

                self.prev_background_indexes[x + i] = palette_index as usize;

                let color = self.bgp.indexes[palette_index as usize];

                let pixel = self.get_pixel(color);

                self.picture.set_pixel(x + i, self.line_y as usize, pixel);
            }
        }

        self.current_window_line += 1;
    }

    pub fn read_stat(&self) -> u8 {
        let lcd_status = if !self.lcdc.contains(LCDControlRegister::LCD_AND_PPU_ENABLE) { 0 } else { self.mode as u8 };
        self.stat.read() | lcd_status | ((self.line_y == self.lyc) as u8) << 2
    }

    fn draw_gbc_background(&mut self) {
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

        let scrolled_y = (scroll_y as usize + y as usize) & 0xff;
        for x in 0..SCREEN_WIDTH {
            if self.lcdc.contains(LCDControlRegister::BG_WINDOW_ENABLE_PRIORITY) {
                let scrolled_x = (scroll_x as usize + x) & 0xff;
                let tile_number = (scrolled_y / 8) * 32 + scrolled_x / 8;

                let tilemap_address = base_tilemap_address as usize + tile_number;

                let tile_id = self.vram_read8(tilemap_address, 0);
                let attributes = self.vram_read8(tilemap_address, 1);

                let color_palette = attributes & 0x7;
                let bank = (attributes >> 3) & 0x1;
                let x_flip = (attributes >> 5) & 0x1 == 1;
                let y_flip = (attributes >> 6) & 0x1 == 1;
                let priority = (attributes >> 7) & 0x1 == 1;

                let base_bgp_index = color_palette * 4 * 2;

                let x_in_tile = (x as usize + scroll_x as usize) % 8;
                let mut y_in_tile = (y as usize + scroll_y as usize) % 8;

                if y_flip {
                    y_in_tile = 7 - y_in_tile;
                }

                let tile_address = if base_tile_address == 0x8000 {
                    base_tile_address + tile_id as usize * 16 + y_in_tile * 2
                } else {
                    let offset = ((tile_id as i8 as i32) * 16) + ((y_in_tile as i32) * 2);

                    (base_tile_address as i32 + offset) as usize
                };

                let lower_byte = self.vram_read8(tile_address, bank as usize);
                let upper_byte = self.vram_read8(tile_address + 1, bank as usize);

                let shift = if x_flip { x_in_tile } else { 7 - x_in_tile };

                let palette_index = (upper_byte >> shift & 0x1) << 1 | lower_byte >> shift & 0x1;

                let palette_address = (base_bgp_index + palette_index * 2) as usize;

                let color = self.bg_pal_read16(palette_address);

                let pixel = Self::convert_pixel(color);

                self.picture.set_pixel(x, y as usize, pixel);

                self.prev_background_indexes[x as usize] = color as usize;
            } else {
                let color = self.bg_pal_read16(0);

                let pixel = Self::convert_pixel(color);

                self.picture.set_pixel(x, y as usize, pixel);

                self.prev_background_indexes[x as usize] = 0;
            }
        }
    }

    fn convert_pixel(color: u16) -> Color {
        let mut r = color & 0x1f;
        let mut g = (color >> 5) & 0x1f;
        let mut b = (color >> 10) & 0x1f;

        r = r << 3 | r >> 2;
        g = g << 3 | g >> 2;
        b = b << 3 | b >> 2;

        Color::new(r as u8, g as u8, b as u8)
    }

    fn bg_pal_read16(&self, address: usize) -> u16 {
        unsafe { *(&self.palette_ram[address] as *const u8 as *const u16 )}
    }

    fn obj_pal_read16(&self, address: usize) -> u16 {
        unsafe { *(&self.obj_palette_ram[address] as *const u8 as *const u16 )}
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

        let scrolled_y = (scroll_y as usize + y as usize) & 0xff;
        for x in 0..SCREEN_WIDTH {
            if self.lcdc.contains(LCDControlRegister::BG_WINDOW_ENABLE_PRIORITY) {
                let scrolled_x = (scroll_x as usize + x) & 0xff;
                let tile_number = (scrolled_y / 8) * 32 + scrolled_x / 8;

                let tilemap_address = base_tilemap_address as usize + tile_number;

                let tile_id = self.vram_read8(tilemap_address, 0);

                let x_in_tile = (x as usize + scroll_x as usize) % 8;
                let y_in_tile = (y as usize + scroll_y as usize) % 8;

                let tile_address = if base_tile_address == 0x8000 {
                    base_tile_address + tile_id as usize * 16 + y_in_tile * 2
                } else {
                    let offset = ((tile_id as i8 as i32) * 16) + ((y_in_tile as i32) * 2);

                    (base_tile_address as i32 + offset) as usize
                };

                let lower_byte = self.vram_read8(tile_address, 0);
                let upper_byte = self.vram_read8(tile_address + 1, 0);

                let palette_index = (upper_byte >> (7 - x_in_tile) & 0x1) << 1 | lower_byte >> (7 - x_in_tile) & 0x1;

                let color = self.bgp.indexes[palette_index as usize];

                let pixel = self.get_pixel(color);

                self.picture.set_pixel(x, y as usize, pixel);

                self.prev_background_indexes[x as usize] = color as usize;
            } else {
                let color = self.bgp.indexes[0];

                let pixel = self.get_pixel(color);

                self.picture.set_pixel(x, y as usize, pixel);

                self.prev_background_indexes[x as usize] = 0;
            }
        }
    }

    fn vram_read8(&self, address: usize, bank: usize) -> u8 {
        self.vram[bank][address - 0x8000]
    }

    fn draw_objects(&mut self) {
        if !self.lcdc.contains(LCDControlRegister::OBJ_ENABLE) {
            return;
        }

        let is8by16 = self.lcdc.contains(LCDControlRegister::OBJ_SIZE);

        let sprite_height = if is8by16 {
            16
        } else {
            8
        };

        let mut candidates: Vec<OAMEntry> = Vec::new();



        for i in 0..self.oam.len() {
            let entry = self.oam[i];
             let y_diff: i16 = self.line_y as i16 - (entry.y_position as i16 - 16);
            if y_diff >= 0 && y_diff < sprite_height as i16 {
                let mut entry = entry.clone();

                entry.address = i;

                candidates.push(entry);

                if candidates.len() == 10 {
                    break;
                }
            }
        }

        // clear previously drawn sprites, otherwise data will be junk
        for sprite in &mut self.previous_objs {
            *sprite = None;
        }

        let base_tilemap_address: u16 = 0x8000;

        for sprite in candidates {
            for i in 0..8 {
                let x_pos = i + (sprite.x_position as i16 - 8);

                if x_pos < 0 || x_pos as usize >= SCREEN_WIDTH {
                    continue;
                }

                let mut y_in_tile = self.line_y - (sprite.y_position - 16);

                if sprite.attributes.y_flip {
                    y_in_tile = sprite_height - 1 - y_in_tile;
                }

                let tile_index = if is8by16 {
                    sprite.tile_index & 0xfe
                } else {
                    sprite.tile_index
                };

                let tile_address = base_tilemap_address + tile_index as u16 * 16 + y_in_tile as u16 * 2;

                let lower_byte = self.vram_read8(tile_address as usize, 0);
                let upper_byte = self.vram_read8(tile_address as usize + 1, 0);

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

                if let Some(prev_obj) = self.previous_objs[x_pos as usize] {
                    if (sprite.x_position > prev_obj.x_position) ||
                        (sprite.x_position == prev_obj.x_position && sprite.address > prev_obj.address)
                    {
                        continue;
                    }
                }

                if sprite.attributes.priority == OamPriority::None || (self.prev_background_indexes[x_pos as usize] == 0 && self.prev_window_indexes[x_pos as usize] == 0) {
                    // draw the pixel!
                    let pixel = self.get_pixel(color);

                    self.picture.set_pixel(x_pos as usize, self.line_y as usize, pixel);

                    self.previous_objs[x_pos as usize] = Some(sprite);
                }
            }
        }
    }

    fn draw_gbc_objects(&mut self) {
        if !self.lcdc.contains(LCDControlRegister::OBJ_ENABLE) {
            return;
        }

        let is8by16 = self.lcdc.contains(LCDControlRegister::OBJ_SIZE);

        let sprite_height = if is8by16 {
            16
        } else {
            8
        };

        let mut candidates: Vec<OAMEntry> = Vec::new();

        for i in 0..self.oam.len() {
            let entry = self.oam[i];
             let y_diff: i16 = self.line_y as i16 - (entry.y_position as i16 - 16);
            if y_diff >= 0 && y_diff < sprite_height as i16 {
                let mut entry = entry.clone();

                entry.address = i;

                candidates.push(entry);

                if candidates.len() == 10 {
                    break;
                }
            }
        }

        // clear previously drawn sprites, otherwise data will be junk
        for sprite in &mut self.previous_objs {
            *sprite = None;
        }

        let base_tilemap_address: u16 = 0x8000;

        for sprite in candidates {
            for i in 0..8 {
                let x_pos = i + (sprite.x_position as i16 - 8);

                if x_pos < 0 || x_pos as usize >= SCREEN_WIDTH {
                    continue;
                }

                let mut y_in_tile = self.line_y - (sprite.y_position - 16);

                if sprite.attributes.y_flip {
                    y_in_tile = sprite_height - 1 - y_in_tile;
                }

                let tile_index = if is8by16 {
                    sprite.tile_index & 0xfe
                } else {
                    sprite.tile_index
                };

                let tile_address = base_tilemap_address + tile_index as u16 * 16 + y_in_tile as u16 * 2;

                let lower_byte = self.vram_read8(tile_address as usize, sprite.attributes.bank as usize);
                let upper_byte = self.vram_read8(tile_address as usize + 1, sprite.attributes.bank as usize);

                let bit_index = if sprite.attributes.x_flip { i } else { 7 - i };

                let lower_bit = (lower_byte >> bit_index) & 1;
                let upper_bit = (upper_byte >> bit_index) & 1;

                let palette_index = lower_bit | (upper_bit << 1);

                if palette_index == 0 {
                    continue;
                }

                // let color = if sprite.attributes.dmg_palette == 0 {
                //     self.obp0.indexes[palette_index as usize]
                // } else {
                //     self.obp1.indexes[palette_index as usize]
                // };

                let base_obj_palette_address = sprite.attributes.gbc_palette * 4 * 2;

                let obj_palette_address = base_obj_palette_address + palette_index * 2;

                let color = self.obj_pal_read16(obj_palette_address as usize);

                if let Some(prev_obj) = self.previous_objs[x_pos as usize] {
                    if sprite.address > prev_obj.address {
                        continue;
                    }
                }

                if sprite.attributes.priority == OamPriority::None || (self.prev_background_indexes[x_pos as usize] == 0 && self.prev_window_indexes[x_pos as usize] == 0) {
                    // draw the pixel!
                    let pixel = Self::convert_pixel(color);

                    self.picture.set_pixel(x_pos as usize, self.line_y as usize, pixel);

                    self.previous_objs[x_pos as usize] = Some(sprite);
                }
            }
        }
    }

    fn handle_vblank(&mut self, interrupt_register: &mut InterruptRegister) {
        if self.cycles >= MODE1_CYCLES {
            self.cycles -= MODE1_CYCLES;

            if self.stat.contains(LCDStatusRegister::MODE1) {
                interrupt_register.set(InterruptRegister::LCD, true);
            }

            self.line_y += 1;

            if self.stat.contains(LCDStatusRegister::LYC_INT) && self.line_y == self.lyc {
                interrupt_register.set(InterruptRegister::LCD, true);
            }

            if self.line_y == 154 {
                if self.stat.contains(LCDStatusRegister::MODE2) {
                    interrupt_register.set(InterruptRegister::LCD, true);
                }
                self.frame_finished = true;

                self.mode = LCDMode::OAMScan;
                self.line_y = 0;

                if self.stat.contains(LCDStatusRegister::LYC_INT) && self.line_y == self.lyc {
                    interrupt_register.set(InterruptRegister::LCD, true);
                }

                self.current_window_line = 0;
            }
        }
    }

    fn handle_oam_scan(&mut self, interrupt_register: &mut InterruptRegister) {
        if self.cycles >= MODE2_CYCLES {
            self.cycles -= MODE2_CYCLES;

            if self.stat.contains(LCDStatusRegister::MODE2) {
                interrupt_register.set(InterruptRegister::LCD, true);
            }

            self.mode = LCDMode::HDraw
        }
    }

    fn handle_hdraw(&mut self) {
        if self.cycles >= MODE3_CYCLES {
            self.cycles -= MODE3_CYCLES;

            self.draw_line();

            self.mode = LCDMode::HBlank;
        }
    }

    pub fn update_stat(&mut self, value: u8, interrupt_register: &mut InterruptRegister) {
        self.stat = LCDStatusRegister::from_bits_truncate(value);

        if self.stat.contains(LCDStatusRegister::LYC_INT) && self.line_y == self.lyc {
            interrupt_register.set(InterruptRegister::LCD, true);
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