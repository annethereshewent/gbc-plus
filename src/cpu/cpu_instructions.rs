use super::{FlagRegister, Register, CPU};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum JumpFlags {
    NoFlag,
    NZ,
    Z,
    NC,
    C
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

    pub fn to_string(&self) -> String {
        match self {
            Self::NoFlag => "".to_string(),
            Self::C => "C".to_string(),
            Self::NZ => "NZ".to_string(),
            Self::Z => "Z".to_string(),
            Self::NC => "NC".to_string(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum IncrementMode {
    Increment,
    Decrement
}

#[derive(Copy, Clone)]
pub enum AluOp {
    ADD,
    ADC,
    SUB,
    SBC,
    AND,
    XOR,
    OR,
    CP
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LoadType {
    Normal,
    LeftPointer,
    RightPointer
}

impl CPU {
    pub fn nop(&mut self) -> usize {
        4
    }

    pub fn stop(&mut self) -> usize {
        todo!("stop command");
    }

    pub fn jr(&mut self, flag: JumpFlags) -> usize {
        let condition_met = match flag {
            JumpFlags::NoFlag => {
                true
            }
            JumpFlags:: NZ => {
                !self.f.contains(FlagRegister::ZERO)
            }
            JumpFlags::Z => {
                self.f.contains(FlagRegister::ZERO)
            }
            JumpFlags::NC => {
                !self.f.contains(FlagRegister::CARRY)
            }
            JumpFlags::C => {
                self.f.contains(FlagRegister::CARRY)
            }
        };

        let signed_imm = self.bus.mem_read8(self.pc) as i8;

        let cycles: usize = if condition_met {
            self.pc = ((self.pc as i32) + 1 + signed_imm as i32) as u16;

            12
        } else {
            self.pc += 1;

            8
        };

        cycles
    }

    pub fn ld_registers(&mut self, reg1: Register, reg2: Register, load_type: LoadType) -> usize {
        let cycles = if reg1 == Register::SP {
            self.sp = self.hl();

            8
        } else {
            self.registers[reg1 as usize] = self.registers[reg2 as usize];

            4
        };

        cycles
    }

    pub fn ld_immediate_sp(&mut self) -> usize {
        todo!("ld_immediate_sp");
    }

    pub fn ld_immediate(&mut self, reg1: Register, load_type: LoadType) -> usize {
        let (immediate, cycles) = if reg1 != Register::HLPointer && reg1 != Register::SP {
            if reg1 as usize > 6 || load_type != LoadType::Normal {
                // 16 bit value
                let val = self.bus.mem_read16(self.pc);
                self.pc += 2;

                let cycles = if load_type == LoadType::Normal {
                    12
                } else {
                    16
                };

                (val, cycles)
            } else {
                // 8 bit value
                let val = self.bus.mem_read8(self.pc) as u16;
                self.pc += 1;

                (val, 8)
            }
        } else {
            if reg1 == Register::HLPointer {
                let immediate = self.bus.mem_read8(self.pc);
                self.pc += 1;

                (immediate as u16, 12)
            } else {
                let immediate = self.bus.mem_read16(self.pc);
                self.pc += 2;

                (immediate, 12)
            }
        };


        if reg1 == Register::HLPointer {
            self.bus.mem_write8(self.hl(), immediate as u8);
        } else if reg1 == Register::SP {
            self.sp = immediate;
        } else {
            match load_type {
                LoadType::Normal => {
                    if reg1 as usize > 6 {
                        self.set_register16(reg1, immediate);
                    } else {
                        self.registers[reg1 as usize] = immediate as u8;
                    }
                }
                LoadType::LeftPointer => {
                    if reg1 as usize > 6 {
                        panic!("invalid register given to ld_immediate with LeftPointer: {:?}", reg1);
                    } else {
                        self.bus.mem_write8(immediate, self.registers[reg1 as usize]);
                    }
                }
                LoadType::RightPointer => {
                    if reg1 as usize > 6 {
                        panic!("invalid register given to ld_immediate with RightPointer: {:?}", reg1);
                    } else {
                        self.registers[reg1 as usize] = self.bus.mem_read8(immediate);
                    }
                }
            }
        }

        cycles
    }

    pub fn ld_upper(&mut self, reg1: Register, load_type: LoadType, use_c: bool) -> usize {
        let (offset, cycles) = if use_c {
            (self.registers[Register::C as usize], 8)
        } else {
            (self.bus.mem_read8(self.pc), 12)
        };

        self.pc += 1;

        match load_type {
            LoadType::LeftPointer => {
                self.bus.mem_write8(0xff00 + offset as u16, self.registers[reg1 as usize]);
            }
            LoadType::RightPointer => {
                self.registers[reg1 as usize] = self.bus.mem_read8(0xff00 + offset as u16);
            }
            _ => panic!("invalid load type for ld_upper given: {:?}", load_type)
        }

        cycles
    }

    pub fn add_hl(&mut self, register: Register) -> usize {
        todo!("add hl");
    }

    pub fn add(&mut self, register: Option<Register>) -> usize {
        todo!("add");
    }

    pub fn adc(&mut self, register: Option<Register>) -> usize {
        todo!("adc");
    }

    pub fn sub(&mut self, register: Option<Register>) -> usize {
        todo!("sub");
    }

    pub fn sbc(&mut self, register: Option<Register>) -> usize {
        todo!("sbc");
    }

    pub fn and(&mut self, register: Option<Register>) -> usize {
        todo!("and");
    }

    pub fn xor(&mut self, register: Option<Register>) -> usize {
        let (value2, cycles) = if let Some(register) = register {
            if register == Register::HLPointer {
                (self.bus.mem_read8(self.hl()), 8)
            } else {
                (self.registers[register as usize], 4)
            }
        } else {
            (self.bus.mem_read8(self.pc), 8)
        };

        self.registers[Register::A as usize] = self.xor_(self.registers[Register::A as usize], value2);

        cycles
    }

    fn xor_(&mut self, val1: u8, val2: u8) -> u8 {
        let result = val1 ^ val2;

        self.f.set(FlagRegister::SUBTRACT, false);
        self.f.set(FlagRegister::HALF_CARRY, false);
        self.f.set(FlagRegister::CARRY, false);
        self.f.set(FlagRegister::ZERO, result == 0);

        result

    }

    pub fn or(&mut self, register: Option<Register>) -> usize {

        let cycles = if let Some(register) = register {
            if register == Register::HLPointer {
                let value = self.bus.mem_read8(self.hl());

                let result = self.registers[Register::A as usize] | value;

                self.bus.mem_write8(self.hl(), result);
                8
            } else {

                self.registers[Register::A as usize] = self.registers[Register::A as usize] | self.registers[register as usize];


                self.f.set(FlagRegister::ZERO, self.registers[Register::A as usize] == 0);
                self.f.set(FlagRegister::SUBTRACT, false);
                self.f.set(FlagRegister::HALF_CARRY, false);
                self.f.set(FlagRegister::CARRY, false);
                4
            }
        } else {
            let value = self.bus.mem_read8(self.pc);
            self.pc += 1;

            self.registers[Register::A as usize] = self.registers[Register::A as usize] | value;

            8
        };

        cycles
    }

    pub fn cp(&mut self, register: Option<Register>) -> usize {
        let a_val = self.registers[Register::A as usize];

        let (operand, cycles) = if let Some(register) = register {
            if register == Register::HLPointer {
                (self.bus.mem_read8(self.hl()), 8)
            } else {
                (self.registers[register as usize], 4)
            }
        } else {
            (self.bus.mem_read8(self.pc), 8)
        };

        self.pc += 1;

        self.subtract(a_val, operand);

        cycles
    }

    fn subtract(&mut self, val1: u8, val2: u8) -> u8 {
        let result = val1 - val2;

        let half_carry = (result & 0xf) > (val1 & 0xf);

        self.f.set(FlagRegister::HALF_CARRY, half_carry);
        self.f.set(FlagRegister::CARRY, result > val1);
        self.f.set(FlagRegister::SUBTRACT, true);
        self.f.set(FlagRegister::ZERO, result == 0);

        result
    }

    pub fn store_hl_ptr(&mut self, r1: Register, increment_mode: IncrementMode) -> usize {
        self.bus.mem_write8(self.hl(), self.registers[r1 as usize]);

        match increment_mode {
            IncrementMode::Decrement => self.dec_register16(Register::HL),
            IncrementMode::Increment => self.inc_register16(Register::HL)
        }

        4
    }

    pub fn load_hl_ptr(&mut self, r1: Register, increment_mode: IncrementMode) -> usize {
        self.registers[r1 as usize] = self.bus.mem_read8(self.hl());

        match increment_mode {
            IncrementMode::Decrement => self.dec_register16(Register::HL),
            IncrementMode::Increment => self.inc_register16(Register::HL)
        }

        4
    }

    pub fn ld_hl_displacement(&mut self) -> usize {
        todo!("load_hl_displacement");
    }

    pub fn inc(&mut self, r1: Register) -> usize {
        todo!("inc");
    }

    pub fn dec(&mut self, r1: Register) -> usize {
        let cycles = if r1 as usize > 6 {
            if r1 != Register::HLPointer {
                self.dec_register16(r1);

                8
            } else {
                let val = self.bus.mem_read8(self.hl());

                self.bus.mem_write8(self.hl(), val - 1);

                12
            }
        } else {
            let result = self.registers[r1 as usize] - 1;

            self.f.set(FlagRegister::ZERO, result == 0);
            self.f.set(FlagRegister::SUBTRACT, true);
            self.f.set(FlagRegister::HALF_CARRY, (result & 0xf) > (self.registers[r1 as usize] & 0xf));

            self.registers[r1 as usize] = result;

            4
        };

        cycles
    }

    pub fn rlca(&mut self) -> usize {
        todo!("rlca");
    }

    pub fn rrca(&mut self) -> usize {
        todo!("rrca");
    }

    pub fn rra(&mut self) -> usize {
        todo!("rra");
    }

    pub fn rla(&mut self) -> usize {
        todo!("rla");
    }

    pub fn daa(&mut self) -> usize {
        todo!("daa");
    }

    pub fn cpl(&mut self) -> usize {
        todo!("cpl");
    }

    pub fn scf(&mut self) -> usize {
        todo!("scf");
    }

    pub fn ccf(&mut self) -> usize {
        todo!("ccf");
    }

    pub fn halt(&mut self) -> usize {
        todo!("halt");
    }

    pub fn ret(&mut self, flags: JumpFlags) -> usize {
        let condition_met = match flags {
            JumpFlags::NoFlag => true,
            JumpFlags::C => self.f.contains(FlagRegister::CARRY),
            JumpFlags::NC => !self.f.contains(FlagRegister::CARRY),
            JumpFlags::Z => self.f.contains(FlagRegister::ZERO),
            JumpFlags::NZ => !self.f.contains(FlagRegister::ZERO)
        };

        let cycles = if condition_met {
            if flags == JumpFlags::NoFlag {
                16
            } else {
                20
            }
        } else {
            8
        };

        if condition_met {
            self.pc = self.pop_from_stack();
        }

        cycles
    }

    pub fn add_sp(&mut self) -> usize {
        todo!("add_sp");
    }

    pub fn pop(&mut self, r1: Register) -> usize {
        todo!("pop");
    }

    pub fn reti(&mut self) -> usize {
        todo!("reti");
    }

    pub fn jp_hl(&mut self) -> usize {
        todo!("jp_hl");
    }

    pub fn jp(&mut self, flags: JumpFlags) -> usize {
        let condition_met = match flags {
            JumpFlags::NoFlag => true,
            JumpFlags::NC => !self.f.contains(FlagRegister::CARRY),
            JumpFlags::Z => self.f.contains(FlagRegister::ZERO),
            JumpFlags::NZ => !self.f.contains(FlagRegister::ZERO),
            JumpFlags::C => self.f.contains(FlagRegister::CARRY)
        };

        let address = self.bus.mem_read16(self.pc);

        let cycles = if condition_met {
            self.pc = address;

            12
        } else {
            self.pc += 2;

            8
        };

        cycles
    }

    pub fn ei(&mut self) -> usize {
        self.bus.ime = true;

        4
    }

    pub fn di(&mut self) -> usize {
        self.bus.ime = false;

        4
    }

    pub fn call(&mut self, flags: JumpFlags) -> usize {
        let address = self.bus.mem_read16(self.pc);

        self.pc += 2;

        self.push_to_stack(self.pc);

        self.pc = address;

        24
    }

    pub fn push(&mut self, r1: Register) -> usize {
        todo!("push");
    }

    pub fn rst(&mut self, y: u8) -> usize {
        todo!("rst");
    }

    pub fn decode_instruction(&mut self, instruction: u8) -> usize {
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
                        1 => self.add_hl(RP_TABLE[p as usize]),
                        _ => unreachable!()
                    }
                }
                2 => {
                    match q {
                        0 => match p {
                            0 => self.ld_registers(Register::BC, Register::A, LoadType::LeftPointer),
                            1 => self.ld_registers(Register::DE, Register::A, LoadType::LeftPointer),
                            2 => self.store_hl_ptr(Register::A, IncrementMode::Increment),
                            3 => self.store_hl_ptr(Register::A, IncrementMode::Decrement),
                            _ => unreachable!()
                        }
                        1 => match p {
                            0 => self.ld_registers(Register::A, Register::BC, LoadType::RightPointer),
                            1 => self.ld_registers(Register::A, Register::DE, LoadType::RightPointer),
                            2 => self.load_hl_ptr(Register::A, IncrementMode::Increment),
                            3 => self.load_hl_ptr(Register::A, IncrementMode::Decrement),
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
                4 => self.inc(R_TABLE[y as usize]),
                5 => self.dec(R_TABLE[y as usize]),
                6 => self.ld_immediate(R_TABLE[y as usize], LoadType::Normal),
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
                    self.halt()
                } else {
                    self.ld_registers(R_TABLE[y as usize], R_TABLE[z as usize], LoadType::Normal)
                }
            }
            2 => {
                match ALU_TABLE[y as usize] {
                    AluOp::ADD => self.add(Some(R_TABLE[z as usize])),
                    AluOp::ADC => self.adc(Some(R_TABLE[z as usize])),
                    AluOp::SUB => self.sub(Some(R_TABLE[z as usize])),
                    AluOp::SBC => self.sbc(Some(R_TABLE[z as usize])),
                    AluOp::AND => self.and(Some(R_TABLE[z as usize])),
                    AluOp::CP => self.cp(Some(R_TABLE[z as usize])),
                    AluOp::OR => self.or(Some(R_TABLE[z as usize])),
                    AluOp::XOR => self.xor(Some(R_TABLE[z as usize]))
                }
            }
            3 => {
                match z {
                    0 => match y {
                        0..=3 => self.ret(CC_TABLE[y as usize]),
                        4 => self.ld_upper(Register::A, LoadType::LeftPointer, false),
                        5 => self.add_sp(),
                        6 => self.ld_upper(Register::A, LoadType::RightPointer, false),
                        7 => self.ld_hl_displacement(),
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
                            todo!("CB instructions");
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
                                self.call(JumpFlags::NoFlag)
                            } else {
                                panic!("invalid option for p given: {p}");
                            }
                        }
                        _ => unreachable!()
                    }
                    6 => {
                        match ALU_TABLE[y as usize] {
                            AluOp::ADD => self.add(None),
                            AluOp::ADC => self.adc(None),
                            AluOp::SUB => self.sub(None),
                            AluOp::SBC => self.sbc(None),
                            AluOp::AND => self.and(None),
                            AluOp::CP => self.cp(None),
                            AluOp::OR => self.or(None),
                            AluOp::XOR => self.xor(None)
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