#[derive(Debug, Clone, Copy)]
pub struct FlagsRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
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

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | (u8::from(self.f) as u16)
    }
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        // Low nibble of F is always 0 on real hardware
        self.f = FlagsRegister::from((value & 0xF0) as u8);
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
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

#[derive(Clone, Copy)]
pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Clone, Copy)]
pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

#[derive(Clone, Copy)]
pub enum LoadByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,
    HLI,
}

#[derive(Clone, Copy)]
pub enum Indirect {
    BCIndirect,
    DEIndirect,
    HLIndirectMinus, // HL-, decrement HL after access
    HLIndirectPlus,  // HL+, increment HL after access
    WordIndirect,    // address given as next 2 bytes
}

#[derive(Clone, Copy)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
}

pub enum LoadType {
    /// 8-bit register / immediate / (HL) loads
    Byte(LoadByteTarget, LoadByteSource),
    /// 16-bit immediate loads:  LD rr, d16
    Word(LoadWordTarget),
    /// LD A, (rr)
    AFromIndirect(Indirect),
    /// LD (rr), A
    IndirectFromA(Indirect),
    /// LD A, (0xFF00 + n)  — n is the next byte
    AFromByteAddress,
    /// LD (0xFF00 + n), A  — n is the next byte
    ByteAddressFromA,
}

pub enum Instruction {
    // 8-bit arithmetic
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
    // Misc / flag ops
    CCF,
    SCF,
    RRA,
    RLA,
    RRCA,
    RLCA,
    CPL,
    // CB-prefixed bit ops
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
    // Control flow
    JP(JumpTest),
    CALL(JumpTest),
    RET(JumpTest),
    // Load / store
    LD(LoadType),
    // Stack
    PUSH(StackTarget),
    POP(StackTarget),
    // Misc
    NOP,
    HALT,
}

