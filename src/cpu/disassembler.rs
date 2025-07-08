use crate::cpu::FlagRegister;

use super::{cpu_instructions::{AluOp, CBOp, IncrementMode, JumpFlags, LoadType, ALU_TABLE, CC_TABLE, RP2_TABLE, RP_TABLE, R_TABLE}, Register, CPU};

impl CPU {

    fn diss_ld_immediate_sp(&self) -> String {
        let immediate = self.bus.mem_read16(self.pc);

        format!("LD ({:x}), SP", immediate)
    }

    fn diss_jr(&self, flag: JumpFlags) -> String {
        let displacement = self.bus.mem_read8(self.pc) as i8;

        let operand = format!("0x{:x}", self.pc as i32 + displacement as i32 + 1);

        let condition_met = match flag {
            JumpFlags::C => self.f.contains(FlagRegister::CARRY),
            JumpFlags::NoFlag => true,
            JumpFlags::NZ => !self.f.contains(FlagRegister::ZERO),
            JumpFlags::Z => self.f.contains(FlagRegister::ZERO),
            JumpFlags::NC => !self.f.contains(FlagRegister::CARRY),
        };

        format!("JR {} {operand} (branch: {})", flag.to_string(), if condition_met { "yes" } else { "no" })
    }

    fn diss_jp(&self, flag: JumpFlags) -> String {
        let address = self.bus.mem_read16(self.pc);

        let condition_met = match flag {
            JumpFlags::C => self.f.contains(FlagRegister::CARRY),
            JumpFlags::NoFlag => true,
            JumpFlags::NZ => !self.f.contains(FlagRegister::ZERO),
            JumpFlags::Z => self.f.contains(FlagRegister::ZERO),
            JumpFlags::NC => !self.f.contains(FlagRegister::CARRY),
        };

        format!("JP {} 0x{:x} (branch: {})", flag.to_string(), address, if condition_met { "yes" } else { "no" })
    }

    fn diss_ld_immediate(&self, reg1: Register, load_type: LoadType) -> String {
        match load_type {
            LoadType::Normal => {
                let immediate = if reg1 as usize > 6 {
                    self.bus.mem_read16(self.pc)
                } else {
                    self.bus.mem_read8(self.pc) as u16
                };

                format!("LD {:?}, 0x{:x}", reg1, immediate)
            }
            LoadType::LeftPointer => {
                let immediate = self.bus.mem_read16(self.pc);

                format!("LD (0x{:x}), {:?}", immediate, reg1)
            }
            LoadType::RightPointer => {
                let immediate = self.bus.mem_read16(self.pc);

                format!("LD {:?}, (0x{:x})", reg1, immediate)
            }
        }
    }

    fn diss_add(&self, r1: Register, r2: Register) -> String {
        format!("ADD {:?}, {:?}", r1, r2)
    }

    fn diss_alu(&self, op: &str, r1: Option<Register>) -> String {
        let operand = if let Some(register) = r1 {
            format!("{:?}", register)
        } else {
            format!("0x{:x}", self.bus.mem_read8(self.pc))
        };

        format!("{op} A, {operand}")
    }
    fn diss_ld_registers(&self, reg1: Register, reg2: Register, load_type: LoadType) -> String {
        match load_type {
            LoadType::LeftPointer => format!("LD ({:?}), {:?}", reg1, reg2),
            LoadType::Normal => format!("LD {:?}, {:?}", reg1, reg2),
            LoadType::RightPointer => format!("LD {:?} ({:?})", reg1, reg2)
        }
    }

    fn diss_store_hl_ptr(&self, reg1: Register, inc_mode: IncrementMode) -> String {
        let sign = match inc_mode {
            IncrementMode::Decrement => "-",
            IncrementMode::Increment => "+"
        };

        format!("LD (HL{sign}), {:?}", reg1)
    }

    fn diss_load_hl_ptr(&self, reg1: Register, inc_mode: IncrementMode) -> String {
        let sign = match inc_mode {
            IncrementMode::Decrement => "-",
            IncrementMode::Increment => "+"
        };

        format!("LD {:?}, (HL{sign})", reg1)
    }

    fn diss_inc(&self, reg1: Register) -> String {
        format!("INC {:?}", reg1)
    }

    fn diss_dec(&self, reg1: Register) -> String {
        format!("DEC {:?}", reg1)
    }

    fn diss_ld_upper(&self, r1: Register, load_type: LoadType, use_c: bool) -> String {
        let mut immediate = if use_c {
            self.registers[Register::C as usize] as u16
        } else {
            self.bus.mem_read8(self.pc) as u16
        };

        immediate += 0xff00;

        match load_type {
            LoadType::LeftPointer => format!("LD (0x{:x}), {:?}", immediate, r1),
            LoadType::RightPointer => format!("LD {:?}, (0x{:x})", r1, immediate),
            _ => panic!("invalid option given to diss_ld_upper: {:?}", load_type)
        }
    }

    fn diss_ret(&self, cond: JumpFlags) -> String {
        format!("RET {}", cond.to_string())
    }

