use bitflags::bitflags;
use bus::Bus;
use timer::Timer;

pub mod bus;
pub mod cpu_instructions;
pub mod disassembler;
pub mod timer;

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
    SP = 13,
    HLPointer = 14,
    AF = 15
}

pub struct CPU {
    registers: [u8; 7],
    pc: u16,
    sp: u16,
    f: FlagRegister,
    bus: Bus,
    timer: Timer
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            registers: [0x1, 0, 0x13, 0, 0xd8, 0x1, 0x4d],
            pc: 0x100,
            sp: 0xfffe,
            f: FlagRegister::from_bits_retain(0xb0),
            bus: Bus::new(),
            timer: Timer::new()
        }
    }

    pub fn step(&mut self) {
        self.handle_interrupts();
        let opcode = self.bus.mem_read8(self.pc);

        self.pc += 1;

        println!("[Opcode: 0x{:x}] [Address: 0x{:x}] {}", opcode, self.pc, self.disassemble(opcode));

        let cycles = self.decode_instruction(opcode);

        self.bus.ppu.tick(cycles);
        self.timer.tick(cycles);

    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.bus.cartridge.rom = bytes.to_vec();
    }

    pub fn handle_interrupts(&mut self) {
        if self.bus.ime && (self.bus.IF.bits() & self.bus.ie.bits()) != 0 {
            panic!("interrupt happening! how exciting!");
        }
    }

    pub fn set_register16(&mut self, r1: Register, val: u16) {
        let base = r1 as usize - 7;

        self.registers[base] = (val >> 8) as u8;
        self.registers[base + 1] = val as u8;
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