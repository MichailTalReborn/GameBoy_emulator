const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBSTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

#[derive(Debug)]
pub struct CPU {
    pub registers: Registers,
}

pub enum Instruction {
    ADD(ArithmeticTarget),
    SUB(ArithmeticTarget),
    ADDHL(WordTarget),
    ADC(ArithmeticTarget),
    SBC(ArithmeticTarget),
    AND(ArithmeticTarget),
    OR(ArithmeticTarget),
    XOR(ArithmeticTarget),
    CP(ArithmeticTarget),
    INC(ArithmeticTarget),
    DEC(ArithmeticTarget),
    CCF(),
    SCF(),
    RRA(),
    RLA(),
    RRCA(),
    RRLA(),
    CPL(),
    BIT(u8, ArithmeticTarget),
    RESET(u8, ArithmeticTarget),
    SET(u8, ArithmeticTarget),
    SRL(ArithmeticTarget),
    RR(ArithmeticTarget),
    RL(ArithmeticTarget),
    RRC(ArithmeticTarget),
    RLC(ArithmeticTarget),
    SRA(ArithmeticTarget),
    SLA(ArithmeticTarget),
    SWAP(ArithmeticTarget),
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

enum WordTarget {
    AF,
    BC,
    HL,
    DE,
}

#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: FlagsRegister,
    h: u8,
    l: u8,
}

#[derive(Debug, Clone, Copy)]
struct FlagsRegister {
    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
        }
    }
}

impl Registers {
    fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister {
                zero: false,
                subtract: false,
                half_carry: false,
                carry: false,
            },
            h: 0,
            l: 0,
        }
    }

    fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (u8::from(self.f) as u16)
    }

    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;

        // IMPORTANT: lower nibble must always be 0
        self.f = FlagsRegister::from((value & 0x00FF) as u8 & 0xF0);
    }
    fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn get(&self, target: ArithmeticTarget) -> u8 {
        match target {
            ArithmeticTarget::A => self.a,
            ArithmeticTarget::B => self.b,
            ArithmeticTarget::C => self.c,
            ArithmeticTarget::D => self.d,
            ArithmeticTarget::E => self.e,
            ArithmeticTarget::H => self.h,
            ArithmeticTarget::L => self.l,
        }
    }

    fn set(&mut self, target: ArithmeticTarget, value: u8) {
        match target {
            ArithmeticTarget::A => self.a = value,
            ArithmeticTarget::B => self.b = value,
            ArithmeticTarget::C => self.c = value,
            ArithmeticTarget::D => self.d = value,
            ArithmeticTarget::E => self.e = value,
            ArithmeticTarget::H => self.h = value,
            ArithmeticTarget::L => self.l = value,
        }
    }

    fn get_word(&self, target: WordTarget) -> u16 {
        match target {
            WordTarget::BC => self.get_bc(),
            WordTarget::HL => self.get_hl(),
            WordTarget::DE => self.get_de(),
            WordTarget::AF => self.get_af(),
        }
    }
}

impl std::convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flag.subtract { 1 } else { 0 }) << SUBSTRACT_FLAG_BYTE_POSITION
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flag.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl std::convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> SUBSTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

impl CPU {
    pub fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::ADD(target) => {
                let value = self.registers.get(target);
                self.registers.a = self.add(value);
            }
            Instruction::SUB(target) => {
                let value = self.registers.get(target);
                self.registers.a = self.sub(value);
            }
            Instruction::ADDHL(target) => {
                let value = self.registers.get_word(target);
                self.add_hl(value);
            }
            Instruction::ADC(target) => {
                let value = self.registers.get(target);
                self.registers.a = self.addc(value);
            }
            Instruction::SBC(target) => {
                let value = self.registers.get(target);
                self.registers.a = self.subc(value);
            }
            Instruction::AND(target) => {
                let value = self.registers.get(target);
                self.registers.a = self.and(value);
            }
            Instruction::OR(target) => {
                let value = self.registers.get(target);
                self.registers.a = self.or(value);
            }
            Instruction::XOR(target) => {
                let value = self.registers.get(target);
                self.registers.a = self.xor(value);
            }
            Instruction::CP(target) => {
                let value = self.registers.get(target);
                self.cp(value);
            }
            Instruction::INC(target) => {
                let value = self.registers.get(target);
                let result = self.inc(value);
                self.registers.set(target, result);
            }
            Instruction::DEC(target) => {
                let value = self.registers.get(target);
                let result = self.dec(value);
                self.registers.set(target, result);
            }
            Instruction::CCF() => self.ccf(),

            Instruction::SCF() => self.scf(),

            Instruction::RRA() => self.rra(),

            Instruction::RLA() => self.rla(),

            Instruction::RRCA() => self.rrca(),

            Instruction::RRLA() => self.rrla(),

            Instruction::CPL() => self.cpl(),

            Instruction::BIT(bit, target) => {
                let value = self.registers.get(target);
                self.bit(bit, value);
            }

            Instruction::RESET(bit, target) => {
                let value = self.registers.get(target);
                let result = self.reset(bit, value);
                self.registers.set(target, result);
            }

            Instruction::SET(bit, target) => {
                let value = self.registers.get(target);
                let result = self.set(bit, value);
                self.registers.set(target, result);
            }

            Instruction::SRL(target) => {
                let value = self.registers.get(target);
                let result = self.srl(value);

                self.registers.set(target, result);
            }

            Instruction::RR(target) => {
                let value = self.registers.get(target);
                let result = self.rr(value);

                self.registers.set(target, result);
            }

            Instruction::RL(target) => {
                let value = self.registers.get(target);
                let result = self.rl(value);

                self.registers.set(target, result);
            }

