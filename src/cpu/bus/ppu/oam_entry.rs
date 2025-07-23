use serde::{Deserialize, Serialize};

use super::{OamAttributes, OamPriority};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct OAMEntry {
    pub y_position: u8,
    pub x_position: u8,
    pub tile_index: u8,
    pub attributes: OamAttributes,
    pub address: usize
}

impl OAMEntry {
    pub fn new() -> Self {
        Self {
            x_position: 0,
            y_position: 0,
            tile_index: 0,
            address: 0,
            attributes: OamAttributes {
                x_flip: false,
                y_flip: false,
                priority: OamPriority::None,
                dmg_palette: 0,
                gbc_palette: 0,
                bank: 0
            }
        }
    }
}