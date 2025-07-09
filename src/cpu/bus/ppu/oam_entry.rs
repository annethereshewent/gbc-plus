use super::{OamAttributes, OamPriority};

#[derive(Copy, Clone)]
pub struct OAMEntry {
    pub y_position: u8,
    pub x_position: u8,
    pub tile_index: u8,
    pub attributes: OamAttributes,
}

impl OAMEntry {
    pub fn new() -> Self {
        Self {
            x_position: 0,
            y_position: 0,
            tile_index: 0,
            attributes: OamAttributes {
                x_flip: false,
                y_flip: false,
                priority: OamPriority::None,
                dmg_palette: 0
            }
        }
    }
}