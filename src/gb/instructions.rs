use crate::gb::registers::{Registers, RegisterNames};
use crate::gb::mmu::MMU;
/// Represents an operand, which can be a register or a memory address.

#[derive(Debug)]
pub enum Operand {
    Register(RegisterNames),
    Memory(u16), // Memory address
    Immediate(u8), // Immediate value
    Immediate16(u16), // 16-bit immediate value
}


// get operand bit length 

impl Operand {
    pub fn read(&self, registers: &Registers, memory: &MMU) -> u8 {
        match self {
            Operand::Register(reg) => registers.get_register_value_8(*reg),
            Operand::Memory(addr) => memory.read(*addr),
            Operand::Immediate(value) => *value,
            _ => panic!("Invalid operand for read"),
        }
    }

    pub fn write(&self, value: u8, registers: &mut Registers, memory: &mut MMU) {
        match self {
            Operand::Register(reg) => registers.set_register_value_8(*reg, value),
            Operand::Memory(addr) => memory.write(*addr, value),
            _ => panic!("Invalid operand for write"),
            
        }
    }
    pub fn write_u16(&self, value: u16, registers: &mut Registers) {
        match self {
            Operand::Register(RegisterNames::AF) => registers.af = value,
            Operand::Register(RegisterNames::BC) => registers.bc = value,
            Operand::Register(RegisterNames::DE) => registers.de = value,
            Operand::Register(RegisterNames::HL) => registers.hl = value,

            _ => panic!("Invalid register for 16-bit write"),
        }
    }
    pub fn read_16(&self, registers: &Registers) -> u16 {
        match self {
            Operand::Register(RegisterNames::AF) => registers.af,
            Operand::Register(RegisterNames::BC) => registers.bc,
            Operand::Register(RegisterNames::DE) => registers.de,
            Operand::Register(RegisterNames::HL) => registers.hl,
            Operand::Immediate16(value) => *value,
            _ => panic!("Invalid register for 16-bit read"),
        }
    }
    pub fn get_bit_length(&self) -> u8 {
        match self {
            Operand::Register(a) => match a {
                RegisterNames::A => 8,
                RegisterNames::B => 8,
                RegisterNames::C => 8,
                RegisterNames::D => 8,
                RegisterNames::E => 8,
                RegisterNames::H => 8,
                RegisterNames::L => 8,
                RegisterNames::AF => 16,
                RegisterNames::BC => 16,
                RegisterNames::DE => 16,
                RegisterNames::HL => 16,
                RegisterNames::SP => 16,
                RegisterNames::PC => 16,

            },
            Operand::Memory(_) => 8,
            Operand::Immediate(_) => 8,
            Operand::Immediate16(_) => 16,
        }
        
    }
}


pub fn inc_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let result = value.wrapping_add(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = (value & 0x0F) == 0x0F;

    // Write result back to the operand
    operand.write(result, registers, memory);
}
pub fn dec_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let result = value.wrapping_sub(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value & 0x0F) == 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn inc_16bit(
    registers: &mut Registers,
    operand: Operand,
) {
    let value = operand.read_16(registers);
    let result = value.wrapping_add(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = (value & 0x0FFF) == 0x0FFF;

    // Write result back to the operand
    operand.write_u16(result, registers);
}

pub fn dec_16bit(
    registers: &mut Registers,
    operand: Operand,
) {
    let value = operand.read_16(registers);
    let result = value.wrapping_sub(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value & 0x0FFF) == 0;

    // Write result back to the operand
    operand.write_u16(result, registers);
}


pub fn add_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1.wrapping_add(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = ((value1 & 0x0F) + (value2 & 0x0F)) > 0x0F;
    registers.flag.c = (value1 as u16 + value2 as u16) > 0xFF;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}


pub fn adc_8bit (
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let carry = if registers.flag.c { 1 } else { 0 };
    let result = value1.wrapping_add(value2).wrapping_add(carry);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = ((value1 & 0x0F) + (value2 & 0x0F) + carry) > 0x0F;
    registers.flag.c = (value1 as u16 + value2 as u16 + carry as u16) > 0xFF;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}


pub fn add_16bit(
    registers: &mut Registers,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read_16(registers);
    let value2 = operand2.read_16(registers);
    let result = value1.wrapping_add(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = ((value1 & 0x0FFF) + (value2 & 0x0FFF)) > 0x0FFF;
    registers.flag.c = (value1 as u32 + value2 as u32) > 0xFFFF;

    // Write result back to the first operand
    operand1.write_u16(result, registers);
}

pub fn sub_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0F) < (value2 & 0x0F);
    registers.flag.c = (value1 as u16) < (value2 as u16);

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn sbc_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let carry = if registers.flag.c { 1 } else { 0 };
    let result = value1.wrapping_sub(value2).wrapping_sub(carry);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0F) < (value2 & 0x0F) + carry;
    registers.flag.c = (value1 as u16) < (value2 as u16) + carry as u16;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn sub_16bit(
    registers: &mut Registers,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read_16(registers);
    let value2 = operand2.read_16(registers);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0FFF) < (value2 & 0x0FFF);
    registers.flag.c = (value1 as u32) < (value2 as u32);

    // Write result back to the first operand
    operand1.write_u16(result, registers);
}


