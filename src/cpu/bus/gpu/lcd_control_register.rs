use bitflags::bitflags;

bitflags! {
    pub struct LCDControlRegister: u8 {
        const BG_WINDOW_ENABLE_PRIORITY = 1;
        const OBJ_ENABLE = 1 << 1;
        const OBJ_SIZE = 1 << 2;
        const BG_TILEMAP = 1 << 3;
        const BG_AND_WINDOW_TILES = 1 << 4;
        const WINDOW_ENABLE = 1 << 5;
        const WINDOW_TILEMAP = 1 << 6;
        const LCD_AND_PPU_ENABLE = 1 << 7;
    }
}