use crate::cpu::shared::{Operand, Registers};
use crate::MMU;

pub fn and_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
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

pub fn or_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
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

pub fn xor_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
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
