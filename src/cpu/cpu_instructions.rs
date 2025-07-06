use super::{Register, CPU};

#[derive(Copy, Clone)]
enum JumpFlags {
    NoFlag,
    NZ,
    Z,
    NC,
    C
}

enum IncrementMode {
    Increment,
    Decrement
}

enum AluOp {
    ADD,
    ADC,
    SUB,
    SBC,
    AND,
    XOR,
    OR,
    CP
}

impl JumpFlags {
    pub fn new(val: u8) -> Self {
        match val {
            0 => Self::NZ,
            1 => Self::Z,
            2 => Self::NC,
            3 => Self::C,
            _ => unreachable!()
        }
    }
}

pub const RP_TABLE: [Register; 4] = [
    Register::BC,
    Register::DE,
    Register::HL,
    Register::SP
];

pub const R_TABLE: [Register; 8] = [
    Register::B,
    Register::C,
    Register::D,
    Register::E,
    Register::H,
    Register::L,
    Register::HLPointer,
    Register::A
];

pub const ALU_TABLE: [AluOp; 8] = [
    AluOp::ADD,
    AluOp::ADC,
    AluOp::SUB,
    AluOp::SBC,
    AluOp::AND,
    AluOp::XOR,
    AluOp::OR,
    AluOp::CP
];

pub const CC_TABLE: [JumpFlags; 4] = [
    JumpFlags::NZ,
    JumpFlags::Z,
    JumpFlags::NC,
    JumpFlags::C
];

pub const RP2_TABLE: [Register; 4] = [
    Register::BC,
    Register::DE,
    Register::HL,
    Register::AF
];

enum LoadType {
    Normal,
    LeftPointer,
    RightPointer
}

impl CPU {
    pub fn nop(&mut self) {

    }

    pub fn stop(&mut self) {

    }

    pub fn jr(&mut self, flag: JumpFlags) {
        match flag {
            JumpFlags::NoFlag => {

            }
            JumpFlags:: NZ => {

            }
            JumpFlags::Z => {

            }
            JumpFlags::NC => {

            }
            JumpFlags::C => {

            }
        }
    }

    pub fn ld_registers(&mut self, reg1: Register, reg2: Register, load_type: LoadType) {

    }

    pub fn ld_immediate_sp(&mut self) {

    }

    pub fn ld_immediate(&mut self, reg1: Register, load_type: LoadType) {

    }

    pub fn ld_upper(&mut self, reg1: Register, load_type: LoadType, use_c: bool) {

    }

