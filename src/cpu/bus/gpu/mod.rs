use lcd_control_register::LCDControlRegister;
use lcd_status_register::LCDStatusRegister;

pub mod lcd_status_register;
pub mod lcd_control_register;

pub struct GPU {
    pub scy: u8,
    pub scx: u8,
    pub stat: LCDStatusRegister,
    pub lcdc: LCDControlRegister,
    pub line_y: u8
}

impl GPU {
    pub fn new() -> Self {
        Self {
            scy: 0,
            scx: 0,
            stat: LCDStatusRegister::from_bits_truncate(0),
            lcdc: LCDControlRegister::from_bits_retain(0),
            line_y: 0
        }
    }
}