use bitflags::bitflags;
use bus::Bus;

pub mod bus;
pub mod cpu_instructions;

bitflags! {
    pub struct FlagRegister: u8 {
        const CARRY = 1 << 4;
        const HALF_CARRY = 1 << 5;
        const SUBTRACT = 1 << 6;
        const ZERO = 1 << 7;
    }
}

#[derive(Copy, Clone)]
pub enum Register {
    A = 0,
    F = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    H = 6,
    L = 7,
    AF = 8,
    BC = 10,
    DE = 12,
    HL = 14,
    SP = 16,
    HLPointer = 17
}

pub struct CPU {
    registers: [u8; 7],
    pc: u16,
    sp: u16,
    f: FlagRegister,
    bus: Bus,
    cycles: usize
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            // a: 0x1,
            // b: 0,
            // c: 0x13,
            // d: 0,
            // e: 0xd8,
            // h: 0x1,
            // l: 0x4d,
            registers: [0x1, 0, 0x13, 0, 0xd8, 0x1, 0x4d],
            pc: 0x100,
            sp: 0xfffe,
            f: FlagRegister::from_bits_retain(0xb0),
            bus: Bus::new(),
            cycles: 0
        }
    }

    pub fn step(&mut self) {
        let opcode = self.bus.mem_read8(self.pc);

        println!("[Opcode: 0x{:x}] [Address: 0x{:x}]", opcode, self.pc);

        self.pc += 1;

        self.decode_instruction(opcode);
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        self.bus.rom = bytes.to_vec();
    }
}