#[derive(Debug)]
pub struct CPU {
    pub registers: Registers,
    pub pc: u16,
    pub sp: u16,
    pub bus: MemoryBus,
    pub is_halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            pc: 0,
            sp: 0,
            bus: MemoryBus::new(),
            is_halted: false,
        }
    }

    pub fn step(&mut self) {
        if self.is_halted {
            return;
        }

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
            None => panic!(
                "Unrecognized opcode: {:#04x} at PC={:#06x}",
                opcode,
                self.pc.wrapping_sub(if prefixed { 2 } else { 1 })
            ),
        };

        self.execute(instruction);
    }

    fn read_next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn read_next_word(&mut self) -> u16 {
        let word = self.bus.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        word
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }

    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let msb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        (msb << 8) | lsb
    }

    fn call(&mut self, should_jump: bool) {
        let next_pc = self.pc.wrapping_add(2);
        if should_jump {
            self.push(next_pc);
            self.pc = self.read_next_word();
        } else {
            self.pc = next_pc;
        }
    }

    fn return_(&mut self, should_jump: bool) {
        if should_jump {
            self.pc = self.pop();
        }
        // else: pc already points past the RET opcode (1 byte, already consumed)
    }

    pub fn execute(&mut self, instruction: Instruction) {
        if self.is_halted {
            return;
        }

        match instruction {
            Instruction::NOP => {}

            Instruction::HALT => {
                self.is_halted = true;
            }

            Instruction::ADD(target) => {
                let value = self.read_arithmetic_target(target);
                self.registers.a = self.add(value, false);
            }
            Instruction::ADC(target) => {
                let value = self.read_arithmetic_target(target);
                self.registers.a = self.add(value, self.registers.f.carry);
            }
            Instruction::SUB(target) => {
                let value = self.read_arithmetic_target(target);
                self.registers.a = self.sub(value, false);
            }
            Instruction::SBC(target) => {
                let value = self.read_arithmetic_target(target);
                self.registers.a = self.sub(value, self.registers.f.carry);
            }
            Instruction::AND(target) => {
                let value = self.read_arithmetic_target(target);
                self.registers.a &= value;
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;
                self.registers.f.carry = false;
            }
            Instruction::OR(target) => {
                let value = self.read_arithmetic_target(target);
                self.registers.a |= value;
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::XOR(target) => {
                let value = self.read_arithmetic_target(target);
                self.registers.a ^= value;
                self.registers.f.zero = self.registers.a == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;
                self.registers.f.carry = false;
            }
            Instruction::CP(target) => {
                let value = self.read_arithmetic_target(target);
                self.cp(value);
            }
            Instruction::INC(target) => {
                let value = self.read_arithmetic_target(target);
                let result = self.inc(value);
                self.write_arithmetic_target(target, result);
            }
            Instruction::DEC(target) => {
                let value = self.read_arithmetic_target(target);
                let result = self.dec(value);
                self.write_arithmetic_target(target, result);
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
                self.write_prefix_target(target, value & !(1 << bit));
            }
            Instruction::SET(bit, target) => {
                let value = self.read_prefix_target(target);
                self.write_prefix_target(target, value | (1 << bit));
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
                let should_jump = self.evaluate_jump_test(test);
                if should_jump {
                    self.pc = self.read_next_word();
                } else {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            Instruction::CALL(test) => {
                let should_jump = self.evaluate_jump_test(test);
                self.call(should_jump);
            }
            Instruction::RET(test) => {
                let should_jump = self.evaluate_jump_test(test);
                self.return_(should_jump);
            }

            Instruction::PUSH(target) => {
                let value = match target {
                    StackTarget::AF => self.registers.get_af(),
                    StackTarget::BC => self.registers.get_bc(),
                    StackTarget::DE => self.registers.get_de(),
                    StackTarget::HL => self.registers.get_hl(),
                };
                self.push(value);
            }
            Instruction::POP(target) => {
                let value = self.pop();
                match target {
                    StackTarget::AF => self.registers.set_af(value),
                    StackTarget::BC => self.registers.set_bc(value),
                    StackTarget::DE => self.registers.set_de(value),
                    StackTarget::HL => self.registers.set_hl(value),
                }
            }

            Instruction::LD(load_type) => match load_type {
                LoadType::Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::A => self.registers.a,
                        LoadByteSource::B => self.registers.b,
                        LoadByteSource::C => self.registers.c,
                        LoadByteSource::D => self.registers.d,
                        LoadByteSource::E => self.registers.e,
                        LoadByteSource::H => self.registers.h,
                        LoadByteSource::L => self.registers.l,
                        LoadByteSource::D8 => self.read_next_byte(),
                        LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
                    };
                    match target {
                        LoadByteTarget::A => self.registers.a = source_value,
                        LoadByteTarget::B => self.registers.b = source_value,
                        LoadByteTarget::C => self.registers.c = source_value,
                        LoadByteTarget::D => self.registers.d = source_value,
                        LoadByteTarget::E => self.registers.e = source_value,
                        LoadByteTarget::H => self.registers.h = source_value,
                        LoadByteTarget::L => self.registers.l = source_value,
                        LoadByteTarget::HLI => {
                            self.bus.write_byte(self.registers.get_hl(), source_value)
                        }
                    }
                    // NOTE: pc was already advanced by read_next_byte() for
                    // D8 source, so we don't need to add anything extra here.
                }

                LoadType::Word(target) => {
                    let value = self.read_next_word();
                    match target {
                        LoadWordTarget::BC => self.registers.set_bc(value),
                        LoadWordTarget::DE => self.registers.set_de(value),
                        LoadWordTarget::HL => self.registers.set_hl(value),
                        LoadWordTarget::SP => self.sp = value,
                    }
                }

                // LD A, (rr)
                LoadType::AFromIndirect(indirect) => {
                    self.registers.a = match indirect {
                        Indirect::BCIndirect => self.bus.read_byte(self.registers.get_bc()),
                        Indirect::DEIndirect => self.bus.read_byte(self.registers.get_de()),
                        Indirect::HLIndirectPlus => {
                            let addr = self.registers.get_hl();
                            self.registers.set_hl(addr.wrapping_add(1));
                            self.bus.read_byte(addr)
                        }
                        Indirect::HLIndirectMinus => {
                            let addr = self.registers.get_hl();
                            self.registers.set_hl(addr.wrapping_sub(1));
                            self.bus.read_byte(addr)
                        }
                        Indirect::WordIndirect => {
                            let addr = self.read_next_word();
                            self.bus.read_byte(addr)
                        }
                    };
                }

                // LD (rr), A
                LoadType::IndirectFromA(indirect) => {
                    let a = self.registers.a;
                    match indirect {
                        Indirect::BCIndirect => {
                            let addr = self.registers.get_bc();
                            self.bus.write_byte(addr, a);
                        }
                        Indirect::DEIndirect => {
                            let addr = self.registers.get_de();
                            self.bus.write_byte(addr, a);
                        }
                        Indirect::HLIndirectPlus => {
                            let addr = self.registers.get_hl();
                            self.registers.set_hl(addr.wrapping_add(1));
                            self.bus.write_byte(addr, a);
                        }
                        Indirect::HLIndirectMinus => {
                            let addr = self.registers.get_hl();
                            self.registers.set_hl(addr.wrapping_sub(1));
                            self.bus.write_byte(addr, a);
                        }
                        Indirect::WordIndirect => {
                            let addr = self.read_next_word();
                            self.bus.write_byte(addr, a);
                        }
                    }
                }

                // LD A, (0xFF00 + n)
                LoadType::AFromByteAddress => {
                    let offset = self.read_next_byte() as u16;
                    self.registers.a = self.bus.read_byte(0xFF00 | offset);
                }

                // LD (0xFF00 + n), A
                LoadType::ByteAddressFromA => {
                    let offset = self.read_next_byte() as u16;
                    self.bus.write_byte(0xFF00 | offset, self.registers.a);
                }
            },
        }
    }

    fn evaluate_jump_test(&self, test: JumpTest) -> bool {
        match test {
            JumpTest::NotZero => !self.registers.f.zero,
            JumpTest::Zero => self.registers.f.zero,
            JumpTest::NotCarry => !self.registers.f.carry,
            JumpTest::Carry => self.registers.f.carry,
            JumpTest::Always => true,
        }
    }

    fn read_arithmetic_target(&self, target: ArithmeticTarget) -> u8 {
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

    fn write_arithmetic_target(&mut self, target: ArithmeticTarget, value: u8) {
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

    fn add(&mut self, value: u8, with_carry: bool) -> u8 {
        let carry = with_carry as u8;
        let a = self.registers.a;
        let result = a.wrapping_add(value).wrapping_add(carry);
        self.registers.f.zero = result == 0;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = (a & 0xF) + (value & 0xF) + carry > 0xF;
        self.registers.f.carry = (a as u16) + (value as u16) + (carry as u16) > 0xFF;
        result
    }

    fn sub(&mut self, value: u8, with_carry: bool) -> u8 {
        let carry = with_carry as u8;
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
        let result = (value >> 1) | (value & 0x80); // sign-extend
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
        let result = (value >> 4) | (value << 4); // swap nibbles
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
            // Misc
            0x00 => Some(Instruction::NOP),
            0x76 => Some(Instruction::HALT),

            // LD r, d8  (load immediate into register)
            0x06 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::D8,
            ))),
            0x0E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::D8,
            ))),
            0x16 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::D8,
            ))),
            0x1E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::D8,
            ))),
            0x26 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::D8,
            ))),
            0x2E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::D8,
            ))),
            0x3E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::D8,
            ))),

            // LD rr, d16
            0x01 => Some(Instruction::LD(LoadType::Word(LoadWordTarget::BC))),
            0x11 => Some(Instruction::LD(LoadType::Word(LoadWordTarget::DE))),
            0x21 => Some(Instruction::LD(LoadType::Word(LoadWordTarget::HL))),
            0x31 => Some(Instruction::LD(LoadType::Word(LoadWordTarget::SP))),

            // LD (rr), A
            0x02 => Some(Instruction::LD(LoadType::IndirectFromA(
                Indirect::BCIndirect,
            ))),
            0x12 => Some(Instruction::LD(LoadType::IndirectFromA(
                Indirect::DEIndirect,
            ))),
            0x22 => Some(Instruction::LD(LoadType::IndirectFromA(
                Indirect::HLIndirectPlus,
            ))),
            0x32 => Some(Instruction::LD(LoadType::IndirectFromA(
                Indirect::HLIndirectMinus,
            ))),

            // LD A, (rr)
            0x0A => Some(Instruction::LD(LoadType::AFromIndirect(
                Indirect::BCIndirect,
            ))),
            0x1A => Some(Instruction::LD(LoadType::AFromIndirect(
                Indirect::DEIndirect,
            ))),
            0x2A => Some(Instruction::LD(LoadType::AFromIndirect(
                Indirect::HLIndirectPlus,
            ))),
            0x3A => Some(Instruction::LD(LoadType::AFromIndirect(
                Indirect::HLIndirectMinus,
            ))),

            // LD (0xFF00+n), A  /  LD A, (0xFF00+n)
            0xE0 => Some(Instruction::LD(LoadType::ByteAddressFromA)),
            0xF0 => Some(Instruction::LD(LoadType::AFromByteAddress)),

            // LD (nn), A  /  LD A, (nn)
            0xEA => Some(Instruction::LD(LoadType::IndirectFromA(
                Indirect::WordIndirect,
            ))),
            0xFA => Some(Instruction::LD(LoadType::AFromIndirect(
                Indirect::WordIndirect,
            ))),

            // LD r, r  (register-to-register, rows 0x40–0x7F minus HALT)
            // B row
            0x40 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::B,
            ))),
            0x41 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::C,
            ))),
            0x42 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::D,
            ))),
            0x43 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::E,
            ))),
            0x44 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::H,
            ))),
            0x45 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::L,
            ))),
            0x46 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::HLI,
            ))),
            0x47 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                LoadByteSource::A,
            ))),
            // C row
            0x48 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::B,
            ))),
            0x49 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::C,
            ))),
            0x4A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::D,
            ))),
            0x4B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::E,
            ))),
            0x4C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::H,
            ))),
            0x4D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::L,
            ))),
            0x4E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::HLI,
            ))),
            0x4F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                LoadByteSource::A,
            ))),
            // D row
            0x50 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::B,
            ))),
            0x51 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::C,
            ))),
            0x52 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::D,
            ))),
            0x53 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::E,
            ))),
            0x54 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::H,
            ))),
            0x55 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::L,
            ))),
            0x56 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::HLI,
            ))),
            0x57 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                LoadByteSource::A,
            ))),
            // E row
            0x58 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::B,
            ))),
            0x59 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::C,
            ))),
            0x5A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::D,
            ))),
            0x5B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::E,
            ))),
            0x5C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::H,
            ))),
            0x5D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::L,
            ))),
            0x5E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::HLI,
            ))),
            0x5F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                LoadByteSource::A,
            ))),
            // H row
            0x60 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::B,
            ))),
            0x61 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::C,
            ))),
            0x62 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::D,
            ))),
            0x63 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::E,
            ))),
            0x64 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::H,
            ))),
            0x65 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::L,
            ))),
            0x66 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::HLI,
            ))),
            0x67 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                LoadByteSource::A,
            ))),
            // L row
            0x68 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::B,
            ))),
            0x69 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::C,
            ))),
            0x6A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::D,
            ))),
            0x6B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::E,
            ))),
            0x6C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::H,
            ))),
            0x6D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::L,
            ))),
            0x6E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::HLI,
            ))),
            0x6F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                LoadByteSource::A,
            ))),
            // (HL) row
            0x70 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::B,
            ))),
            0x71 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::C,
            ))),
            0x72 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::D,
            ))),
            0x73 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::E,
            ))),
            0x74 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::H,
            ))),
            0x75 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::L,
            ))),
            // 0x76 = HALT (handled above)
            0x77 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                LoadByteSource::A,
            ))),
            // A row
            0x78 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::B,
            ))),
            0x79 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::C,
            ))),
            0x7A => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::D,
            ))),
            0x7B => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::E,
            ))),
            0x7C => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::H,
            ))),
            0x7D => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::L,
            ))),
            0x7E => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::HLI,
            ))),
            0x7F => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                LoadByteSource::A,
            ))),

            // ADD A, r
            0x80 => Some(Instruction::ADD(ArithmeticTarget::B)),
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            0x82 => Some(Instruction::ADD(ArithmeticTarget::D)),
            0x83 => Some(Instruction::ADD(ArithmeticTarget::E)),
            0x84 => Some(Instruction::ADD(ArithmeticTarget::H)),
            0x85 => Some(Instruction::ADD(ArithmeticTarget::L)),
            0x87 => Some(Instruction::ADD(ArithmeticTarget::A)),

            // ADC A, r
            0x88 => Some(Instruction::ADC(ArithmeticTarget::B)),
            0x89 => Some(Instruction::ADC(ArithmeticTarget::C)),
            0x8A => Some(Instruction::ADC(ArithmeticTarget::D)),
            0x8B => Some(Instruction::ADC(ArithmeticTarget::E)),
            0x8C => Some(Instruction::ADC(ArithmeticTarget::H)),
            0x8D => Some(Instruction::ADC(ArithmeticTarget::L)),
            0x8F => Some(Instruction::ADC(ArithmeticTarget::A)),

            // SUB r
            0x90 => Some(Instruction::SUB(ArithmeticTarget::B)),
            0x91 => Some(Instruction::SUB(ArithmeticTarget::C)),
            0x92 => Some(Instruction::SUB(ArithmeticTarget::D)),
            0x93 => Some(Instruction::SUB(ArithmeticTarget::E)),
            0x94 => Some(Instruction::SUB(ArithmeticTarget::H)),
            0x95 => Some(Instruction::SUB(ArithmeticTarget::L)),
            0x97 => Some(Instruction::SUB(ArithmeticTarget::A)),

            // SBC A, r
            0x98 => Some(Instruction::SBC(ArithmeticTarget::B)),
            0x99 => Some(Instruction::SBC(ArithmeticTarget::C)),
            0x9A => Some(Instruction::SBC(ArithmeticTarget::D)),
            0x9B => Some(Instruction::SBC(ArithmeticTarget::E)),
            0x9C => Some(Instruction::SBC(ArithmeticTarget::H)),
            0x9D => Some(Instruction::SBC(ArithmeticTarget::L)),
            0x9F => Some(Instruction::SBC(ArithmeticTarget::A)),

            // AND r
            0xA0 => Some(Instruction::AND(ArithmeticTarget::B)),
            0xA1 => Some(Instruction::AND(ArithmeticTarget::C)),
            0xA2 => Some(Instruction::AND(ArithmeticTarget::D)),
            0xA3 => Some(Instruction::AND(ArithmeticTarget::E)),
            0xA4 => Some(Instruction::AND(ArithmeticTarget::H)),
            0xA5 => Some(Instruction::AND(ArithmeticTarget::L)),
            0xA7 => Some(Instruction::AND(ArithmeticTarget::A)),

            // XOR r
            0xA8 => Some(Instruction::XOR(ArithmeticTarget::B)),
            0xA9 => Some(Instruction::XOR(ArithmeticTarget::C)),
            0xAA => Some(Instruction::XOR(ArithmeticTarget::D)),
            0xAB => Some(Instruction::XOR(ArithmeticTarget::E)),
            0xAC => Some(Instruction::XOR(ArithmeticTarget::H)),
            0xAD => Some(Instruction::XOR(ArithmeticTarget::L)),
            0xAF => Some(Instruction::XOR(ArithmeticTarget::A)),

            // OR r
            0xB0 => Some(Instruction::OR(ArithmeticTarget::B)),
            0xB1 => Some(Instruction::OR(ArithmeticTarget::C)),
            0xB2 => Some(Instruction::OR(ArithmeticTarget::D)),
            0xB3 => Some(Instruction::OR(ArithmeticTarget::E)),
            0xB4 => Some(Instruction::OR(ArithmeticTarget::H)),
            0xB5 => Some(Instruction::OR(ArithmeticTarget::L)),
            0xB7 => Some(Instruction::OR(ArithmeticTarget::A)),

            // CP r
            0xB8 => Some(Instruction::CP(ArithmeticTarget::B)),
            0xB9 => Some(Instruction::CP(ArithmeticTarget::C)),
            0xBA => Some(Instruction::CP(ArithmeticTarget::D)),
            0xBB => Some(Instruction::CP(ArithmeticTarget::E)),
            0xBC => Some(Instruction::CP(ArithmeticTarget::H)),
            0xBD => Some(Instruction::CP(ArithmeticTarget::L)),
            0xBF => Some(Instruction::CP(ArithmeticTarget::A)),

            // RET conditional / unconditional
            0xC0 => Some(Instruction::RET(JumpTest::NotZero)),
            0xC8 => Some(Instruction::RET(JumpTest::Zero)),
            0xC9 => Some(Instruction::RET(JumpTest::Always)),
            0xD0 => Some(Instruction::RET(JumpTest::NotCarry)),
            0xD8 => Some(Instruction::RET(JumpTest::Carry)),

            // POP rr
            0xC1 => Some(Instruction::POP(StackTarget::BC)),
            0xD1 => Some(Instruction::POP(StackTarget::DE)),
            0xE1 => Some(Instruction::POP(StackTarget::HL)),
            0xF1 => Some(Instruction::POP(StackTarget::AF)),

            // JP conditional / unconditional
            0xC2 => Some(Instruction::JP(JumpTest::NotZero)),
            0xC3 => Some(Instruction::JP(JumpTest::Always)),
            0xCA => Some(Instruction::JP(JumpTest::Zero)),
            0xD2 => Some(Instruction::JP(JumpTest::NotCarry)),
            0xDA => Some(Instruction::JP(JumpTest::Carry)),

            // CALL conditional / unconditional
            0xC4 => Some(Instruction::CALL(JumpTest::NotZero)),
            0xCC => Some(Instruction::CALL(JumpTest::Zero)),
            0xCD => Some(Instruction::CALL(JumpTest::Always)),
            0xD4 => Some(Instruction::CALL(JumpTest::NotCarry)),
            0xDC => Some(Instruction::CALL(JumpTest::Carry)),

            // PUSH rr
            0xC5 => Some(Instruction::PUSH(StackTarget::BC)),
            0xD5 => Some(Instruction::PUSH(StackTarget::DE)),
            0xE5 => Some(Instruction::PUSH(StackTarget::HL)),
            0xF5 => Some(Instruction::PUSH(StackTarget::AF)),

            // Misc flag / rotate ops
            0x07 => Some(Instruction::RLCA),
            0x0F => Some(Instruction::RRCA),
            0x17 => Some(Instruction::RLA),
            0x1F => Some(Instruction::RRA),
            0x2F => Some(Instruction::CPL),
            0x37 => Some(Instruction::SCF),
            0x3F => Some(Instruction::CCF),

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
                let bit = (byte >> 3) & 0b111;
                Some(Instruction::BIT(bit, reg))
            }
            0x80..=0xBF => {
                let bit = (byte >> 3) & 0b111;
                Some(Instruction::RES(bit, reg))
            }
            0xC0..=0xFF => {
                let bit = (byte >> 3) & 0b111;
                Some(Instruction::SET(bit, reg))
            }
        }
    }
}