    fn diss_add_sp(&self) -> String {
        let displacement = self.bus.mem_read8(self.pc) as i8;

        let operand = if displacement < 0 {
            format!("-0x{:x}", displacement)
        } else {
            format!("0x{:x}", displacement)
        };

        format!("ADD SP, {operand}")
    }

    fn diss_ld_hl_displacement(&self) -> String {
        let displacement = self.bus.mem_read8(self.pc) as i8;

        let operand = if displacement < 0 {
            format!("-0x{:x}", displacement)
        } else {
            format!("0x{:x}", displacement)
        };

        format!("LD HL, SP + {operand}")
    }

    fn diss_call(&self, cond: JumpFlags) -> String {
        let address = self.bus.mem_read16(self.pc);
        format!("CALL {} 0x{:x}", cond.to_string(), address)
    }

    fn diss_bit(&self, bit: u8, r1: Register) -> String {
        format!("BIT {bit}, {:?}", r1)
    }

    fn diss_res(&self, bit: u8, r1: Register) -> String {
        format!("RES {bit}, {:?}", r1)
    }

    fn diss_set(&self, bit: u8, r1: Register) -> String {
        format!("SET {bit}, {:?}", r1)
    }

    fn diss_rl(&self, r1: Register) -> String {
        format!("RL {:?}", r1)
    }

    fn diss_srl(&self, r1: Register) -> String {
        format!("SRL {:?}", r1)
    }

    fn diss_sra(&self, r1: Register) -> String {
        format!("SRA {:?}", r1)
    }

    fn diss_rlc(&self, r1: Register) -> String {
        format!("RLC {:?}", r1)
    }

    fn diss_rrc(&self, r1: Register) -> String {
       format!("RRC {:?}", r1)
    }

    fn diss_sla(&self, r1: Register) -> String {
       format!("SLA {:?}", r1)
    }

    fn diss_rr(&self, r1: Register) -> String {
        format!("RR {:?}", r1)
    }

    fn diss_swap(&self, r1: Register) -> String {
        format!("SWAP {:?}", r1)
    }