pub fn and_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1 & value2;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = true;
    registers.flag.c = false;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn or_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1 | value2;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = false;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn xor_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1 ^ value2;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = false;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}
pub fn cp_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0F) < (value2 & 0x0F);
    registers.flag.c = (value1 as u16) < (value2 as u16);
}

pub fn ld_8bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand1: Operand,
    operand2: Operand,
) {
    let value = operand2.read(registers, memory);
    operand1.write(value, registers, memory);
}

pub fn ld_16bit(
    registers: &mut Registers,
    operand1: Operand,
    operand2: Operand,
) {
    let value = operand2.read_16(registers);
    operand1.write_u16(value, registers);
}

// rlc : rotate left circular
pub fn rlc(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let result = (value << 1) | (value >> 7);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x80) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn rrc (
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let result = (value >> 1) | (value << 7);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x01) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn rl (
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let carry = registers.flag.c as u8;
    let result = (value << 1) | carry;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x80) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn rr (
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let carry = registers.flag.c as u8;
    let result = (value >> 1) | (carry << 7);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x01) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// shift left arithmetic
pub fn sla(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let result = value << 1;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x80) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}
// shift right arithmetic
pub fn sra(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let result = (value >> 1) | (value & 0x80);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x01) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Swap nibbles
pub fn swap(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read(registers, memory);
    let result = (value << 4) | (value >> 4);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = false;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Bit test
pub fn bit(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
    bit: u8,
) {
    let value = operand.read(registers, memory);
    let result = value & (1 << bit);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = true;
    registers.flag.c = false;
}

// Bit set
pub fn set(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
    bit: u8,
) {
    let value = operand.read(registers, memory);
    let result = value | (1 << bit);

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Bit reset
pub fn res(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
    bit: u8,
) {
    let value = operand.read(registers, memory);
    let result = value & !(1 << bit);

    // Write result back to the operand
    operand.write(result, registers, memory);
}



// Control flow instructions

pub fn jp(
    registers: &mut Registers,
    memory: &mut MMU,
    pc : &mut u16,
    operand: Operand,
    condition: bool,

) {
    if condition {
        let address = operand.read_16(registers);
        *pc = address;
    }
}

pub fn call(
    registers: &mut Registers,
    memory: &mut MMU,
    pc : &mut u16,
    sp : &mut u16,
    operand: Operand,
    condition: bool,

) {
    if condition {
        let address = operand.read_16(registers);
        *sp = sp.wrapping_sub(2);
        memory.write_word(*sp, *pc);
        *pc = address;
    }
}

pub fn jr(
    registers: &mut Registers,
    memory: &mut MMU,
    pc : &mut  u16,
    operand: Operand,
) {
    let offset = operand.read(registers, memory) as i8;
    pc.wrapping_add(offset as u16);
}


pub fn jr_conditional(
    registers: &mut Registers,
    memory: &mut MMU,
    pc : &mut u16,
    condition: bool,
    operand: Operand,
) {
    if condition {
        let offset = operand.read(registers, memory) as i8;
        *pc = pc.wrapping_add(offset as u16);
    }
}


pub fn ret(
    _registers: &mut Registers,
    memory: &mut MMU,
    pc : &mut u16,
    sp : &mut u16,
    cond : bool,
) {

    // Pop the address from the stack and set it as the new program counter
    if cond {
        let low = memory.read(*sp);
        let high = memory.read(*sp + 1);
        *pc = ((high as u16) << 8) | (low as u16);
        *sp = sp.wrapping_add(2);
    }
    

}



pub fn daa(
    registers: &mut Registers,) 
    {
    let mut a = registers.af >> 8;
    a &= 0xFF; // Ensure a is 8 bits
    let mut carry = 0;

    if registers.flag.n {
        if registers.flag.c {
            a = a.wrapping_sub(0x60);
            carry = 1;
        }
        if registers.flag.h {
            a = a.wrapping_sub(0x06);
            carry = 1;
        }
    } else {
        if registers.flag.c || a > 0x99 {
            a = a.wrapping_add(0x60);
            carry = 1;
        }
        if registers.flag.h || (a & 0x0F) > 0x09 {
            a = a.wrapping_add(0x06);
            carry = 1;
        }
    }

    registers.af = (registers.af & 0xFF00) | (a << 8);
    registers.flag.z = a == 0;
    registers.flag.h = false;
    registers.flag.c = carry != 0;
}


pub fn cpl(
    registers: &mut Registers,
) {
    let a = registers.af >> 8;
    registers.af = (registers.af & 0xFF00) | (!a << 8);
    registers.flag.n = true;
    registers.flag.h = true;
}
pub fn scf(
    registers: &mut Registers,
) {
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = true;
}
pub fn ccf(
    registers: &mut Registers,
) {
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = !registers.flag.c;
}


pub fn push(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let value = operand.read_16(registers);
    let sp = &mut registers.sp;
    *sp = sp.wrapping_sub(2);
    memory.write_word(*sp, value);
}

pub fn pop(
    registers: &mut Registers,
    memory: &mut MMU,
    operand: Operand,
) {
    let sp = &mut registers.sp;
    let value = memory.read_word(*sp);
    *sp = sp.wrapping_add(2);
    operand.write_u16(value, registers);
}