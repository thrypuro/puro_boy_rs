mod alu;
mod execute;
mod registers;
mod shared;

use crate::MMU;
use shared::{RegisterNames, Registers};

const IF_ADDR: u16 = 0xFF0F;
const IE_ADDR: u16 = 0xFFFF;

/// 8-bit register operand encoding used by the opcode table:
/// 0=B 1=C 2=D 3=E 4=H 5=L 6=(HL) 7=A. Index 6 is memory and never looked up here.
const R8: [RegisterNames; 8] = [
    RegisterNames::B,
    RegisterNames::C,
    RegisterNames::D,
    RegisterNames::E,
    RegisterNames::H,
    RegisterNames::L,
    RegisterNames::HL,
    RegisterNames::A,
];

pub struct CPU<'a> {
    registers: Registers,
    pub memory: &'a mut MMU,
    halted: bool,
    /// Interrupt Master Enable flag.
    ime: bool,
    /// EI only sets IME after the *following* instruction. This counts the steps left
    /// until that happens (0 = nothing pending).
    ei_delay: u8,
}

impl<'a> CPU<'a> {
    /// Creates a CPU in the state the DMG boot ROM leaves it in (PC = 0x0100).
    pub fn new(mmu: &'a mut MMU) -> Self {
        Self {
            registers: Registers::new(),
            memory: mmu,
            halted: false,
            ime: false,
            ei_delay: 0,
        }
    }

    /// Services one pending interrupt, or executes one instruction, and returns the
    /// number of T-cycles it took (NOP = 4).
    pub fn step(&mut self) -> u8 {
        let pending = self.pending_interrupts();

        if self.halted {
            if pending == 0 {
                return 4;
            }
            // Any pending interrupt wakes the CPU, even with IME clear.
            self.halted = false;
        }

        if self.ime && pending != 0 {
            return self.service_interrupt(pending);
        }

        let opcode = self.fetch8();
        log::trace!("{:04X}: {:02X}", self.registers.pc.wrapping_sub(1), opcode);
        let cycles = self.execute(opcode);

        if self.ei_delay > 0 {
            self.ei_delay -= 1;
            if self.ei_delay == 0 {
                self.ime = true;
            }
        }
        cycles
    }

    /// Interrupts that are both requested (IF) and enabled (IE).
    fn pending_interrupts(&mut self) -> u8 {
        self.read8(IE_ADDR) & self.read8(IF_ADDR) & 0x1F
    }

    /// Jumps to the vector of the highest-priority (lowest bit) pending interrupt.
    fn service_interrupt(&mut self, pending: u8) -> u8 {
        let bit = pending.trailing_zeros() as u8;
        self.ime = false;
        let requested = self.read8(IF_ADDR);
        self.write8(IF_ADDR, requested & !(1 << bit));
        self.push16(self.registers.pc);
        self.registers.pc = 0x0040 + 8 * bit as u16;
        20
    }

    // ---- Memory access -------------------------------------------------------------

    /// Reads a byte. Addresses the MMU rejects (echo RAM, the unusable region) read as
    /// 0xFF rather than crashing the emulator.
    fn read8(&mut self, addr: u16) -> u8 {
        self.memory.read(addr).unwrap_or_else(|e| {
            log::warn!("read {addr:#06X}: {e:?}");
            0xFF
        })
    }

    fn write8(&mut self, addr: u16, value: u8) {
        if let Err(e) = self.memory.write(addr, value) {
            log::warn!("write {addr:#06X} <- {value:#04X}: {e:?}");
        }
    }

    fn write16(&mut self, addr: u16, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.write8(addr, low);
        self.write8(addr.wrapping_add(1), high);
    }

    /// Reads the byte at PC and advances PC.
    fn fetch8(&mut self) -> u8 {
        let byte = self.read8(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }

    /// Reads the little-endian word at PC and advances PC by 2.
    fn fetch16(&mut self) -> u16 {
        let low = self.fetch8();
        let high = self.fetch8();
        u16::from_le_bytes([low, high])
    }

    fn push16(&mut self, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.write16(self.registers.sp, value);
    }

    fn pop16(&mut self) -> u16 {
        let sp = self.registers.sp;
        let low = self.read8(sp);
        let high = self.read8(sp.wrapping_add(1));
        self.registers.sp = sp.wrapping_add(2);
        u16::from_le_bytes([low, high])
    }

    // ---- Register operands ---------------------------------------------------------

    fn a(&self) -> u8 {
        self.registers.get_register_value_8(RegisterNames::A)
    }

    fn set_a(&mut self, value: u8) {
        self.registers.set_register_value_8(RegisterNames::A, value);
    }

    /// Reads an 8-bit operand by its 3-bit index (see [`R8`]); 6 reads (HL).
    fn get_r8(&mut self, idx: u8) -> u8 {
        match idx & 7 {
            6 => self.read8(self.registers.hl),
            i => self.registers.get_register_value_8(R8[i as usize]),
        }
    }

    /// Writes an 8-bit operand by its 3-bit index (see [`R8`]); 6 writes (HL).
    fn set_r8(&mut self, idx: u8, value: u8) {
        match idx & 7 {
            6 => self.write8(self.registers.hl, value),
            i => self.registers.set_register_value_8(R8[i as usize], value),
        }
    }

    /// 16-bit register pair by its 2-bit index: 0=BC 1=DE 2=HL 3=SP.
    fn get_rp(&self, idx: u8) -> u16 {
        match idx & 3 {
            0 => self.registers.bc,
            1 => self.registers.de,
            2 => self.registers.hl,
            _ => self.registers.sp,
        }
    }

    fn set_rp(&mut self, idx: u8, value: u16) {
        match idx & 3 {
            0 => self.registers.bc = value,
            1 => self.registers.de = value,
            2 => self.registers.hl = value,
            _ => self.registers.sp = value,
        }
    }

    /// Register pair for PUSH/POP by its 2-bit index: 0=BC 1=DE 2=HL 3=AF.
    fn get_rp2(&self, idx: u8) -> u16 {
        match idx & 3 {
            3 => self.registers.get_register_value_16(RegisterNames::AF),
            i => self.get_rp(i),
        }
    }

    fn set_rp2(&mut self, idx: u8, value: u16) {
        match idx & 3 {
            3 => self
                .registers
                .set_register_value_16(RegisterNames::AF, value),
            i => self.set_rp(i, value),
        }
    }

    /// Branch condition by its 2-bit index: 0=NZ 1=Z 2=NC 3=C.
    fn condition(&self, idx: u8) -> bool {
        let flag = &self.registers.flag;
        match idx & 3 {
            0 => !flag.z,
            1 => flag.z,
            2 => !flag.c,
            _ => flag.c,
        }
    }

    pub fn print_registers(&self) {
        let r = &self.registers;
        println!(
            "AF: {:04X}  BC: {:04X}  DE: {:04X}  HL: {:04X}  SP: {:04X}  PC: {:04X}  IME: {}",
            r.get_register_value_16(RegisterNames::AF),
            r.bc,
            r.de,
            r.hl,
            r.sp,
            r.pc,
            self.ime as u8
        );
    }
}

#[cfg(test)]
mod tests;
