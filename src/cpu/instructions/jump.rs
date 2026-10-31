use crate::cpu::shared::{Operand, Registers};
use crate::MMU;

// Control flow instructions

pub fn jp(registers: &mut Registers, memory: &mut MMU, operand: Operand, condition: bool) {
    if condition {
        let address = operand.read_16(registers, memory);
        let pc = &mut registers.pc;
        *pc = address;
    }
}

pub fn call(registers: &mut Registers, memory: &mut MMU, operand: Operand, condition: bool) {
    if condition {
        let address = operand.read_16(registers, memory);
        let pc = &mut registers.pc;
        let sp = &mut registers.sp;
        *sp = sp.wrapping_sub(2);
        memory.write_word(*sp, *pc).expect("MMU write");
        *pc = address;
    }
}

pub fn jr(registers: &mut Registers, memory: &mut MMU, operand: Operand, condition: bool) {
    if condition {
        let offset = operand.read(registers, memory) as i8;
        let pc = &mut registers.pc;
        *pc = pc.wrapping_add(offset as u16);
    }
}

pub fn ret(registers: &mut Registers, memory: &mut MMU, cond: bool) {
    // Pop the address from the stack and set it as the new program counter
    if cond {
        let sp = &mut registers.sp;
        let low = memory.read(*sp).expect("MMU read");
        let high = memory.read(*sp + 1).expect("MMU read");
        let pc = &mut registers.pc;
        *pc = ((high as u16) << 8) | (low as u16);
        *sp = sp.wrapping_add(2);
    }
}
