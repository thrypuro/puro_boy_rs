use crate::cpu::shared::{Flag, RegisterNames, Registers};

impl Flag {
    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.z = z;
        self.n = n;
        self.h = h;
        self.c = c;
    }

    pub fn get_flags(&self) -> (bool, bool, bool, bool) {
        (self.z, self.n, self.h, self.c)
    }

    /// The F register: Z N H C in bits 7..4, low nibble always 0.
    pub fn to_byte(&self) -> u8 {
        (self.z as u8) << 7 | (self.n as u8) << 6 | (self.h as u8) << 5 | (self.c as u8) << 4
    }

    pub fn set_from_byte(&mut self, value: u8) {
        self.set_flags(
            value & 0x80 != 0,
            value & 0x40 != 0,
            value & 0x20 != 0,
            value & 0x10 != 0,
        );
    }
}

impl Registers {
    /// Register values the DMG boot ROM leaves behind.
    pub fn new() -> Self {
        Self {
            af: 0x0100,
            bc: 0x0013,
            de: 0x00D8,
            hl: 0x014D,
            sp: 0xFFFE,
            pc: 0x0100,
            // F = 0xB0
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
            RegisterNames::C => self.bc as u8,
            RegisterNames::D => (self.de >> 8) as u8,
            RegisterNames::E => self.de as u8,
            RegisterNames::H => (self.hl >> 8) as u8,
            RegisterNames::L => self.hl as u8,
            _ => panic!("{register:?} is not an 8-bit register"),
        }
    }

    pub fn set_register_value_8(&mut self, register: RegisterNames, value: u8) {
        let hi = |pair: u16| (pair & 0x00FF) | (value as u16) << 8;
        let lo = |pair: u16| (pair & 0xFF00) | value as u16;
        match register {
            RegisterNames::A => self.af = hi(self.af),
            RegisterNames::B => self.bc = hi(self.bc),
            RegisterNames::C => self.bc = lo(self.bc),
            RegisterNames::D => self.de = hi(self.de),
            RegisterNames::E => self.de = lo(self.de),
            RegisterNames::H => self.hl = hi(self.hl),
            RegisterNames::L => self.hl = lo(self.hl),
            _ => panic!("{register:?} is not an 8-bit register"),
        }
    }

    pub fn get_register_value_16(&self, register: RegisterNames) -> u16 {
        match register {
            RegisterNames::AF => (self.af & 0xFF00) | self.flag.to_byte() as u16,
            RegisterNames::BC => self.bc,
            RegisterNames::DE => self.de,
            RegisterNames::HL => self.hl,
            RegisterNames::SP => self.sp,
            _ => panic!("{register:?} is not a 16-bit register"),
        }
    }

    pub fn set_register_value_16(&mut self, register: RegisterNames, value: u16) {
        match register {
            RegisterNames::AF => {
                self.af = value & 0xFF00;
                self.flag.set_from_byte(value as u8);
            }
            RegisterNames::BC => self.bc = value,
            RegisterNames::DE => self.de = value,
            RegisterNames::HL => self.hl = value,
            RegisterNames::SP => self.sp = value,
            _ => panic!("{register:?} is not a 16-bit register"),
        }
    }

    /// (Z, N, H, C)
    pub fn get_flags(&self) -> (bool, bool, bool, bool) {
        self.flag.get_flags()
    }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.flag.set_flags(z, n, h, c);
    }
}
