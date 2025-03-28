use crate::gb::instructions::*;
use crate::gb::registers::{Registers, RegisterNames};
use crate::gb::mmu::MMU;

const deb : bool = false;

pub struct CPU {
    registers: Registers,
    memory: MMU,
    sp : u16,
    pc : u16,
    halted : bool,
}

impl CPU {
    /// Creates a new CPU instance with the given ROM data.
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            registers: Registers::new(),
            memory: MMU::new(rom),
            sp : 0xFFFE,
            pc : 0x0,
            halted : false,
        }
    }
    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        // Fetch the opcode from memory
        let opcode = self.memory.read(self.pc);
        self.pc += 1;

        // Execute the instruction
        self.execute_instruction(opcode);

        // Update the program counter
        self.pc += 1;
    }

    fn execute_instruction(&mut self, opcode : u8) {
        match opcode {
            0x00 => {
                // NOP    
            },
            0x01 => {
                // LD BC, d16
                let operand1  = Operand::Register(RegisterNames::BC);
                self.pc += 1;
                let operand2 = Operand::Memory(self.pc);
                ld_16bit(&mut self.registers, operand1, operand2);
            },
            0x02 => {
                // LD (BC), A
                let operand1 = Operand::Memory(self.registers.bc);
                let operand2 = Operand::Register(RegisterNames::A);
                ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x03 => {
                // INC BC
                let operand1 = Operand::Register(RegisterNames::BC);
                let operand2 = Operand::Immediate(1);
                add_16bit(&mut self.registers, operand1, operand2);
            },
            0x04 => {
                // INC B
                let operand1 = Operand::Register(RegisterNames::B);
                let operand2 = Operand::Immediate(1);
                add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x05 => {
                // DEC B
                let operand1 = Operand::Register(RegisterNames::B);
                let operand2 = Operand::Immediate(1);
                sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x06 => {
                // LD B, d8
                let operand1 = Operand::Register(RegisterNames::B);
                self.pc += 1;
                let operand2 = Operand::Memory(self.pc);
                ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x07 => {
                // RLCA (not done)
            },
            0x08 => {
                // LD (a16), SP
                self.pc += 1;
                let operand1 = Operand::Memory(self.pc);
                let operand2 = Operand::Memory(self.sp );
                ld_16bit(&mut self.registers, operand1, operand2);
            },
            0x09 => {
                // ADD HL, BC
                let operand1 = Operand::Register(RegisterNames::HL);
                let operand2 = Operand::Register(RegisterNames::BC);
                add_16bit(&mut self.registers, operand1, operand2);
            },
            0x0A => {
                // LD A, (BC)
                let operand1 = Operand::Register(RegisterNames::A);
                let operand2 = Operand::Memory(self.registers.bc);
                ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x0B => {
                // DEC BC
                let operand1 = Operand::Register(RegisterNames::BC);
                let operand2 = Operand::Immediate(1);
                sub_16bit(&mut self.registers, operand1, operand2);
            },
            0x0C => {
                // INC C
                let operand1 = Operand::Register(RegisterNames::C);
                let operand2 = Operand::Immediate(1);
                add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x0D => {
                // DEC C
                let operand1 = Operand::Register(RegisterNames::C);
                let operand2 = Operand::Immediate(1);
                sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x0E => {
                // LD C, d8
                let operand1 = Operand::Register(RegisterNames::C);
                self.pc += 1;
                let operand2 = Operand::Memory(self.pc);
                ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x0F => {
                // RRCA (not done)
            },
            0x10 => {
                // STOP
                self.halted = true;
            },
            0x11 => {
                // LD DE, d16
                let operand1 = Operand::Register(RegisterNames::DE);
                self.pc += 1;
                let operand2 = Operand::Memory(self.pc);
                ld_16bit(&mut self.registers, operand1, operand2);
            },
            0x12 => {
                // LD (DE), A
                let operand1 = Operand::Memory(self.registers.de);
                let operand2 = Operand::Register(RegisterNames::A);
                ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x13 => {
                // INC DE
                let operand1 = Operand::Register(RegisterNames::DE);
                let operand2 = Operand::Immediate(1);
                add_16bit(&mut self.registers, operand1, operand2);
            },
            0x14 => {
                // INC D
                let operand1 = Operand::Register(RegisterNames::D);
                let operand2 = Operand::Immediate(1);
                add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x15 => {
                // DEC D
                let operand1 = Operand::Register(RegisterNames::D);
                let operand2 = Operand::Immediate(1);
                sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x16 => {
                // LD D, d8
                let operand1 = Operand::Register(RegisterNames::D);
                self.pc += 1;
                let operand2 = Operand::Memory(self.pc);
                ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
            },
            0x17 => {
                // RLA (not done)
            },
            

            _ => {
                // Handle other opcodes
                println!("Unknown opcode: {:#04x}", opcode);
            },
        }


    }
   
}

#[cfg(test)]
mod tests {
    use super::*;

}