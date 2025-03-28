use crate::gb::instructions::*;
use crate::gb::registers::{Registers, RegisterNames,match_string_to_register,get_register_bit_length};
use crate::gb::mmu::MMU;
use json;
use super::registers;
use crate::gb::opcodes::{get_opcodes, Instruction, match_string_to_instruction};



const deb : bool = true;

pub struct CPU {
    registers: Registers,
    memory: MMU,

    halted : bool,
    opcodes : json::JsonValue,
}

impl CPU {
    /// Creates a new CPU instance with the given ROM data.
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            registers: Registers::new(),
            memory: MMU::new(rom),
            halted : false,
            opcodes : get_opcodes(),
        }
    }
    
    // POP 
    fn pop(&mut self) -> u16 {
        let low = self.memory.read(self.registers.sp);
        let high = self.memory.read(self.registers.sp + 1);
        self.registers.sp += 2;
        ((high as u16) << 8) | (low as u16)
    }
    
    // PUSH 
    fn push(&mut self, value: u16) {
        self.memory.write(self.registers.sp, (value & 0x00FF) as u8);
        self.memory.write(self.registers.sp + 1, ((value >> 8) & 0x00FF) as u8);
    }

    // readw word from rom
    fn read_word(&mut self) -> u16 {
        let low = self.memory.read_rom(self.registers.pc);
        let high = self.memory.read_rom(self.registers.pc + 1);
        self.registers.pc += 2;
        ((high as u16) << 8) | (low as u16)
    }

    // read byte from rom
    fn read_byte(&mut self) -> u8 {
        let byte = self.memory.read_rom(self.registers.pc);
        self.registers.pc += 1;
        byte
    }
    
    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        // Fetch the opcode from memory
        let opcode = self.read_byte();
        // Execute the instruction
        self.execute_instruction(opcode);
    }

    fn get_operand(&self, operand : &str, op_im : bool) -> Operand {
        // get the immediate value
        let op_reg = match_string_to_register(operand);
        
        if op_im {
            Operand::Register(op_reg)
        } else {
            let value = self.registers.get_register_value_16(op_reg);
            Operand::Memory(value)
        }
    }
    
    fn execute_instruction(&mut self, op : u8) {
        // take the hex string of op 
        let op = format!("0x{:02X}", op);
        // print OPCODE 
        let a = self.opcodes["unprefixed"][&op].clone();

        // get the instruction from the opcode
        let instr = match_string_to_instruction((&a["mnemonic"]).as_str().unwrap());

        let operands = &a["operands"];

        if operands.len() == 2 {
            // get the first operand
            let op1 = &operands[0]["name"].as_str().unwrap();
            let op1_imm : bool = operands[0]["immediate"].as_bool().unwrap();
            // get the second operand
            let op2 = &operands[1]["name"].as_str().unwrap();
            // get the immediate value
            let op2_imm : bool = operands[1]["immediate"].as_bool().unwrap();
            let imm : bool = operands[1]["immediate"].as_bool().unwrap();
            
    
            // if op1_imm is true, then it is an immediate value
            // let operand1 = Operand::Register(match_string_to_register(op1));
            let operand1 =  self.get_operand(op1, op1_imm);
            // if op2_imm is true, then it is an immediate value

            let operand2 =  {
                if imm {
                    if op2.contains("8"){
                        // get the immediate value
                        let imm = self.read_byte();
                        Operand::Immediate(imm)
                    } else {
                        // get the immediate value
                        let imm = self.read_word();
                        Operand::Immediate16(imm)
                    }
                } else {
                    self.get_operand(op2, op2_imm)
                }
            };

            if deb {
                println!("---------------DEBUG------------------------");
                println!("Instructions: {:?}", instr);
                println!("Operand 1: {:?}", operand1);
                println!("Operand 2: {:?}", operand2);
                println!("Immediate: {:?}", imm);
                println!("Instruction: {:?}", instr);
                println!("-------------------------------------------");

            }
            match instr {
                Instruction::ADD => {
                    // ADD instruction
                    // self.registers.add(operand1, operand2);
                    let bit_len = operand1.get_bit_length();
                    if bit_len == 8 {
                        // 8 bit add
                        // self.registers.add_8bit(operand1, operand2);
                        add_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                    } else if bit_len == 16 {
                        // 16 bit add
                        // self.registers.add_16bit(operand1, operand2);
                        add_16bit(&mut self.registers, operand1, operand2);
                    } else {
                        panic!("Invalid operand size");
                    }

                }
                Instruction::SUB => {
                    // SUB instruction
                    // self.registers.sub(operand1, operand2);
                    sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                }
               
                
                Instruction::LD => {
                    
                    // LD instruction
                    let bit_len = operand1.get_bit_length();
                    if bit_len == 8 {
                        // 8 bit load
                        ld_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                    } else if bit_len == 16 {
                        // 16 bit load
                        // self.registers.ld(operand1, operand2);
                        ld_16bit(&mut self.registers, operand1, operand2);
                    } else {
                        panic!("Invalid operand size");
                    }
                }
            

                _ => {
                    panic!("Unknown instruction: {:?}", instr);
                }
    
            }
        }
        else if operands.len() == 1 {
            
            
        }
        else if operands.len() == 0 {
            // no operands
        }
        else {
            panic!("Invalid number of operands");
            
        }



        
        // print the opcode
        // if deb {
        //     println!("Opcode: {} - Instruction: {:?}", op, instr);
        //     println!("a : {:?}", a);
        //     println!("Operands: {:?}", operands);
        // }

    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_new() {
        let rom = vec![0x00; 0x8000]; // Dummy ROM data
        let cpu = CPU::new(rom);
     
        assert_eq!(cpu.registers.sp, 0xFFFE);
        assert_eq!(cpu.registers.pc, 0x0);
    }

    // Opcode tests
    #[test]
    fn test_load_bc_imm() {
        let rom = vec![0x69; 0x8000]; // Dummy ROM data
        let mut cpu = CPU::new(rom);
        
        // Test a specific opcode execution
        cpu.execute_instruction(0x01); // NOP instruction
        assert_eq!(cpu.registers.pc, 2); // PC should increment by 1
        // BC should be equal to 0x6969
        assert_eq!(cpu.registers.get_register_value_16(RegisterNames::BC), 0x6969-1);
    }
}