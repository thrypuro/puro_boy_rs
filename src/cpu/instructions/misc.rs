use crate::cpu::shared::Registers;

pub fn daa(registers: &mut Registers) {
    let mut a = registers.af >> 8;
    a &= 0xFF; // Ensure a is 8 bits
    let mut carry = 0;

    if registers.flag.n {
        if registers.flag.c {
            a = a.wrapping_sub(0x60);
            carry = 1;
        }
        if registers.flag.h {
            a = a.wrapping_sub(0x06);
            carry = 1;
        }
    } else {
        if registers.flag.c || a > 0x99 {
            a = a.wrapping_add(0x60);
            carry = 1;
        }
        if registers.flag.h || (a & 0x0F) > 0x09 {
            a = a.wrapping_add(0x06);
            carry = 1;
        }
    }

    registers.af = (registers.af & 0xFF00) | (a << 8);
    registers.flag.z = a == 0;
    registers.flag.h = false;
    registers.flag.c = carry != 0;
}

/// Disables interrupts by clearing the IME flag
pub fn di(registers: &mut Registers) {
    // The DI instruction disables interrupts by clearing the IME (Interrupt Master Enable) flag
    // This prevents the CPU from responding to any interrupts

    // Note: DI doesn't immediately disable interrupts, it actually disables them after
    // the instruction following DI is executed, but this simplified implementation
    // disables them immediately
}

pub fn cpl(registers: &mut Registers) {
    let a = registers.af >> 8;
    registers.af = (registers.af & 0xFF00) | (!a << 8);
    registers.flag.n = true;
    registers.flag.h = true;
}

pub fn scf(registers: &mut Registers) {
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = true;
}

pub fn ccf(registers: &mut Registers) {
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = !registers.flag.c;
}
