use crate::gb::instructions::*;
use crate::gb::registers::{Registers, RegisterNames};
use crate::gb::mmu::MMU;
use crate::gb::opcodes::OPCODES;
use super::registers;

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
    
    // POP 
    fn pop(&mut self) -> u16 {
        let low = self.memory.read(self.sp);
        let high = self.memory.read(self.sp + 1);
        self.sp += 2;
        ((high as u16) << 8) | (low as u16)
    }
    
    // PUSH 
    fn push(&mut self, value: u16) {
        self.memory.write(self.sp, (value & 0x00FF) as u8);
        self.memory.write(self.sp + 1, ((value >> 8) & 0x00FF) as u8);
    }

    // readw word from rom
    fn read_word(&mut self) -> u16 {
        let low = self.memory.read(self.pc);
        let high = self.memory.read(self.pc + 1);
        self.pc += 2;
        ((high as u16) << 8) | (low as u16)
    }

    // read byte from rom
    fn read_byte(&mut self) -> u8 {
        let byte = self.memory.read(self.pc);
        self.pc += 1;
        byte
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

    fn execute_instruction2(&mut self, opcode : u8) {
        match opcode {
            0x00 => {
                // NOP
            },
            0x01 => {
                // LD BC, d16
                let operand1 = Operand::Register(RegisterNames::BC);
                let operand2 = Operand::Immediate16(self.read_word());
                ld_16bit(&mut self.registers, operand1, operand2);
            },
            _ => {
                panic!("Unknown opcode: {:#04X}", opcode);
            }
        }
    }



    // fn execute_instruction(&mut self, opcode : u8) {
    //     match opcode {

    //         // 0x00 to 0x0F
    //         0x00 => {
    //             // NOP    
    //         },
    //         0x01 => {
    //             // LD BC, d16
    //             let operand1  = Operand::Register(RegisterNames::BC);
    //             let operand2 = Operand::Immediate16(self.read_word());
    //             ld_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x02 => {
    //             // LD (BC), A
    //             let operand1 = Operand::Memory(self.registers.bc);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x03 => {
    //             // INC BC
    //             let operand1 = Operand::Register(RegisterNames::BC);
    //             let operand2 = Operand::Immediate(1);
    //             add_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x04 => {
    //             // INC B
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x05 => {
    //             // DEC B
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x06 => {
    //             // LD B, d8
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x07 => {
    //             // RLCA
    //             let operand1 = Operand::Register(RegisterNames::A); 
    //             rlc(&mut self.registers,&mut self.memory, operand1);
    //         },
    //         0x08 => {
    //             // LD (a16), SP
    //             self.pc += 1;
    //             let operand1 = Operand::Memory(self.pc);
    //             let operand2 = Operand::Memory(self.sp );
    //             ld_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x09 => {
    //             // ADD HL, BC
    //             let operand1 = Operand::Register(RegisterNames::HL);
    //             let operand2 = Operand::Register(RegisterNames::BC);
    //             add_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x0A => {
    //             // LD A, (BC)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.bc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x0B => {
    //             // DEC BC
    //             let operand1 = Operand::Register(RegisterNames::BC);
    //             let operand2 = Operand::Immediate(1);
    //             sub_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x0C => {
    //             // INC C
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x0D => {
    //             // DEC C
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x0E => {
    //             // LD C, d8
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x0F => {
    //             // RRCA 
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             rrc(&mut self.registers,&mut self.memory, operand1);
    //         },

    //         // 0x10 to 0x1F
    //         0x10 => {
    //             // STOP
    //             self.halted = true;
    //         },
    //         0x11 => {
    //             // LD DE, d16
    //             let operand1 = Operand::Register(RegisterNames::DE);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x12 => {
    //             // LD (DE), A
    //             let operand1 = Operand::Memory(self.registers.de);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x13 => {
    //             // INC DE
    //             let operand1 = Operand::Register(RegisterNames::DE);
    //             let operand2 = Operand::Immediate(1);
    //             add_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x14 => {
    //             // INC D
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x15 => {
    //             // DEC D
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x16 => {
    //             // LD D, d8
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x17 => {
    //             // RLA
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             rl(&mut self.registers,&mut self.memory, operand1);
    //         },
    //         0x18 => {
    //             // JR r8
    //             self.pc += 1;
    //             let operand = Operand::Memory(self.pc);
    //             jr(&mut self.registers, &mut self.memory, &mut self.pc, operand);
    //         },
    //         0x19 => {
    //             // ADD HL, DE
    //             let operand1 = Operand::Register(RegisterNames::HL);
    //             let operand2 = Operand::Register(RegisterNames::DE);
    //             add_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x1A => {
    //             // LD A, (DE)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.de);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x1B => {
    //             // DEC DE
    //             let operand1 = Operand::Register(RegisterNames::DE);
    //             let operand2 = Operand::Immediate(1);
    //             sub_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x1C => {
    //             // INC E
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x1D => {
    //             // DEC E
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x1E => {
    //             // LD E, d8
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x1F => {
    //             // RRA
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             rr(&mut self.registers,&mut self.memory, operand1);
    //         },

    //         // 0x20 to 0x2F
    //         0x20 => {
    //             // JR NZ, r8
    //             self.pc += 1;
    //             let cond = self.registers.get_zero_flag();
    //             let operand = Operand::Memory(self.pc);
    //             jr_conditional(&mut self.registers, &mut self.memory, &mut self.pc, cond, operand);
    //         },
    //         0x21 => {
    //             // LD HL, d16
    //             let operand1 = Operand::Register(RegisterNames::HL);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_16bit(&mut self.registers, operand1, operand2);
    //         },

    //         0x22 => {
    //             // LD (HL+), A
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x23 => {
    //             // INC HL
    //             let operand1 = Operand::Register(RegisterNames::HL);
    //             let operand2 = Operand::Immediate(1);
    //             add_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x24 => {
    //             // INC H
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x25 => {
    //             // DEC H
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x26 => {
    //             // LD H, d8
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x27 => {
    //             // DAA
              
    //             daa(&mut self.registers);
    //         },
    //         0x28 => {
    //             // JR Z, r8
    //             let cond = self.registers.get_zero_flag();
    //             let operand = Operand::Immediate(self.memory.read_rom(self.pc));    
    //             jr_conditional(&mut self.registers, &mut self.memory, &mut self.pc, cond, operand);
    //         },
    //         0x29 => {
    //             // ADD HL, HL
    //             let operand1 = Operand::Register(RegisterNames::HL);
    //             let operand2 = Operand::Register(RegisterNames::HL);
    //             add_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x2A => {
    //             // LD A, (HL+)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x2B => {
    //             // DEC HL
    //             let operand1 = Operand::Register(RegisterNames::HL);
    //             let operand2 = Operand::Immediate(1);
    //             sub_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x2C => {
    //             // INC L
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x2D => {
    //             // DEC L
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x2E => {
    //             // LD L, d8
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x2F => {
    //             // CPL
    //             cpl(&mut self.registers);
    //         },

    //         // 0x30 to 0x3F
    //         0x30 => {
    //             // JR NC, r8
    //             self.pc += 1;
    //             let cond = self.registers.get_carry_flag();
    //             let operand = Operand::Memory(self.pc);
    //             jr_conditional(&mut self.registers, &mut self.memory, &mut self.pc, cond, operand);
    //         },
    //         0x31 => {
    //             // LD SP, d16
    //             self.pc += 1;
    //             let operand2 = MMU::read_word(&mut self.memory, self.pc);
    //             self.sp = operand2;

    //         },
    //         0x32 => {
    //             // LD (HL-), A
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x33 => {
    //             // INC SP
    //             self.sp = self.sp.wrapping_add(1);
    //         },
    //         0x34 => {
    //             // INC (HL)
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x35 => {
    //             // DEC (HL)
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x36 => {
    //             // LD (HL), d8
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
            
    //         0x37 => {
    //             // SCF
    //             scf(&mut self.registers);
    //         },
    //         0x38 => {
    //             // JR C, r8
    //             self.pc += 1;
    //             let cond = self.registers.get_carry_flag();
    //             let operand = Operand::Memory(self.pc);
    //             jr_conditional(&mut self.registers, &mut self.memory, &mut self.pc, cond, operand);
    //         },
    //         0x39 => {
    //             // ADD HL, SP
    //             let operand1 = Operand::Register(RegisterNames::HL);
    //             let operand2 = Operand::Immediate16(self.sp);
    //             add_16bit(&mut self.registers, operand1, operand2);
    //         },
    //         0x3A => {
    //             // LD A, (HL-)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x3B => {
    //             // DEC SP
    //             self.sp = self.sp.wrapping_sub(1);
    //         },
    //         0x3C => {
    //             // INC A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Immediate(1);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x3D => {
    //             // DEC A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Immediate(1);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x3E => {
    //             // LD A, d8
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             self.pc += 1;
    //             let operand2 = Operand::Memory(self.pc);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x3F => {
    //             // CCF
    //             ccf(&mut self.registers);
    //         },

    //         // 0x40 to 0x4F
    //         0x40 => {
    //             // LD B, B
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x41 => {
    //             // LD B, C
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x42 => {
    //             // LD B, D
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x43 => {
    //             // LD B, E
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x44 => {
    //             // LD B, H
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x45 => {
    //             // LD B, L
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x46 => {
    //             // LD B, (HL)
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x47 => {
    //             // LD B, A
    //             let operand1 = Operand::Register(RegisterNames::B);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x48 => {
    //             // LD C, B
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x49 => {
    //             // LD C, C
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x4A => {
    //             // LD C, D
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x4B => {
    //             // LD C, E
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x4C => {
    //             // LD C, H
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x4D => {
    //             // LD C, L
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x4E => {
    //             // LD C, (HL)
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x4F => {
    //             // LD C, A
    //             let operand1 = Operand::Register(RegisterNames::C);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0x50 to 0x5F
    //         0x50 => {
    //             // LD D, B
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x51 => {
    //             // LD D, C
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x52 => {
    //             // LD D, D
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x53 => {
    //             // LD D, E
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x54 => {
    //             // LD D, H
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x55 => {
    //             // LD D, L
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x56 => {
    //             // LD D, (HL)
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x57 => {
    //             // LD D, A
    //             let operand1 = Operand::Register(RegisterNames::D);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x58 => {
    //             // LD E, B
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x59 => {
    //             // LD E, C
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x5A => {
    //             // LD E, D
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x5B => {
    //             // LD E, E
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x5C => {
    //             // LD E, H
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x5D => {
    //             // LD E, L
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x5E => {
    //             // LD E, (HL)
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x5F => {
    //             // LD E, A
    //             let operand1 = Operand::Register(RegisterNames::E);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0x60 to 0x6F
    //         0x60 => {
    //             // LD H, B
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x61 => {
    //             // LD H, C
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x62 => {
    //             // LD H, D
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x63 => {
    //             // LD H, E
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x64 => {
    //             // LD H, H
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x65 => {
    //             // LD H, L
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x66 => {
    //             // LD H, (HL)
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x67 => {
    //             // LD H, A
    //             let operand1 = Operand::Register(RegisterNames::H);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x68 => {
    //             // LD L, B
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x69 => {
    //             // LD L, C
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x6A => {
    //             // LD L, D
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x6B => {
    //             // LD L, E
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x6C => {
    //             // LD L, H
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x6D => {
    //             // LD L, L
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x6E => {
    //             // LD L, (HL)
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x6F => {
    //             // LD L, A
    //             let operand1 = Operand::Register(RegisterNames::L);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0x70 to 0x7F
    //         0x70 => {
    //             // LD (HL), B
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x71 => {
    //             // LD (HL), C
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x72 => {
    //             // LD (HL), D
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x73 => {
    //             // LD (HL), E
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x74 => {
    //             // LD (HL), H
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x75 => {
    //             // LD (HL), L
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x76 => {
    //             // HALT
    //             self.halted = true;
    //         },

    //         0x77 => {
    //             // LD (HL), A
    //             let operand1 = Operand::Memory(self.registers.hl);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x78 => {
    //             // LD A, B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x79 => {
    //             // LD A, C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x7A => {
    //             // LD A, D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x7B => {
    //             // LD A, E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x7C => {
    //             // LD A, H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x7D => {
    //             // LD A, L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x7E => {
    //             // LD A, (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x7F => {
    //             // LD A, A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0x80 to 0x8F
    //         0x80 => {
    //             // ADD A, B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x81 => {
    //             // ADD A, C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x82 => {
    //             // ADD A, D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x83 => {
    //             // ADD A, E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x84 => {
    //             // ADD A, H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x85 => {
    //             // ADD A, L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x86 => {
    //             // ADD A, (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x87 => {
    //             // ADD A, A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x88 => {
    //             // ADC A, B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x89 => {
    //             // ADC A, C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x8A => {
    //             // ADC A, D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x8B => {
    //             // ADC A, E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x8C => {
    //             // ADC A, H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x8D => {
    //             // ADC A, L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x8E => {
    //             // ADC A, (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x8F => {
    //             // ADC A, A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0x90 to 0x9F
    //         0x90 => {
    //             // SUB B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x91 => {
    //             // SUB C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x92 => {
    //             // SUB D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x93 => {
    //             // SUB E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x94 => {
    //             // SUB H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x95 => {
    //             // SUB L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x96 => {
    //             // SUB (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x97 => {
    //             // SUB A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x98 => {
    //             // SBC A, B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x99 => {
    //             // SBC A, C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x9A => {
    //             // SBC A, D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x9B => {
    //             // SBC A, E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x9C => {
    //             // SBC A, H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x9D => {
    //             // SBC A, L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x9E => {
    //             // SBC A, (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0x9F => {
    //             // SBC A, A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             sbc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0xA0 to 0xAF
    //         0xA0 => {
    //             // AND B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA1 => {
    //             // AND C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA2 => {
    //             // AND D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA3 => {
    //             // AND E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA4 => {
    //             // AND H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA5 => {
    //             // AND L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA6 => {
    //             // AND (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA7 => {
    //             // AND A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA8 => {
    //             // XOR B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xA9 => {
    //             // XOR C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xAA => {
    //             // XOR D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xAB => {
    //             // XOR E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xAC => {
    //             // XOR H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xAD => {
    //             // XOR L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xAE => {
    //             // XOR (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xAF => {
    //             // XOR A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0xB0 to 0xBF
    //         0xB0 => {
    //             // OR B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB1 => {
    //             // OR C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB2 => {
    //             // OR D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB3 => {
    //             // OR E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB4 => {
    //             // OR H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB5 => {
    //             // OR L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB6 => {
    //             // OR (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB7 => {
    //             // OR A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB8 => {
    //             // CP B
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::B);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xB9 => {
    //             // CP C
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::C);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xBA => {
    //             // CP D
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::D);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xBB => {
    //             // CP E
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::E);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xBC => {
    //             // CP H
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::H);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xBD => {
    //             // CP L
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::L);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xBE => {
    //             // CP (HL)
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Memory(self.registers.hl);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xBF => {
    //             // CP A
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Register(RegisterNames::A);
    //             cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },

    //         // 0xC0 to 0xCF
    //         0xC0 => {
    //             // RET NZ
    //             let condition = self.registers.get_zero_flag();
    //             ret(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, condition);
    //         },
    //         0xC1 => {
    //             // POP BC
    //             self.registers.bc = self.pop();
    //         },
    //         0xC2 => {
    //             // JP NZ, a16
    //             let condition = self.registers.get_zero_flag();
    //             let address = self.memory.read_word(self.pc);
    //             jp(&mut self.registers, &mut self.memory, &mut self.pc, address, condition);
    //         },
    //         0xC3 => {
    //             // JP a16
    //             let address = self.memory.read_word(self.pc);
    //             let operand = Operand::Immediate16(address);
    //             jp(&mut self.registers, &mut self.memory, &mut self.pc, operand,  true);
    //         },
    //         0xC4 => {
    //             // CALL NZ, a16
    //             let condition = self.registers.get_zero_flag();
    //             let address = self.memory.read_word(self.pc);
    //             call(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, address, condition);
    //         },
    //         0xC5 => {
    //             // PUSH BC
    //             self.push(self.registers.bc);
    //         },
    //         0xC6 => {
    //             // ADD A, d8
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Immediate(self.memory.read(self.pc));
    //             add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xC7 => {
    //             // RST 00H
    //             let address = 0x0000;
    //             // rst(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, address);
    //         },
    //         0xC8 => {
    //             // RET Z
    //             let condition = self.registers.get_zero_flag();
    //             ret(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, condition);
    //         },
    //         0xC9 => {
    //             // RET
    //             ret(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, true);
    //         },
    //         0xCA => {
    //             // JP Z, a16
    //             let condition = self.registers.get_zero_flag();
    //             let address = self.memory.read_word(self.pc);
    //             jp(&mut self.registers, &mut self.memory, &mut self.pc, address, condition);
    //         },
    //         0xCB => {
    //             // CB prefix
    //             // let cb_opcode = self.memory.read(self.pc);
    //             // self.execute_cb_opcode(cb_opcode);
    //         },
    //         0xCC => {
    //             // CALL Z, a16
    //             let condition = self.registers.get_zero_flag();
    //             let address = self.memory.read_word(self.pc);
    //             let operand1 = Operand::Immediate16(address);
    //             call(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, operand1, condition);
    //         },
    //         0xCD => {
    //             // CALL a16
    //             let address = self.memory.read_word(self.pc);
    //             let operand1 = Operand::Immediate16(address);
    //             call(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, operand1, true);
    //         },
    //         0xCE => {
    //             // ADC A, d8
    //             let operand1 = Operand::Register(RegisterNames::A);
    //             let operand2 = Operand::Immediate(self.memory.read(self.pc));
    //             adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
    //         },
    //         0xCF => {
    //             // RST 08H
    //             let address = 0x0008;
    //             // rst(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, address);
    //         },

    //         // 0xD0 to 0xDF
    //         0xD0 => {
    //             // RET NC
    //             let condition = self.registers.get_carry_flag();
    //             ret(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, condition);
    //         },
    //         0xD1 => {
    //             // POP DE 
    //             self.registers.de = self.pop(&mut self.registers, &mut self.memory, &mut self.sp);
    //         },
    //         0xD2 => {
    //             // JP NC, a16
    //             let condition = self.registers.get_carry_flag();
    //             let address = self.memory.read_word(self.pc);
    //             jp(&mut self.registers, &mut self.memory, &mut self.pc, address, condition);
    //         },
    //         0xD3 => {
    //             // OUT (C), A
    //             let port = self.registers.c;
    //             let value = self.registers.a;
    //             out(&mut self.registers, &mut self.memory, port, value);
    //         },
    //         0xD4 => {
    //             // CALL NC, a16
    //             let condition = self.registers.get_carry_flag();
    //             let address = self.memory.read_word(self.pc);
    //             call(&mut self.registers, &mut self.memory, &mut self.pc, &mut self.sp, address, condition);
    //         },
    //         0xD5 => {
    //             // PUSH DE
    //             push(&mut self.registers, &mut self.memory, &mut self.sp, self.registers.de);
    //         },


    //         _ => {
    //             // Handle other opcodes
    //             println!("Unknown opcode: {:#04x}", opcode);
    //         },
    //     }


    // }
   
}

#[cfg(test)]
mod tests {
    use super::*;

}