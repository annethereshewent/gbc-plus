use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use super::interrupt_register::InterruptRegister;

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct TimerControl: u8 {
        const ENABLE = 1 << 2;
    }
}

impl TimerControl {
    pub fn clock_select(&self) -> u8 {
        self.bits() & 0x3
    }
}

pub const DIV_CYCLES: usize = 256;

#[derive(Serialize, Deserialize)]
pub struct Timer {
    pub div: u8,
    pub tima: u8,
    pub tma: u8,
    pub tac: TimerControl,
    pub interval: usize,
    cycles: usize,
    div_cycles: usize
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: TimerControl::from_bits_retain(0),
            interval: 0,
            cycles: 0,
            div_cycles: 0
        }
    }

    pub fn tick(&mut self, cycles: usize, interrupt_register: &mut InterruptRegister) {
        self.div_cycles += cycles;
        if self.tac.contains(TimerControl::ENABLE) {
            self.cycles += cycles;

            if self.cycles >= self.interval {
                self.cycles -= self.interval;

                let (result, overflow) = self.tima.overflowing_add(1);

                self.tima = result;

                if overflow {
                    self.tima = self.tma;
                    // request interrupt
                    interrupt_register.set(InterruptRegister::TIMER, true);
                }
            }
        }

        if self.div_cycles >= DIV_CYCLES {
            self.div_cycles -= DIV_CYCLES;
            self.div += 1;
        }
    }

    pub fn write_tima(&mut self, value: u8) {
        self.tima = value;
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