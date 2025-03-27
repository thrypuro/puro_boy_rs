use core::panic;

use crate::gb::mmu::Memory;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum RegisterNames {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}
pub struct Flag {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

pub struct Registers {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub flag: Flag,
}



impl Registers {
    pub fn new() -> Self {
        Self {

            af: 0,
            bc: 0,
            de: 0,
            hl: 0,
            sp: 0xFFFF,
            pc: 0,
            flag: Flag {
                z: false,
                n: false,
                h: false,
                c: false,
            },
        }
    }
    
    pub fn get_register_value_8(&self, register: RegisterNames) -> u8 {
        match register {
            RegisterNames::A => (self.af >> 8) as u8,
            RegisterNames::B => (self.bc >> 8) as u8,
            RegisterNames::C => (self.bc & 0xFF) as u8,
            RegisterNames::D => (self.de >> 8) as u8,
            RegisterNames::E => (self.de & 0xFF) as u8,
            RegisterNames::H => (self.hl >> 8) as u8,
            RegisterNames::L => (self.hl & 0xFF) as u8,
            _ =>  panic!("Invalid register"),
        }
    }
    pub fn set_register_value_16(&mut self, register: RegisterNames, value: u16) {
       match register {
            RegisterNames::AF => self.af = value,
            RegisterNames::BC => self.bc = value,
            RegisterNames::DE => self.de = value,
            RegisterNames::HL => self.hl = value,
            RegisterNames::SP => self.sp = value,
            RegisterNames::PC => self.pc = value,
            _ => panic!("Invalid register"),
       }
        
    }
    pub fn get_register_value_16(&self, register: RegisterNames) -> u16 {
        match register {
            RegisterNames::AF => self.af,
            RegisterNames::BC => self.bc,
            RegisterNames::DE => self.de,
            RegisterNames::HL => self.hl,
            RegisterNames::SP => self.sp,
            RegisterNames::PC => self.pc,
            _ => panic!("Invalid register"),
        }
    }
    pub fn set_register_value_8(&mut self, register: RegisterNames, value: u8) {
        match register {
            RegisterNames::A => self.af = (self.af & 0xFF) | ((value as u16) << 8),
            RegisterNames::B => self.bc = (self.bc & 0x00FF) | ((value as u16) << 8),
            RegisterNames::C => self.bc = (self.bc & 0xFF00) | (value as u16),
            RegisterNames::D => self.de = (self.de & 0x00FF) | ((value as u16) << 8),
            RegisterNames::E => self.de = (self.de & 0xFF00) | (value as u16),
            RegisterNames::H => self.hl = (self.hl & 0x00FF) | ((value as u16) << 8),
            RegisterNames::L => self.hl = (self.hl & 0xFF00) | (value as u16),
            _ => panic!("Invalid register"),
        }
    }

   

}


