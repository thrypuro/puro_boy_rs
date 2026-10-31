//! Opcode dispatch. Each arm executes one instruction and returns its T-cycle count.
//!
//! Many instructions encode their operands in bit fields of the opcode:
//!   op = xx yyy zzz   (x = op >> 6, y = (op >> 3) & 7, z = op & 7)
//!   p = y >> 1 selects a register pair, y & 3 a branch condition.
//! The helpers in `cpu.rs` (`get_r8`, `get_rp`, `condition`, ...) take those indices.
//!
//! Reference: https://gbdev.io/gb-opcodes/optables/

use super::CPU;

impl<'a> CPU<'a> {
    pub(super) fn execute(&mut self, op: u8) -> u8 {
        let y = (op >> 3) & 7;
        let z = op & 7;
        let p = y >> 1;

        match op {
            0x00 => 4, // NOP
            0x10 => {
                // STOP: a 2-byte instruction. Low-power mode isn't emulated.
                self.fetch8();
                4
            }
            0x76 => {
                self.halted = true;
                4
            }

            // ---- 8-bit loads -------------------------------------------------------
            0x40..=0x7F => {
                // LD r, r'  (0x76 is HALT, handled above)
                let value = self.get_r8(z);
                self.set_r8(y, value);
                if y == 6 || z == 6 {
                    8
                } else {
                    4
                }
            }
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                // LD r, n8
                let value = self.fetch8();
                self.set_r8(y, value);
                if y == 6 {
                    12
                } else {
                    8
                }
            }
            0x02 | 0x12 => {
                // LD (BC), A / LD (DE), A
                self.write8(self.get_rp(p), self.a());
                8
            }
            0x0A | 0x1A => {
                // LD A, (BC) / LD A, (DE)
                let value = self.read8(self.get_rp(p));
                self.set_a(value);
                8
            }
            0x22 | 0x32 => {
                // LD (HL+), A / LD (HL-), A
                let hl = self.registers.hl;
                self.write8(hl, self.a());
                self.registers.hl = if op == 0x22 {
                    hl.wrapping_add(1)
                } else {
                    hl.wrapping_sub(1)
                };
                8
            }
            0x2A | 0x3A => {
                // LD A, (HL+) / LD A, (HL-)
                let hl = self.registers.hl;
                let value = self.read8(hl);
                self.set_a(value);
                self.registers.hl = if op == 0x2A {
                    hl.wrapping_add(1)
                } else {
                    hl.wrapping_sub(1)
                };
                8
            }
            0xE0 => {
                // LDH (a8), A
                let addr = 0xFF00 | self.fetch8() as u16;
                self.write8(addr, self.a());
                12
            }
            0xF0 => {
                // LDH A, (a8)
                let addr = 0xFF00 | self.fetch8() as u16;
                let value = self.read8(addr);
                self.set_a(value);
                12
            }
            0xE2 => {
                // LD (C), A  -- i.e. (0xFF00 + C)
                let addr = 0xFF00 | (self.registers.bc & 0xFF);
                self.write8(addr, self.a());
                8
            }
            0xF2 => {
                // LD A, (C)
                let addr = 0xFF00 | (self.registers.bc & 0xFF);
                let value = self.read8(addr);
                self.set_a(value);
                8
            }
            0xEA => {
                // LD (a16), A
                let addr = self.fetch16();
                self.write8(addr, self.a());
                16
            }
            0xFA => {
                // LD A, (a16)
                let addr = self.fetch16();
                let value = self.read8(addr);
                self.set_a(value);
                16
            }

            // ---- 16-bit loads ------------------------------------------------------
            0x01 | 0x11 | 0x21 | 0x31 => {
                // LD rr, n16
                let value = self.fetch16();
                self.set_rp(p, value);
                12
            }
            0x08 => {
                // LD (a16), SP
                let addr = self.fetch16();
                self.write16(addr, self.registers.sp);
                20
            }
            0xF8 => {
                // LD HL, SP + e8
                let offset = self.fetch8();
                self.registers.hl = self.sp_plus_e8(offset);
                12
            }
            0xF9 => {
                // LD SP, HL
                self.registers.sp = self.registers.hl;
                8
            }
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                // POP rr
                let value = self.pop16();
                self.set_rp2(p, value);
                12
            }
            0xC5 | 0xD5 | 0xE5 | 0xF5 => {
                // PUSH rr
                self.push16(self.get_rp2(p));
                16
            }

