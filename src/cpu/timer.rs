use bitflags::bitflags;

bitflags! {
    pub struct TimerControl: u8 {
        const ENABLE = 1 << 2;
    }
}

impl TimerControl {
    pub fn clock_select(&self) -> u8 {
        self.bits() & 0x3
    }
}

pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    tac: TimerControl,
    interval: usize
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: TimerControl::from_bits_retain(0),
            interval: 0
        }
    }

    pub fn tick(&mut self, cycles: usize) {

    }

    pub fn update_tac(&mut self, val: u8) {
        self.tac = TimerControl::from_bits_retain(val);

        // numbers are in m-cycles, but emulator counts in t-cycles, so we need to multiply by 4
        match self.tac.clock_select() {
            0 => self.interval = 256 * 4,
            1 => self.interval = 4 * 4,
            2 => self.interval = 16 * 4,
            3 => self.interval = 64 * 4,
            _ => unreachable!()
        }
    }
}