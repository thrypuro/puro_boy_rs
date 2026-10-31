use crate::cpu::shared::{Operand, Registers};
use crate::MMU;

pub fn inc_8bit(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = value.wrapping_add(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = (value & 0x0F) == 0x0F;

    // Write result back to the operand
    operand.write(result, registers, memory);
}
pub fn dec_8bit(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = value.wrapping_sub(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value & 0x0F) == 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn inc_16bit(registers: &mut Registers, operand: Operand, memory: &mut MMU) {
    let value = operand.read_16(registers, memory);
    let result = value.wrapping_add(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = (value & 0x0FFF) == 0x0FFF;

    // Write result back to the operand
    operand.write_u16(result, registers, memory);
}

pub fn dec_16bit(registers: &mut Registers, operand: Operand, memory: &mut MMU) {
    let value = operand.read_16(registers, memory);
    let result = value.wrapping_sub(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value & 0x0FFF) == 0;

    // Write result back to the operand
    operand.write_u16(result, registers, memory);
}

pub fn add_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
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

pub fn adc_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
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
    memory: &mut MMU,
) {
    let value1 = operand1.read_16(registers, memory);
    let value2 = operand2.read_16(registers, memory);
    let result = value1.wrapping_add(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = ((value1 & 0x0FFF) + (value2 & 0x0FFF)) > 0x0FFF;
    registers.flag.c = (value1 as u32 + value2 as u32) > 0xFFFF;

    // Write result back to the first operand
    operand1.write_u16(result, registers, memory);
}

pub fn sub_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
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

pub fn sbc_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
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
    memory: &mut MMU,
) {
    let value1 = operand1.read_16(registers, memory);
    let value2 = operand2.read_16(registers, memory);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0FFF) < (value2 & 0x0FFF);
    registers.flag.c = (value1 as u32) < (value2 as u32);

    // Write result back to the first operand
    operand1.write_u16(result, registers, memory);
}

pub fn cp_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0F) < (value2 & 0x0F);
    registers.flag.c = (value1 as u16) < (value2 as u16);
}