            // ---- 8-bit arithmetic / logic ------------------------------------------
            0x80..=0xBF => {
                // ADD/ADC/SUB/SBC/AND/XOR/OR/CP A, r
                let value = self.get_r8(z);
                self.alu(y, value);
                if z == 6 {
                    8
                } else {
                    4
                }
            }
            0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => {
                // ALU A, n8
                let value = self.fetch8();
                self.alu(y, value);
                8
            }
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                // INC r
                let value = self.get_r8(y);
                let result = self.inc8(value);
                self.set_r8(y, result);
                if y == 6 {
                    12
                } else {
                    4
                }
            }
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                // DEC r
                let value = self.get_r8(y);
                let result = self.dec8(value);
                self.set_r8(y, result);
                if y == 6 {
                    12
                } else {
                    4
                }
            }
            0x27 => {
                self.daa();
                4
            }
            0x2F => {
                // CPL
                self.set_a(!self.a());
                self.registers.flag.n = true;
                self.registers.flag.h = true;
                4
            }
            0x37 => {
                // SCF
                let z = self.registers.flag.z;
                self.registers.set_flags(z, false, false, true);
                4
            }
            0x3F => {
                // CCF
                let flag = &self.registers.flag;
                let (z, c) = (flag.z, flag.c);
                self.registers.set_flags(z, false, false, !c);
                4
            }

            // ---- 16-bit arithmetic (INC/DEC rr leave the flags alone) --------------
            0x03 | 0x13 | 0x23 | 0x33 => {
                self.set_rp(p, self.get_rp(p).wrapping_add(1));
                8
            }
            0x0B | 0x1B | 0x2B | 0x3B => {
                self.set_rp(p, self.get_rp(p).wrapping_sub(1));
                8
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                // ADD HL, rr
                self.add_hl(self.get_rp(p));
                8
            }
            0xE8 => {
                // ADD SP, e8
                let offset = self.fetch8();
                self.registers.sp = self.sp_plus_e8(offset);
                16
            }

            // ---- Rotates on A (always clear Z, unlike their CB versions) -----------
            0x07 | 0x0F | 0x17 | 0x1F => {
                // RLCA / RRCA / RLA / RRA share the CB shift implementation (y = 0..3)
                let result = self.shift(y, self.a());
                self.set_a(result);
                self.registers.flag.z = false;
                4
            }

            // ---- Jumps -------------------------------------------------------------
            0xC3 => {
                self.registers.pc = self.fetch16();
                16
            }
            0xE9 => {
                // JP HL
                self.registers.pc = self.registers.hl;
                4
            }
            0xC2 | 0xCA | 0xD2 | 0xDA => {
                // JP cc, a16
                let addr = self.fetch16();
                if self.condition(y) {
                    self.registers.pc = addr;
                    16
                } else {
                    12
                }
            }
            0x18 => {
                // JR e8
                let offset = self.fetch8() as i8;
                self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                12
            }
            0x20 | 0x28 | 0x30 | 0x38 => {
                // JR cc, e8  (y = 4..7 maps to cc = 0..3)
                let offset = self.fetch8() as i8;
                if self.condition(y) {
                    self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);
                    12
                } else {
                    8
                }
            }

            // ---- Calls and returns -------------------------------------------------
            0xCD => {
                let addr = self.fetch16();
                self.push16(self.registers.pc);
                self.registers.pc = addr;
                24
            }
            0xC4 | 0xCC | 0xD4 | 0xDC => {
                // CALL cc, a16
                let addr = self.fetch16();
                if self.condition(y) {
                    self.push16(self.registers.pc);
                    self.registers.pc = addr;
                    24
                } else {
                    12
                }
            }
            0xC9 => {
                self.registers.pc = self.pop16();
                16
            }
            0xD9 => {
                // RETI: return and enable interrupts immediately (no EI delay)
                self.registers.pc = self.pop16();
                self.ime = true;
                16
            }
            0xC0 | 0xC8 | 0xD0 | 0xD8 => {
                // RET cc
                if self.condition(y) {
                    self.registers.pc = self.pop16();
                    20
                } else {
                    8
                }
            }
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                // RST vec  (vector = y * 8)
                self.push16(self.registers.pc);
                self.registers.pc = (y as u16) * 8;
                16
            }

            // ---- Interrupt control -------------------------------------------------
            0xF3 => {
                // DI (also cancels an EI that hasn't taken effect yet)
                self.ime = false;
                self.ei_delay = 0;
                4
            }
            0xFB => {
                // EI: IME is set after the next instruction. `step` counts this down
                // once at the end of this step and once after the next one.
                if !self.ime {
                    self.ei_delay = 2;
                }
                4
            }

            0xCB => self.execute_cb(),

            0xD3 | 0xDB | 0xDD | 0xE3 | 0xE4 | 0xEB | 0xEC | 0xED | 0xF4 | 0xFC | 0xFD => {
                // These lock up a real Game Boy. Reaching one means PC went somewhere bad.
                panic!(
                    "illegal opcode {op:#04X} at {:#06X}",
                    self.registers.pc.wrapping_sub(1)
                )
            }
        }
    }

    /// Executes the CB-prefixed instruction that follows the 0xCB byte.
    /// Returned cycles include the prefix.
    fn execute_cb(&mut self) -> u8 {
        let op = self.fetch8();
        let y = (op >> 3) & 7;
        let z = op & 7;
        let value = self.get_r8(z);
        let on_hl = z == 6;

        match op >> 6 {
            0 => {
                // RLC/RRC/RL/RR/SLA/SRA/SWAP/SRL r
                let result = self.shift(y, value);
                self.set_r8(z, result);
                if on_hl {
                    16
                } else {
                    8
                }
            }
            1 => {
                // BIT y, r: Z = bit is clear, C untouched
                let c = self.registers.flag.c;
                self.registers
                    .set_flags(value & (1 << y) == 0, false, true, c);
                if on_hl {
                    12
                } else {
                    8
                }
            }
            2 => {
                // RES y, r
                self.set_r8(z, value & !(1 << y));
                if on_hl {
                    16
                } else {
                    8
                }
            }
            _ => {
                // SET y, r
                self.set_r8(z, value | (1 << y));
                if on_hl {
                    16
                } else {
                    8
                }
            }
        }
    }
}
