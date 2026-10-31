//! Arithmetic, logic and shift operations: the parts of instructions that compute
//! results and flags. Operand fetching and cycle counts live in `execute.rs`.

use super::CPU;

impl<'a> CPU<'a> {
    /// The eight accumulator operations, selected by the opcode's y field:
    /// 0=ADD 1=ADC 2=SUB 3=SBC 4=AND 5=XOR 6=OR 7=CP.
    pub(super) fn alu(&mut self, op: u8, value: u8) {
        let a = self.a();
        let carry_in = self.registers.flag.c as u8;
        let flags = &mut self.registers;

        let result = match op & 7 {
            0 | 1 => {
                // ADD / ADC
                let carry = if op == 1 { carry_in } else { 0 };
                let sum = a as u16 + value as u16 + carry as u16;
                let result = sum as u8;
                let half = (a & 0x0F) + (value & 0x0F) + carry > 0x0F;
                flags.set_flags(result == 0, false, half, sum > 0xFF);
                result
            }
            2 | 3 | 7 => {
                // SUB / SBC / CP
                let carry = if op == 3 { carry_in } else { 0 };
                let result = a.wrapping_sub(value).wrapping_sub(carry);
                let half = (a & 0x0F) < (value & 0x0F) + carry;
                let borrow = (a as u16) < value as u16 + carry as u16;
                flags.set_flags(result == 0, true, half, borrow);
                if op == 7 {
                    return; // CP only sets flags
                }
                result
            }
            4 => {
                let result = a & value;
                flags.set_flags(result == 0, false, true, false);
                result
            }
            5 => {
                let result = a ^ value;
                flags.set_flags(result == 0, false, false, false);
                result
            }
            _ => {
                let result = a | value;
                flags.set_flags(result == 0, false, false, false);
                result
            }
        };
        self.set_a(result);
    }

    /// INC r: C is unchanged.
    pub(super) fn inc8(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        let flag = &mut self.registers.flag;
        flag.z = result == 0;
        flag.n = false;
        flag.h = value & 0x0F == 0x0F;
        result
    }

    /// DEC r: C is unchanged.
    pub(super) fn dec8(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        let flag = &mut self.registers.flag;
        flag.z = result == 0;
        flag.n = true;
        flag.h = value & 0x0F == 0;
        result
    }

    /// ADD HL, rr: Z is unchanged, H/C come from bits 11 and 15.
    pub(super) fn add_hl(&mut self, value: u16) {
        let hl = self.registers.hl;
        let (result, carry) = hl.overflowing_add(value);
        let half = (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF;
        let z = self.registers.flag.z;
        self.registers.set_flags(z, false, half, carry);
        self.registers.hl = result;
    }

    /// SP + signed 8-bit offset, shared by ADD SP,e8 and LD HL,SP+e8.
    /// Z and N are cleared; H and C come from an *unsigned* add of the low byte.
    pub(super) fn sp_plus_e8(&mut self, offset: u8) -> u16 {
        let sp = self.registers.sp;
        let half = (sp & 0x0F) + (offset as u16 & 0x0F) > 0x0F;
        let carry = (sp & 0xFF) + offset as u16 > 0xFF;
        self.registers.set_flags(false, false, half, carry);
        sp.wrapping_add_signed(offset as i8 as i16)
    }

    /// Decimal-adjusts A after a BCD add or subtract.
    pub(super) fn daa(&mut self) {
        let mut a = self.a();
        let (_, n, h, mut c) = self.registers.get_flags();
        if !n {
            if c || a > 0x99 {
                a = a.wrapping_add(0x60);
                c = true;
            }
            if h || a & 0x0F > 0x09 {
                a = a.wrapping_add(0x06);
            }
        } else {
            if c {
                a = a.wrapping_sub(0x60);
            }
            if h {
                a = a.wrapping_sub(0x06);
            }
        }
        self.registers.set_flags(a == 0, n, false, c);
        self.set_a(a);
    }

    /// The CB rotate/shift group, selected by the opcode's y field:
    /// 0=RLC 1=RRC 2=RL 3=RR 4=SLA 5=SRA 6=SWAP 7=SRL.
    /// Sets Z from the result, clears N and H, and sets C to the bit shifted out.
    /// (RLCA/RRCA/RLA/RRA reuse 0..3 and then clear Z.)
    pub(super) fn shift(&mut self, op: u8, value: u8) -> u8 {
        let carry_in = self.registers.flag.c as u8;
        let (result, carry) = match op & 7 {
            0 => (value.rotate_left(1), value & 0x80 != 0),
            1 => (value.rotate_right(1), value & 0x01 != 0),
            2 => ((value << 1) | carry_in, value & 0x80 != 0),
            3 => ((value >> 1) | (carry_in << 7), value & 0x01 != 0),
            4 => (value << 1, value & 0x80 != 0),
            5 => ((value >> 1) | (value & 0x80), value & 0x01 != 0),
            6 => (value.rotate_left(4), false),
            _ => (value >> 1, value & 0x01 != 0),
        };
        self.registers.set_flags(result == 0, false, false, carry);
        result
    }
}
