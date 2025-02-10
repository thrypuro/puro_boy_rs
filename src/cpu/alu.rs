pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8,
    pub pc: u16,
    pub sp: u16,

}


enum  register_names{
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


pub fn add(registers: &mut Registers, value: u8) {
    let result = registers.a as u16 + value as u16;
    registers.f = 0;
    if result > 0xFF {
        registers.f |= 0x10;
    }
    if (registers.a as u16 & 0x0F) + (value as u16 & 0x0F) > 0x0F {
        registers.f |= 0x20;
    }
    registers.a = result as u8;
    if registers.a == 0 {
        registers.f |= 0x80;
    }
}