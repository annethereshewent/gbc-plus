use std::{collections::{HashSet, VecDeque}, sync::{Arc, Mutex}};

use bitflags::bitflags;
use bus::{interrupt_register::InterruptRegister, Bus};

pub mod bus;
pub mod cpu_instructions;
pub mod disassembler;

pub const CLOCK_SPEED: usize = 4194304;
const CGB_ADDR: usize = 0x143;

bitflags! {
    pub struct FlagRegister: u8 {
        const CARRY = 1 << 4;
        const HALF_CARRY = 1 << 5;
        const SUBTRACT = 1 << 6;
        const ZERO = 1 << 7;
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Register {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    H = 5,
    L = 6,
    BC = 8,
    DE = 10,
    HL = 12,
    SP = 14,
    HLPointer = 15,
    AF = 16
}

pub struct CPU {
    registers: [u8; 7],
    pc: u16,
    sp: u16,
    f: FlagRegister,
    pub bus: Bus,
    found: HashSet<u16>,
    pub debug_on: bool,
    is_halted: bool
}

impl CPU {
    pub fn new(audio_buffer: Arc<Mutex<VecDeque<f32>>>, rom_path: &str) -> CPU {
        CPU {
            registers: [0x1, 0, 0x13, 0, 0xd8, 0x1, 0x4d],
            pc: 0x100,
            sp: 0xfffe,
            f: FlagRegister::from_bits_retain(0xb0),
            bus: Bus::new(audio_buffer, rom_path),
            found: HashSet::new(),
            debug_on: false,
            is_halted: false
        }
    }

    pub fn push_to_stack(&mut self, value: u16) {
        self.sp -= 2;

        self.bus.mem_write16(self.sp, value);
    }

    pub fn pop_from_stack(&mut self) -> u16 {
        let value = self.bus.mem_read16(self.sp);

        self.sp += 2;

        value
    }

    pub fn step(&mut self) {
        self.handle_interrupts();

        let previous_pc = self.pc;
        if self.is_halted {
            self.bus.tick(4);

            return;
        }

        let opcode = self.bus.mem_read8(self.pc);

        self.pc += 1;

        if !self.found.contains(&(self.pc - 1)) && self.debug_on {
            println!("[Opcode: 0x{:x}] [Address: 0x{:x}] {}", opcode, previous_pc, self.disassemble(opcode));
            self.found.insert(self.pc - 1);
        }

        let cycles = self.decode_instruction(opcode);

        self.bus.tick(cycles);
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.bus.cartridge.rom = bytes.to_vec();

        self.check_cgb_header();
        self.bus.check_header();
    }

    fn check_cgb_header(&mut self) {
        let cgb_flag = self.bus.cartridge.rom[CGB_ADDR];
        if [0x80, 0xc0].contains(&cgb_flag) {
            self.bus.ppu.cgb_mode = true;
            self.update_cgb_registers();
        }
    }

    fn update_cgb_registers(&mut self) {
        use Register::*;

        self.registers[Register::A as usize] = 0x11;

        self.f.set(FlagRegister::ZERO, true);
        self.f.set(FlagRegister::SUBTRACT, false);
        self.f.set(FlagRegister::CARRY, false);
        self.f.set(FlagRegister::HALF_CARRY, false);

        self.registers[B as usize] = 0x0;
        self.registers[C as usize] = 0x0;
        self.registers[D as usize] = 0xff;
        self.registers[E as usize] = 0x56;

        self.registers[H as usize] = 0x0;
        self.registers[L as usize] = 0xd;

        self.sp = 0xfffe;
        self.pc = 0x100;
    }

    pub fn handle_interrupts(&mut self) {
        let fired_interrupts = self.bus.IF.bits() & self.bus.ie.bits();

        if fired_interrupts != 0 {
            self.is_halted = false;
        }

        if self.bus.ime {
            if fired_interrupts != 0 {
                let irq_index = fired_interrupts.trailing_zeros();


                self.bus.IF = InterruptRegister::from_bits_truncate(self.bus.IF.bits() & !(1 << irq_index));
                self.bus.ime = false;

                // use the InterruptRegister struct to determine what interrupt fired
                let temp = InterruptRegister::from_bits_retain(1 << irq_index);

                self.bus.tick(8);

                self.push_to_stack(self.pc);

                self.bus.tick(8);

                if temp.contains(InterruptRegister::VBLANK) {
                    self.pc = 0x40;
                } else if temp.contains(InterruptRegister::LCD) {
                    self.pc = 0x48;
                } else if temp.contains(InterruptRegister::TIMER) {
                    self.pc = 0x50;
                } else if temp.contains(InterruptRegister::SERIAL) {
                    self.pc = 0x58;
                } else if temp.contains(InterruptRegister::JOYPAD) {
                    self.pc = 0x60;
                }

                self.bus.tick(4);
            }
        }
    }

    pub fn set_register16(&mut self, r1: Register, val: u16) {
        let base = r1 as usize - 7;

        self.registers[base] = (val >> 8) as u8;
        self.registers[base + 1] = val as u8;
    }

    pub fn get_register16(&mut self, r1: Register) -> u16 {
        let base = r1 as usize - 7;

        (self.registers[base] as u16) << 8 | self.registers[base + 1] as u16
    }

    pub fn dec_register16(&mut self, r1: Register) {
        let base = r1 as usize - 7;

        let result = ((self.registers[base] as u16) << 8 | self.registers[base + 1] as u16) - 1;

        self.registers[base] = (result >> 8) as u8;
        self.registers[base + 1] = result as u8;
    }

    pub fn inc_register16(&mut self, r1: Register) {
        let base = r1 as usize - 7;

        let result = ((self.registers[base] as u16) << 8 | self.registers[base + 1] as u16) + 1;

        self.registers[base] = (result >> 8) as u8;
        self.registers[base + 1] = result as u8;
    }

    pub fn hl(&self) -> u16 {
        (self.registers[Register::H as usize] as u16) << 8 | self.registers[Register::L as usize] as u16
    }
}