use crate::cpu::shared::{Operand, Registers};
use crate::MMU;

// Bit test
pub fn bit(registers: &mut Registers, memory: &mut MMU, operand: Operand, bit: u8) {
    let value = operand.read(registers, memory);
    let result = value & (1 << bit);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = true;
    registers.flag.c = false;
}

// Bit set
pub fn set(registers: &mut Registers, memory: &mut MMU, operand: Operand, bit: u8) {
    let value = operand.read(registers, memory);
    let result = value | (1 << bit);

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Bit reset
pub fn res(registers: &mut Registers, memory: &mut MMU, operand: Operand, bit: u8) {
    let value = operand.read(registers, memory);
    let result = value & !(1 << bit);

    // Write result back to the operand
    operand.write(result, registers, memory);
}
