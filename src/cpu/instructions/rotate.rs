use crate::cpu::shared::{Operand, Registers};
use crate::MMU;

// rlc : rotate left circular
pub fn rlc(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
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

pub fn rrc(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
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

pub fn rl(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
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

pub fn rr(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
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
pub fn sla(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
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
pub fn sra(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
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
pub fn swap(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
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
