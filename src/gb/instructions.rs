use crate::gb::registers::{Registers, RegisterNames};
use crate::gb::mmu::MMU;
/// Represents an operand, which can be a register or a memory address.
pub enum Operand {
    Register(RegisterNames),
    Memory(u16), // Memory address
    Immediate(u8), // Immediate value
}

impl Operand {
    pub fn read(&self, registers: &Registers, memory: &MMU) -> u8 {
        match self {
            Operand::Register(reg) => registers.get_register_value_8(*reg),
            Operand::Memory(addr) => memory.read(*addr),
            Operand::Immediate(value) => *value,
        }
    }

    pub fn write(&self, value: u8, registers: &mut Registers, memory: &mut MMU) {
        match self {
            Operand::Register(reg) => registers.set_register_value_8(*reg, value),
            Operand::Memory(addr) => memory.write(*addr, value),
            Operand::Immediate(_) => panic!("Cannot write to an immediate value"),
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
            _ => panic!("Invalid register for 16-bit read"),
        }
    }
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
    mut pc : u16,
    operand: Operand,
    
) {
    let address = operand.read_16(registers);
    pc = address;
}