            Instruction::RRC(target) => {
                let value = self.registers.get(target);
                let result = self.rrc(value);

                self.registers.set(target, result);
            }

            Instruction::RLC(target) => {
                let value = self.registers.get(target);
                let result = self.rlc(value);

                self.registers.set(target, result);
            }

            Instruction::SRA(target) => {
                let value = self.registers.get(target);
                let result = self.sra(value);

                self.registers.set(target, result);
            }

            Instruction::SLA(target) => {
                let value = self.registers.get(target);
                let result = self.sla(value);

                self.registers.set(target, result);
            }

            Instruction::SWAP(target) => {
                let value = self.registers.get(target);
                let result = self.swap(value);

                self.registers.set(target, result);
            }
        }
    }
    fn add(&mut self, value: u8) -> u8 {
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    fn addc(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let carry = self.registers.f.carry as u8;

        let result16 = a as u16 + value as u16 + carry as u16;
        let result = result16 as u8;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = result16 > 0xFF;
        self.registers.f.half_carry = ((a & 0xF) + (value & 0xF) + carry) > 0xF;

        result
    }

    fn sub(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let (result, borrow) = a.overflowing_sub(value);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = borrow;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF);

        result
    }

    fn subc(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let carry = self.registers.f.carry as u8;

        let result = a.wrapping_sub(value).wrapping_sub(carry);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;

        self.registers.f.carry = (a as u16) < (value as u16 + carry as u16);

        self.registers.f.half_carry = (a & 0xF) < ((value & 0xF) + carry);

        result
    }

    fn add_hl(&mut self, value: u16) {
        let hl = self.registers.get_hl();
        let result = hl.wrapping_add(value);

        self.registers.f.subtract = false;
        self.registers.f.half_carry = ((hl & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF;
        self.registers.f.carry = hl.wrapping_add(value) < hl;

        self.registers.set_hl(result);
    }

    fn and(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let result = a & value;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = true;
        self.registers.f.carry = false;

        result
    }

    fn or(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let result = a | value;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;

        result
    }

    fn xor(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let result = a ^ value;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;

        result
    }

    fn cp(&mut self, value: u8) {
        let a = self.registers.a;
        let (result, borrow) = a.overflowing_sub(value);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = borrow;
        self.registers.f.half_carry = (a & 0xF) < (value & 0xF);
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (value & 0x0F) == 0x0F;

        result
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = true;
        self.registers.f.half_carry = (value & 0x0F) == 0x0F;

        result
    }

    fn ccf(&mut self) {
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = !self.registers.f.carry;
    }

    fn scf(&mut self) {
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = true;
    }

    fn rra(&mut self) {
        let a = self.registers.a;
        let old_carry = self.registers.f.carry;
        let new_carry = (a & 0x01) != 0;
        let mut result = a >> 1;

        if old_carry {
            result |= 0x80;
        }

        self.registers.a = result;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry;
    }

    fn rla(&mut self) {
        let a = self.registers.a;
        let old_carry = self.registers.f.carry;
        let new_carry = (a & 0x80) != 0;
        let mut result = a << 1;

        if old_carry {
            result |= 0x01;
        }

        self.registers.a = result;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry;
    }

    fn rrca(&mut self) {
        let a = self.registers.a;
        let carry = (a & 0x01) != 0;
        let mut result = a >> 1;

        if carry {
            result |= 0x00;
        }

        self.registers.a = result;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    fn rrla(&mut self) {
        let a = self.registers.a;
        let carry = (a & 0x80) != 0;
        let mut result = a << 1;

        if carry {
            result |= 0x01;
        }

        self.registers.a = result;

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = carry;
    }

    fn cpl(&mut self) {
        self.registers.a = !self.registers.a;

        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }

    fn bit(&mut self, bit: u8, value: u8) {
        let bit_set = (value & (1 << bit)) != 0;

        self.registers.f.zero = !bit_set;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
    }

    fn reset(&mut self, bit: u8, value: u8) -> u8 {
        value & !(1 << bit)
    }

    fn set(&mut self, bit: u8, value: u8) -> u8 {
        value | (1 << bit)
    }

    fn srl(&mut self, value: u8) -> u8 {
        let result = value >> 1;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = (value & 0x01) != 0;

        result
    }

    fn rr(&mut self, value: u8) -> u8 {
        let old_carry = self.registers.f.carry;
        let new_carry = (value & 0x01) != 0;

        let mut result = value >> 1;

        if old_carry {
            result |= 0x80;
        }

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry;

        result
    }

    fn rl(&mut self, value: u8) -> u8 {
        let old_carry = self.registers.f.carry;
        let new_carry = (value & 0x80) != 0;

        let mut result = value << 1;

        if old_carry {
            result |= 0x01;
        }

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry;

        result
    }

    fn rrc(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x01) != 0;

        let mut result = value >> 1;

        if new_carry {
            result |= 0x80;
        }

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry;

        result
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let new_carry = (value & 0x80) != 0;

        let mut result = value << 1;

        if new_carry {
            result |= 0x01;
        }

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = new_carry;

        result
    }

    fn sra(&mut self, value: u8) -> u8 {
        let bit0 = value & 0x01;
        let bit7 = value & 0x80;

        let result = (value >> 1) | bit7;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = bit0 != 0;

        result
    }

    fn sla(&mut self, value: u8) -> u8 {
        let bit7 = value & 0x80;

        let result = value << 1;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = bit7 != 0;

        result
    }

    fn swap(&mut self, value: u8) -> u8 {
        let upper = value >> 4;
        let lower = value << 4;

        let result = upper | lower;

        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.carry = false;

        result
    }
}