    pub fn disassemble(&self, instruction: u8) -> String {
        let z = instruction & 0x7;
        let q = (instruction >> 3) & 1;
        let p = (instruction >> 4) & 0x3;
        let y = (instruction >> 3) & 0x7;
        let x = (instruction >> 6) & 0x3;

        match x {
            0 => match z {
                0 => {
                    match y {
                        0 => "NOP".to_string(),
                        1 => self.diss_ld_immediate_sp(),
                        2 => "STOP".to_string(),
                        3 => self.diss_jr(JumpFlags::NoFlag),
                        4..=7 => self.diss_jr(JumpFlags::new(y - 4)),
                        _ => unreachable!()
                    }
                }
                1 => {
                    match q {
                        0 => self.diss_ld_immediate(RP_TABLE[p as usize], LoadType::Normal),
                        1 => self.diss_add(Register::HL, RP_TABLE[p as usize]),
                        _ => unreachable!()
                    }
                }
                2 => {
                    match q {
                        0 => match p {
                            0 => self.diss_ld_registers(Register::BC, Register::A, LoadType::LeftPointer),
                            1 => self.diss_ld_registers(Register::DE, Register::A, LoadType::LeftPointer),
                            2 => self.diss_store_hl_ptr(Register::A, IncrementMode::Increment),
                            3 => self.diss_store_hl_ptr(Register::A, IncrementMode::Decrement),
                            _ => unreachable!()
                        }
                        1 => match p {
                            0 => self.diss_ld_registers(Register::A, Register::BC, LoadType::RightPointer),
                            1 => self.diss_ld_registers(Register::A, Register::DE, LoadType::RightPointer),
                            2 => self.diss_load_hl_ptr(Register::A, IncrementMode::Increment),
                            3 => self.diss_load_hl_ptr(Register::A, IncrementMode::Decrement),
                            _ => unreachable!()
                        }
                        _ => unreachable!()
                    }
                }
                3 => {
                    match q {
                        0 => self.diss_inc(RP_TABLE[p as usize]),
                        1 => self.diss_dec(RP_TABLE[p as usize]),
                        _ => unreachable!()
                    }
                }
                4 => self.diss_inc(R_TABLE[y as usize]),
                5 => self.diss_dec(R_TABLE[y as usize]),
                6 => self.diss_ld_immediate(R_TABLE[y as usize], LoadType::Normal),
                7 => match y {
                    0 => "RLCA".to_string(),
                    1 => "RRCA".to_string(),
                    2 => "RLA".to_string(),
                    3 => "RRA".to_string(),
                    4 => "DAA".to_string(),
                    5 => "CPL".to_string(),
                    6 => "SCF".to_string(),
                    7 => "CCF".to_string(),
                    _ => unreachable!()
                }
                _ => unreachable!()
            }
            1 => {
                if z == 6 && y == 6 {
                    "HALT".to_string()
                } else {
                    self.diss_ld_registers(R_TABLE[y as usize], R_TABLE[z as usize], LoadType::Normal)
                }
            }
            2 => {
                match ALU_TABLE[y as usize] {
                    AluOp::ADD => self.diss_alu("ADD", Some(R_TABLE[z as usize])),
                    AluOp::ADC => self.diss_alu("ADC", Some(R_TABLE[z as usize])),
                    AluOp::SUB => self.diss_alu("SUB", Some(R_TABLE[z as usize])),
                    AluOp::SBC => self.diss_alu("SBC", Some(R_TABLE[z as usize])),
                    AluOp::AND => self.diss_alu("AND", Some(R_TABLE[z as usize])),
                    AluOp::CP => self.diss_alu("CP", Some(R_TABLE[z as usize])),
                    AluOp::OR => self.diss_alu("OR", Some(R_TABLE[z as usize])),
                    AluOp::XOR => self.diss_alu("XOR", Some(R_TABLE[z as usize]))
                }
            }
            3 => {
                match z {
                    0 => match y {
                        0..=3 => self.diss_ret(CC_TABLE[y as usize]),
                        4 => self.diss_ld_upper(Register::A, LoadType::LeftPointer, false),
                        5 => self.diss_add_sp(),
                        6 => self.diss_ld_upper(Register::A, LoadType::RightPointer, false),
                        7 => self.diss_ld_hl_displacement(),
                        _ => unreachable!()
                    }
                    1 => match q {
                        0 => format!("POP {:?}", RP2_TABLE[p as usize]),
                        1 => match p {
                            0 => self.diss_ret(JumpFlags::NoFlag),
                            1 => "RETI".to_string(),
                            2 => "JP HL".to_string(),
                            3 => self.diss_ld_registers(Register::SP, Register::HL, LoadType::Normal),
                            _ => unreachable!()
                        }
                        _ => unreachable!()
                    }
                    2 => match y {
                        0..=3 => self.diss_jp(CC_TABLE[y as usize]),
                        4 => self.diss_ld_upper(Register::A, LoadType::LeftPointer, true),
                        5 => self.diss_ld_immediate(Register::A, LoadType::LeftPointer),
                        6 => self.diss_ld_upper(Register::A, LoadType::RightPointer, true),
                        7 => self.diss_ld_immediate(Register::A, LoadType::RightPointer),
                        _ => unreachable!()
                    }
                    3 => match y {
                        0 => self.diss_jp(JumpFlags::NoFlag),
                        1 => {
                            let cb_opcode = self.bus.mem_read8(self.pc);

                            self.disassemble_cb(cb_opcode)
                        }
                        6 => "DI".to_string(),
                        7 => "EI".to_string(),
                        _ => panic!("invalid parameter for y given: {y}")
                    }
                    4 => match y {
                        0..=3 => self.diss_call(CC_TABLE[y as usize]),
                        _ => panic!("invalid option for y given: {y}")
                    }
                    5 => match q {
                        0 => format!("PUSH {:?}", RP2_TABLE[p as usize]),
                        1 => {
                            if p == 0 {
                                self.diss_call(JumpFlags::NoFlag)
                            } else {
                                panic!("invalid option for p given: {p}");
                            }
                        }
                        _ => unreachable!()
                    }
                    6 => {
                        match ALU_TABLE[y as usize] {
                            AluOp::ADD => self.diss_alu("ADD", None),
                            AluOp::ADC => self.diss_alu("ADC", None),
                            AluOp::SUB => self.diss_alu("SUB", None),
                            AluOp::SBC => self.diss_alu("SBC", None),
                            AluOp::AND => self.diss_alu("AND", None),
                            AluOp::CP => self.diss_alu("CP", None),
                            AluOp::OR => self.diss_alu("OR", None),
                            AluOp::XOR => self.diss_alu("XOR", None)
                        }
                    }
                    7 => format!("RST 0x{:x}", y * 8),
                    _ => unreachable!()
                }
            }
            _ => unreachable!()
        }
    }

    fn disassemble_cb(&self, instruction: u8) -> String {
        let z = instruction & 0x7;
        let y = (instruction >> 3) & 0x7;
        let x = (instruction >> 6) & 0x3;

        match x {
            0 => match CBOp::new(y) {
                CBOp::RL => self.diss_rl(R_TABLE[z as usize]),
                CBOp::RLC => self.diss_rlc(R_TABLE[z as usize]),
                CBOp::RR => self.diss_rr(R_TABLE[z as usize]),
                CBOp::RRC => self.diss_rrc(R_TABLE[z as usize]),
                CBOp::SLA => self.diss_sla(R_TABLE[z as usize]),
                CBOp::SRA => self.diss_sra(R_TABLE[z as usize]),
                CBOp::SRL => self.diss_srl(R_TABLE[z as usize]),
                CBOp::SWAP => self.diss_swap(R_TABLE[z as usize])
            }
            1 => self.diss_bit(y, R_TABLE[z as usize]),
            2 => self.diss_res(y, R_TABLE[z as usize]),
            3 => self.diss_set(y, R_TABLE[z as usize]),
            _ => unreachable!()
        }
    }
}