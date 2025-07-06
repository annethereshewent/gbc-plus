use lcd_control_register::LCDControlRegister;
use lcd_status_register::LCDStatusRegister;

pub mod lcd_status_register;
pub mod lcd_control_register;

const MODE2_CYCLES: usize = 80;
const MODE3_CYCLES: usize = 172;
const MODE0_CYCLES: usize = 204;
const MODE1_CYCLES: usize = 456;


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
    pub stat: LCDStatusRegister,
    pub lcdc: LCDControlRegister,
    pub line_y: u8,
    pub vram: Box<[u8]>,
    pub cycles: usize,
    mode: LCDMode
}

impl PPU {
    pub fn new() -> Self {
        Self {
            scy: 0,
            scx: 0,
            stat: LCDStatusRegister::from_bits_truncate(0),
            lcdc: LCDControlRegister::from_bits_retain(0),
            mode: LCDMode::OAMScan,
            line_y: 0,
            vram: vec![0; 0x2000].into_boxed_slice(),
            cycles: 0
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        if self.lcdc.contains(LCDControlRegister::LCD_AND_PPU_ENABLE) {
            self.cycles += cycles;

            match self.mode {
                LCDMode::HBlank => self.handle_hblank(),
                LCDMode::VBlank => self.handle_vblank(),
                LCDMode::OAMScan => self.handle_oam_scan(),
                LCDMode::HDraw => self.handle_hdraw(),
                _ => unreachable!()
            }
        }
    }

    fn handle_hblank(&mut self) {
        if self.cycles >= MODE0_CYCLES {
            self.cycles -= MODE0_CYCLES;
            self.line_y += 1;
            self.mode = if self.line_y == 144 {
                LCDMode::VBlank
            } else {
                LCDMode::OAMScan
            };
        }
    }

    fn handle_vblank(&mut self) {
        if self.cycles >= 456 {
            self.cycles -= 456;

            self.line_y += 1;

            if self.line_y == 154 {
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

    fn handle_hdraw(&mut self) {
        if self.cycles >= MODE3_CYCLES {
            self.cycles -= MODE3_CYCLES;
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
}