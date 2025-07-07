#[derive(Copy, Clone)]
pub struct OAMEntry {
    pub y_position: u8,
    pub x_position: u8,
    pub tile_index: u8,
    pub attributes: u8,
}

impl OAMEntry {
    pub fn new() -> Self {
        Self {
            x_position: 0,
            y_position: 0,
            tile_index: 0,
            attributes: 0
        }
    }
}