    pub fn add(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn adc(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn sub(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn sbc(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn and(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn xor(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn or(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn cp(&mut self, reg1: Register, reg2: Register) {

    }

    pub fn add_imm(&mut self) {

    }

    pub fn adc_imm(&mut self) {

    }

    pub fn sub_imm(&mut self) {

    }

    pub fn sbc_imm(&mut self) {

    }

    pub fn and_imm(&mut self) {

    }

    pub fn xor_imm(&mut self) {

    }

    pub fn or_imm(&mut self) {

    }

    pub fn cp_imm(&mut self) {

    }

    pub fn store_hl(&mut self, r1: Register, increment_mode: IncrementMode) {

    }

    pub fn load_hl(&mut self, r1: Register, increment_mode: IncrementMode) {

    }

    pub fn load_hl_displacement(&mut self) {

    }

    pub fn inc(&mut self, r1: Register) {

    }

    pub fn dec(&mut self, r1: Register) {

    }

    pub fn rlca(&mut self) {

    }

    pub fn rrca(&mut self) {

    }

    pub fn rra(&mut self) {

    }

    pub fn rla(&mut self) {

    }

    pub fn daa(&mut self) {

    }

    pub fn cpl(&mut self) {

    }

    pub fn scf(&mut self) {

    }

    pub fn ccf(&mut self) {

    }

    pub fn halt(&mut self) {

    }

    pub fn ret(&mut self, flags: JumpFlags) {

    }

    pub fn add_sp(&mut self) {

    }

    pub fn pop(&mut self, r1: Register) {

    }

    pub fn reti(&mut self) {

    }

    pub fn jp_hl(&mut self) {

    }

    pub fn jp(&mut self, flags: JumpFlags) {

    }

    pub fn ei(&mut self) {

    }

    pub fn di(&mut self) {

    }

    pub fn call(&mut self, flags: JumpFlags) {

    }

    pub fn push(&mut self, r1: Register) {

    }

    pub fn rst(&mut self, y: u8) {

    }

    pub fn decode_instruction(&mut self, instruction: u8) {
        let z = instruction & 0x7;
        let q = (instruction >> 3) & 1;
        let p = (instruction >> 4) & 0x3;
        let y = (instruction >> 3) & 0x7;
        let x = (instruction >> 6) & 0x3;

        match x {
            0 => match z {
                0 => {
                    match y {
                        0 => self.nop(),
                        1 => self.ld_immediate_sp(),
                        2 => self.stop(),
                        3 => self.jr(JumpFlags::NoFlag),
                        4..=7 => self.jr(JumpFlags::new(y - 4)),
                        _ => unreachable!()
                    }
                }
                1 => {
                    match q {
                        0 => self.ld_immediate(RP_TABLE[p as usize], LoadType::Normal),
                        1 => self.add(Register::HL, RP_TABLE[p as usize]),
                        _ => unreachable!()
                    }
                }
                2 => {
                    match q {
                        0 => match p {
                            0 => self.ld_registers(Register::BC, Register::A, LoadType::LeftPointer),
                            1 => self.ld_registers(Register::DE, Register::A, LoadType::LeftPointer),
                            2 => self.store_hl(Register::A, IncrementMode::Increment),
                            3 => self.store_hl(Register::A, IncrementMode::Decrement),
                            _ => unreachable!()
                        }
                        1 => match p {
                            0 => self.ld_registers(Register::A, Register::BC, LoadType::RightPointer),
                            1 => self.ld_registers(Register::A, Register::DE, LoadType::RightPointer),
                            2 => self.load_hl(Register::A, IncrementMode::Increment),
                            3 => self.load_hl(Register::A, IncrementMode::Decrement),
                            _ => unreachable!()
                        }
                        _ => unreachable!()
                    }
                }
                3 => {
                    match q {
                        0 => self.inc(RP_TABLE[p as usize]),
                        1 => self.dec(RP_TABLE[p as usize]),
                        _ => unreachable!()
                    }
                }
                4 => self.inc(R_TABLE[p as usize]),
                5 => self.dec(R_TABLE[p as usize]),
                6 => self.ld_immediate(R_TABLE[p as usize], LoadType::Normal),
                7 => match y {
                    0 => self.rlca(),
                    1 => self.rrca(),
                    2 => self.rla(),
                    3 => self.rra(),
                    4 => self.daa(),
                    5 => self.cpl(),
                    6 => self.scf(),
                    7 => self.ccf(),
                    _ => unreachable!()
                }
                _ => unreachable!()
            }
            1 => {
                if z == 6 && y == 6 {
                    self.halt();
                } else {
                    self.ld_registers(R_TABLE[y as usize], R_TABLE[z as usize], LoadType::Normal);
                }
            }
            2 => {
                match ALU_TABLE[y as usize] {
                    AluOp::ADD => self.add(Register::A, R_TABLE[z as usize]),
                    AluOp::ADC => self.adc(Register::A, R_TABLE[z as usize]),
                    AluOp::SUB => self.sub(Register::A, R_TABLE[z as usize]),
                    AluOp::SBC => self.sbc(Register::A, R_TABLE[z as usize]),
                    AluOp::AND => self.and(Register::A, R_TABLE[z as usize]),
                    AluOp::CP => self.cp(Register::A, R_TABLE[z as usize]),
                    AluOp::OR => self.or(Register::A, R_TABLE[z as usize]),
                    AluOp::XOR => self.xor(Register::A, R_TABLE[z as usize])
                }
            }
            3 => {
                match z {
                    0 => match y {
                        0..=3 => self.ret(CC_TABLE[y as usize]),
                        4 => self.ld_upper(Register::A, LoadType::LeftPointer, false),
                        5 => self.add_sp(),
                        6 => self.ld_upper(Register::A, LoadType::RightPointer, false),
                        7 => self.load_hl_displacement(),
                        _ => unreachable!()
                    }
                    1 => match q {
                        0 => self.pop(RP2_TABLE[p as usize]),
                        1 => match p {
                            0 => self.ret(JumpFlags::NoFlag),
                            1 => self.reti(),
                            2 => self.jp_hl(),
                            3 => self.ld_registers(Register::SP, Register::HL, LoadType::Normal),
                            _ => unreachable!()
                        }
                        _ => unreachable!()
                    }
                    2 => match y {
                        0..=3 => self.jp(CC_TABLE[y as usize]),
                        4 => self.ld_upper(Register::A, LoadType::LeftPointer, true),
                        5 => self.ld_immediate(Register::A, LoadType::LeftPointer),
                        6 => self.ld_upper(Register::A, LoadType::RightPointer, true),
                        7 => self.ld_immediate(Register::A, LoadType::RightPointer),
                        _ => unreachable!()
                    }
                    3 => match y {
                        0 => self.jp(JumpFlags::NoFlag),
                        1 => {
                            // TODO: CB instructions
                        }
                        6 => self.di(),
                        7 => self.ei(),
                        _ => panic!("invalid parameter for y given: {y}")
                    }
                    4 => match y {
                        0..=3 => self.call(CC_TABLE[y as usize]),
                        _ => panic!("invalid option for y given: {y}")
                    }
                    5 => match q {
                        0 => self.push(RP2_TABLE[p as usize]),
                        1 => {
                            if p == 0 {
                                self.call(JumpFlags::NoFlag);
                            } else {
                                panic!("invalid option for p given: {p}");
                            }
                        }
                        _ => unreachable!()
                    }
                    6 => {
                        match ALU_TABLE[y as usize] {
                            AluOp::ADD => self.add_imm(),
                            AluOp::ADC => self.adc_imm(),
                            AluOp::SUB => self.sub_imm(),
                            AluOp::SBC => self.sbc_imm(),
                            AluOp::AND => self.and_imm(),
                            AluOp::CP => self.cp_imm(),
                            AluOp::OR => self.or_imm(),
                            AluOp::XOR => self.xor_imm()
                        }
                    }
                    7 => self.rst(y * 8),
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }
    }
}