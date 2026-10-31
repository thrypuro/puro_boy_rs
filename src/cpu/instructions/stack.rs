use crate::cpu::shared::{Operand, Registers};
use crate::MMU;

pub fn push(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read_16(registers, memory);
    let sp = &mut registers.sp;
    *sp = sp.wrapping_sub(2);
    memory.write_word(*sp, value).expect("MMU write");
}

pub fn pop(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let sp = &mut registers.sp;
    let value = memory.read_word(*sp).expect("MMU read");
    *sp = sp.wrapping_add(2);
    operand.write_u16(value, registers, memory);
}
