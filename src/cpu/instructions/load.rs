use crate::cpu::shared::{Operand, RegisterNames, Registers};
use crate::MMU;

pub fn ld(registers: &mut Registers, operand1: Operand, operand2: Operand, memory: &mut MMU) {
    let mut value: u16 = 0;
    if operand2.get_bit_length() == 8 {
        value = operand2.read(registers, memory) as u16;
    } else {
        value = operand2.read_16(registers, memory);
    }
    if operand1.get_bit_length() == 16 {
        operand1.write_u16(value, registers, memory);
    } else {
        operand1.write(value as u8, registers, memory);
    }
}

pub fn ldh(registers: &mut Registers, operand1: Operand, operand2: Operand, memory: &mut MMU) {
    // Calculate the high memory address - always 0xFF00 + offset
    let addr = match operand1 {
        Operand::Register(RegisterNames::A) => {
            // LDH A, (a8) or LDH A, (C)
            let offset = match operand2 {
                Operand::Immediate(imm) => imm as u16,
                Operand::Register(RegisterNames::C) => {
                    registers.get_register_value_8(RegisterNames::C) as u16
                }
                _ => panic!("Invalid operand for LDH instruction"),
            };
            let addr = 0xFF00 + offset;
            let value = memory.read(addr).expect("MMU read");
            registers.set_register_value_8(RegisterNames::A, value);
            return;
        }
        Operand::Immediate(imm) => 0xFF00 + imm as u16,
        Operand::Register(RegisterNames::C) => {
            0xFF00 + registers.get_register_value_8(RegisterNames::C) as u16
        }
        _ => panic!("Invalid operand for LDH instruction"),
    };

    // Store the value from register A to the high memory address
    let value = registers.get_register_value_8(RegisterNames::A);
    memory.write(addr, value).expect("MMU write");
}
