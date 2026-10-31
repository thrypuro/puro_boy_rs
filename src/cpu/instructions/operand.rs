use crate::cpu::shared::{Operand, RegisterNames, Registers};
use crate::MMU;

// get operand bit length
impl Operand {
    pub fn read(&self, registers: &Registers, memory: &MMU) -> u8 {
        match self {
            Operand::Register(reg) => registers.get_register_value_8(*reg),
            Operand::Memory(addr) => memory.read(*addr).expect("MMU read"),
            Operand::Immediate(value) => *value,
            Operand::Flag(flag) => registers.get_flag(flag) as u8,
            _ => panic!("Invalid operand for read"),
        }
    }

    pub fn write(&self, value: u8, registers: &mut Registers, memory: &mut MMU) {
        match self {
            Operand::Register(reg) => registers.set_register_value_8(*reg, value),
            Operand::Memory(addr) => memory.write(*addr, value).expect("MMU write"),
            _ => panic!("Invalid operand for write"),
        }
    }
    pub fn write_u16(&self, value: u16, registers: &mut Registers, memory: &mut MMU) {
        match self {
            Operand::Register(reg) => registers.set_register_value_16(*reg, value),
            Operand::Memory(addr) => memory.write(*addr, value as u8).expect("MMU write"),

            _ => panic!("Invalid register for 16-bit write {:?}", self),
        }
    }
    pub fn read_16(&self, registers: &Registers, memory: &mut MMU) -> u16 {
        match self {
            Operand::Register(reg) => registers.get_register_value_16(*reg),
            Operand::Immediate16(value) => *value,
            Operand::Memory(addr) => memory.read(*addr).expect("MMU read") as u16,
            _ => panic!("Invalid register for 16-bit read {:?}", self),
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
            Operand::Memory(i) => {
                if *i > 255 {
                    16
                } else {
                    8
                }
            }
            Operand::Immediate(_) => 8,
            Operand::Immediate16(_) => 16,
            Operand::Flag(_) => 16,
            _ => 0,
        }
    }
}
