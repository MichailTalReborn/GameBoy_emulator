const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

#[derive(Debug)]
pub struct CPU {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub bus: MemoryBus,
}

#[derive(Debug)]
pub struct MemoryBus {
    memory: [u8; 0xFFFF + 1],
}

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            memory: [0; 0xFFFF + 1],
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        (hi << 8) | lo
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub f: FlagsRegister,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

impl FlagsRegister {
    fn new() -> Self {
        FlagsRegister {
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        }
    }
}

impl From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero { 1 } else { 0 }) << 7
            | (if flag.subtract { 1 } else { 0 }) << 6
            | (if flag.half_carry { 1 } else { 0 }) << 5
            | (if flag.carry { 1 } else { 0 }) << 4
    }
}

impl From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        FlagsRegister {
            zero: (byte >> 7) & 1 != 0,
            subtract: (byte >> 6) & 1 != 0,
            half_carry: (byte >> 5) & 1 != 0,
            carry: (byte >> 4) & 1 != 0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Clone, Copy)]
pub enum PrefixTarget {
    B,
    C,
    D,
    E,
    H,
    L,
    A,
    HLInd,
}

#[derive(Clone, Copy)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

pub enum Instruction {
    ADD(ArithmeticTarget),
    ADC(ArithmeticTarget),
    SUB(ArithmeticTarget),
    SBC(ArithmeticTarget),
    AND(ArithmeticTarget),
    OR(ArithmeticTarget),
    XOR(ArithmeticTarget),
    CP(ArithmeticTarget),
    INC(ArithmeticTarget),
    DEC(ArithmeticTarget),
    CCF,
    SCF,
    RRA,
    RLA,
    RRCA,
    RLCA,
    CPL,
    BIT(u8, PrefixTarget),
    RES(u8, PrefixTarget),
    SET(u8, PrefixTarget),
    SRL(PrefixTarget),
    RR(PrefixTarget),
    RL(PrefixTarget),
    RRC(PrefixTarget),
    RLC(PrefixTarget),
    SRA(PrefixTarget),
    SLA(PrefixTarget),
    SWAP(PrefixTarget),
    JP(JumpTest),
    NOP,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: FlagsRegister::new(),
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (u8::from(self.f) as u16)
    }
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            pc: 0,
            sp: 0,
            bus: MemoryBus::new(),
        }
    }

    pub fn step(&mut self) {
        let opcode = self.read_next_byte();
        let prefixed = opcode == 0xCB;

        let instruction = if prefixed {
            let cb_opcode = self.read_next_byte();
            Instruction::from_byte(cb_opcode, true)
        } else {
            Instruction::from_byte(opcode, false)
        };

        let instruction = match instruction {
            Some(i) => i,
            None => {
                panic!(
                    "Unrecognized opcode: {:#04x} at PC={:#06x}",
                    if prefixed {
                        0xCB00 | (self.pc as u16 - 1)
                    } else {
                        opcode as u16
                    },
                    self.pc - if prefixed { 2 } else { 1 }
                )
            }
        };

        self.execute(instruction);
    }

    fn read_next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn read_next_word(&mut self) -> u16 {
        let word = self.bus.read_word(self.pc);
        self.pc += 2;
        word
    }

    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::NOP => {}
            Instruction::ADD(target) => {
                let value = self.read_target(target);
                self.registers.a = self.add(value, false);
            }
            Instruction::ADC(target) => {
                let value = self.read_target(target);
                self.registers.a = self.add(value, self.registers.f.carry);
            }
            Instruction::SUB(target) => {
                let value = self.read_target(target);
                self.registers.a = self.sub(value, false);
            }
            Instruction::SBC(target) => {
                let value = self.read_target(target);
                self.registers.a = self.sub(value, self.registers.f.carry);
            }
            Instruction::AND(target) => {
                let value = self.read_target(target);
                self.registers.a &= value;
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
                self.registers.f.carry = false;
            }
            Instruction::OR(target) => {
                let value = self.read_target(target);
                self.registers.a |= value;
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::XOR(target) => {
                let value = self.read_target(target);
                self.registers.a ^= value;
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::CP(target) => {
                let value = self.read_target(target);
                self.cp(value);
            }
            Instruction::INC(target) => {
                let value = self.read_target(target);
                let result = self.inc(value);
                self.write_target(target, result);
            }
            Instruction::DEC(target) => {
                let value = self.read_target(target);
                let result = self.dec(value);
                self.write_target(target, result);
            }
            Instruction::CCF => {
                self.registers.f.carry = !self.registers.f.carry;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
            }
            Instruction::SCF => {
                self.registers.f.carry = true;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
            }
            Instruction::RRA => self.rra(),
            Instruction::RLA => self.rla(),
            Instruction::RRCA => self.rrca(),
            Instruction::RLCA => self.rlca(),
            Instruction::CPL => {
                self.registers.a = !self.registers.a;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;
            }
            Instruction::BIT(bit, target) => {
                let value = self.read_prefix_target(target);
                self.registers.f.zero = (value & (1 << bit)) == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
            }
            Instruction::RES(bit, target) => {
                let value = self.read_prefix_target(target);
                let result = value & !(1 << bit);
                self.write_prefix_target(target, result);
            }
            Instruction::SET(bit, target) => {
                let value = self.read_prefix_target(target);
                let result = value | (1 << bit);
                self.write_prefix_target(target, result);
            }
            Instruction::SRL(target) => {
                let value = self.read_prefix_target(target);
                let result = self.srl(value);
                self.write_prefix_target(target, result);
            }
            Instruction::RR(target) => {
                let value = self.read_prefix_target(target);
                let result = self.rr(value);
                self.write_prefix_target(target, result);
            }
            Instruction::RL(target) => {
                let value = self.read_prefix_target(target);
                let result = self.rl(value);
                self.write_prefix_target(target, result);
            }
            Instruction::RRC(target) => {
                let value = self.read_prefix_target(target);
                let result = self.rrc(value);
                self.write_prefix_target(target, result);
            }
            Instruction::RLC(target) => {
                let value = self.read_prefix_target(target);
                let result = self.rlc(value);
                self.write_prefix_target(target, result);
            }
            Instruction::SRA(target) => {
                let value = self.read_prefix_target(target);
                let result = self.sra(value);
                self.write_prefix_target(target, result);
            }
            Instruction::SLA(target) => {
                let value = self.read_prefix_target(target);
                let result = self.sla(value);
                self.write_prefix_target(target, result);
            }
            Instruction::SWAP(target) => {
                let value = self.read_prefix_target(target);
                let result = self.swap(value);
                self.write_prefix_target(target, result);
            }
            Instruction::JP(test) => {
                let should_jump = match test {
                    JumpTest::NotZero => !self.registers.f.zero,
                    JumpTest::Zero => self.registers.f.zero,
                    JumpTest::NotCarry => !self.registers.f.carry,
                    JumpTest::Carry => self.registers.f.carry,
                    JumpTest::Always => true,
                };
                if should_jump {
                    self.pc = self.read_next_word();
                }
                // else: PC already advanced by 3 bytes (opcode + 2-byte addr)
            }
        }
    }

    // Helper: read 8-bit register or (HL)
    fn read_target(&self, target: ArithmeticTarget) -> u8 {
        match target {
            ArithmeticTarget::A => self.registers.a,
            ArithmeticTarget::B => self.registers.b,
            ArithmeticTarget::C => self.registers.c,
            ArithmeticTarget::D => self.registers.d,
            ArithmeticTarget::E => self.registers.e,
            ArithmeticTarget::H => self.registers.h,
            ArithmeticTarget::L => self.registers.l,
        }
    }

    fn write_target(&mut self, target: ArithmeticTarget, value: u8) {
        match target {
            ArithmeticTarget::A => self.registers.a = value,
            ArithmeticTarget::B => self.registers.b = value,
            ArithmeticTarget::C => self.registers.c = value,
            ArithmeticTarget::D => self.registers.d = value,
            ArithmeticTarget::E => self.registers.e = value,
            ArithmeticTarget::H => self.registers.h = value,
            ArithmeticTarget::L => self.registers.l = value,
        }
    }

    fn read_prefix_target(&self, target: PrefixTarget) -> u8 {
        match target {
            PrefixTarget::A => self.registers.a,
            PrefixTarget::B => self.registers.b,
            PrefixTarget::C => self.registers.c,
            PrefixTarget::D => self.registers.d,
            PrefixTarget::E => self.registers.e,
            PrefixTarget::H => self.registers.h,
            PrefixTarget::L => self.registers.l,
            PrefixTarget::HLInd => self.bus.read_byte(self.registers.get_hl()),
        }
    }

    fn write_prefix_target(&mut self, target: PrefixTarget, value: u8) {
        match target {
            PrefixTarget::A => self.registers.a = value,
            PrefixTarget::B => self.registers.b = value,
            PrefixTarget::C => self.registers.c = value,
            PrefixTarget::D => self.registers.d = value,
            PrefixTarget::E => self.registers.e = value,
            PrefixTarget::H => self.registers.h = value,
            PrefixTarget::L => self.registers.l = value,
            PrefixTarget::HLInd => self.bus.write_byte(self.registers.get_hl(), value),
        }
    }

    // Arithmetic helpers
    fn add(&mut self, value: u8, with_carry: bool) -> u8 {
        let carry = if with_carry {
            self.registers.f.carry as u8
        } else {
            0
        };
        let a = self.registers.a;
        let result = a.wrapping_add(value).wrapping_add(carry);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (a & 0xF) + (value & 0xF) + carry > 0xF;
        self.registers.f.carry = (a as u16) + (value as u16) + (carry as u16) > 0xFF;

        result
    }

    fn sub(&mut self, value: u8, with_carry: bool) -> u8 {
        let carry = if with_carry {
            self.registers.f.carry as u8
        } else {
            0
        };
        let a = self.registers.a;
        let result = a.wrapping_sub(value).wrapping_sub(carry);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF) + carry;
        self.registers.f.carry = (a as u16) < (value as u16) + (carry as u16);

        result
    }

    fn cp(&mut self, value: u8) {
        let a = self.registers.a;
        self.registers.f.zero = a == value;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF);
        self.registers.f.carry = a < value;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0xF) + 1 > 0xF;
        result
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = value & 0xF == 0;
        result
    }

    // Rotates & shifts
    fn rra(&mut self) {
        let carry = self.registers.f.carry as u8;
        let new_carry = self.registers.a & 1;
        self.registers.a = (self.registers.a >> 1) | (carry << 7);
        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry != 0;
    }

    fn rla(&mut self) {
        let carry = self.registers.f.carry as u8;
        let new_carry = (self.registers.a >> 7) & 1;
        self.registers.a = (self.registers.a << 1) | carry;
        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry != 0;
    }

    fn rrca(&mut self) {
        let bit0 = self.registers.a & 1;
        self.registers.a = (self.registers.a >> 1) | (bit0 << 7);
        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = bit0 != 0;
    }

    fn rlca(&mut self) {
        let bit7 = (self.registers.a >> 7) & 1;
        self.registers.a = (self.registers.a << 1) | bit7;
        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = bit7 != 0;
    }

    fn srl(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = value >> 1;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry != 0;
        result
    }

    fn rr(&mut self, value: u8) -> u8 {
        let carry_in = self.registers.f.carry as u8;
        let carry_out = value & 1;
        let result = (value >> 1) | (carry_in << 7);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry_out != 0;
        result
    }

    fn rl(&mut self, value: u8) -> u8 {
        let carry_in = self.registers.f.carry as u8;
        let carry_out = (value >> 7) & 1;
        let result = (value << 1) | carry_in;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry_out != 0;
        result
    }

    fn rrc(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = (value >> 1) | (carry << 7);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry != 0;
        result
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) & 1;
        let result = (value << 1) | carry;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry != 0;
        result
    }

    fn sra(&mut self, value: u8) -> u8 {
        let carry = value & 1;
        let result = (value >> 1) | (value & 0x80);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry != 0;
        result
    }

    fn sla(&mut self, value: u8) -> u8 {
        let carry = (value >> 7) & 1;
        let result = value << 1;
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry != 0;
        result
    }

    fn swap(&mut self, value: u8) -> u8 {
        let result = value.rotate_left(4);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;
        result
    }
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Self> {
        if prefixed {
            Self::from_byte_prefixed(byte)
        } else {
            Self::from_byte_non_prefixed(byte)
        }
    }

    fn from_byte_non_prefixed(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Instruction::NOP),
            0x3E => Some(Instruction::ADD(ArithmeticTarget::A)),
            0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),
            0xC3 => Some(Instruction::JP(JumpTest::Always)),
            0xFE => Some(Instruction::CP(ArithmeticTarget::A)),
            0xAF => Some(Instruction::XOR(ArithmeticTarget::A)),
            0x37 => Some(Instruction::SCF),
            0x3F => Some(Instruction::CCF),
            0x17 => Some(Instruction::RLA),
            0x1F => Some(Instruction::RRA),
            0x0F => Some(Instruction::RRCA),
            0x07 => Some(Instruction::RLCA),
            0x2F => Some(Instruction::CPL),
            _ => None,
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Self> {
        let reg = match byte & 0b00000111 {
            0 => PrefixTarget::B,
            1 => PrefixTarget::C,
            2 => PrefixTarget::D,
            3 => PrefixTarget::E,
            4 => PrefixTarget::H,
            5 => PrefixTarget::L,
            6 => PrefixTarget::HLInd,
            7 => PrefixTarget::A,
            _ => unreachable!(),
        };

        match byte {
            0x00..=0x07 => Some(Instruction::RLC(reg)),
            0x08..=0x0F => Some(Instruction::RRC(reg)),
            0x10..=0x17 => Some(Instruction::RL(reg)),
            0x18..=0x1F => Some(Instruction::RR(reg)),
            0x20..=0x27 => Some(Instruction::SLA(reg)),
            0x28..=0x2F => Some(Instruction::SRA(reg)),
            0x30..=0x37 => Some(Instruction::SWAP(reg)),
            0x38..=0x3F => Some(Instruction::SRL(reg)),
            0x40..=0x7F => {
                let bit = ((byte >> 3) & 0b111) as u8;
                Some(Instruction::BIT(bit, reg))
            }
            0x80..=0xBF => {
                let bit = ((byte >> 3) & 0b111) as u8;
                Some(Instruction::RES(bit, reg))
            }
            0xC0..=0xFF => {
                let bit = ((byte >> 3) & 0b111) as u8;
                Some(Instruction::SET(bit, reg))
            }
            _ => None,
        }
    }
}
