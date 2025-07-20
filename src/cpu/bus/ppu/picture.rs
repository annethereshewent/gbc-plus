use super::{SCREEN_HEIGHT, SCREEN_WIDTH};

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub struct Picture {
    pub data: Vec<u8>
}

impl Picture {
    pub fn new() -> Self {
        Self {
            data: vec![0; 4 * SCREEN_WIDTH * SCREEN_HEIGHT]
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Color) {
        let i = (x + y * SCREEN_WIDTH) * 4;
        self.data[i] = pixel.r;
        self.data[i + 1] = pixel.g;
        self.data[i + 2] = pixel.b;
        self.data[i + 3] = 0xff;

    }
}