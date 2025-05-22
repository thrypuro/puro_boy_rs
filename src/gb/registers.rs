use super::{Flag, FlagNames, RegisterNames, Registers};
use core::panic;
impl Flag {
    pub fn new() -> Self {
        Self {
            z: false,
            n: false,
            h: false,
            c: false,
        }
    }
    pub fn set_z(&mut self, value: bool) {
        self.z = value;
    }
    pub fn set_n(&mut self, value: bool) {
        self.n = value;
    }
    pub fn set_h(&mut self, value: bool) {
        self.h = value;
    }
    pub fn set_c(&mut self, value: bool) {
        self.c = value;
    }
    pub fn get_z(&self) -> bool {
        self.z
    }
    pub fn get_n(&self) -> bool {
        self.n
    }
    pub fn get_h(&self) -> bool {
        self.h
    }
    pub fn get_c(&self) -> bool {
        self.c
    }
    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.z = z;
        self.n = n;
        self.h = h;
        self.c = c;
    }
    pub fn get_flags(&self) -> (bool, bool, bool, bool) {
        (self.z, self.n, self.h, self.c)
    }
    pub fn reset_flags(&mut self) {
        self.z = false;
        self.n = false;
        self.h = false;
        self.c = false;
    }

    pub fn set_flag_value(&mut self, value: u8) {
        let flag_reg = value >> 4;
        let new_z: bool = ((flag_reg & 0b1000) >> 3) == 1;
        let new_n: bool = ((flag_reg & 0b0100) >> 2) == 1;
        let new_h: bool = ((flag_reg & 0b0010) >> 1) == 1;
        let new_c: bool = (flag_reg & 0b0001) == 1;
        self.set_flags(new_z, new_n, new_h, new_c);
    }
}

impl Registers {
    pub fn new() -> Self {
        Self {
            af: 0x1 << 8,
            bc: 0x13,
            de: 0xd8,
            hl: 0x14d,
            sp: 0xFFFE,
            pc: 0x100,
            flag: Flag {
                z: true,
                n: false,
                h: true,
                c: true,
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
            _ => panic!("Invalid register {:?}", register),
        }
    }
    pub fn set_register_value_16(&mut self, register: RegisterNames, value: u16) {
        match register {
            RegisterNames::AF => {
                self.af = value;
                self.flag.set_flag_value((value & 0xff) as u8);
            }
            RegisterNames::BC => self.bc = value,
            RegisterNames::DE => self.de = value,
            RegisterNames::HL => self.hl = value,
            RegisterNames::SP => self.sp = value,

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
    pub fn get_flag(&self, flag: &FlagNames) -> bool {
        match flag {
            FlagNames::Z => self.get_zero_flag(),
            FlagNames::H => self.get_half_carry_flag(),
            FlagNames::C => self.get_carry_flag(),
            FlagNames::N => self.get_subtract_flag(),
            FlagNames::NZ => !self.get_zero_flag(),
            FlagNames::NC => !self.get_carry_flag(),
        }
    }

    pub fn get_carry_flag(&self) -> bool {
        self.flag.get_c()
    }
    pub fn set_carry_flag(&mut self, value: bool) {
        self.flag.set_c(value);
    }
    pub fn get_zero_flag(&self) -> bool {
        self.flag.get_z()
    }
    pub fn set_zero_flag(&mut self, value: bool) {
        self.flag.set_z(value);
    }
    pub fn get_half_carry_flag(&self) -> bool {
        self.flag.get_h()
    }
    pub fn set_half_carry_flag(&mut self, value: bool) {
        self.flag.set_h(value);
    }
    pub fn get_subtract_flag(&self) -> bool {
        self.flag.get_n()
    }
    pub fn set_subtract_flag(&mut self, value: bool) {
        self.flag.set_n(value);
    }
    pub fn get_flags(&self) -> (bool, bool, bool, bool) {
        self.flag.get_flags()
    }
    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.flag.set_flags(z, n, h, c);
    }
    pub fn reset_flags(&mut self) {
        self.flag.reset_flags();
    }
}
