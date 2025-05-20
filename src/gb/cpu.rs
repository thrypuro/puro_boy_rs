use crate::gb::instructions::*;
use crate::gb::mmu::MMU;
use crate::gb::opcodes::{get_opcodes, match_string_to_instruction, Instruction};
use crate::gb::registers::{match_string_to_register, RegisterNames, Registers};
use json;

use super::registers::FlagNames;

const DEB: bool = true;

pub struct CPU {
    registers: Registers,
    pub memory: MMU,
    halted: bool,
    opcodes: json::JsonValue,
}

impl CPU {
    /// Creates a new CPU instance with the given ROM data.
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            registers: Registers::new(),
            memory: MMU::new(rom),
            halted: false,
            opcodes: get_opcodes(),
        }
    }

    fn execute_two_operand<F, T>(
        &mut self,
        operand1: Operand,
        operand2: Operand,
        operation: F,
        operation2: T,
    ) where
        F: Fn(&mut Registers, &mut MMU, Operand, Operand),
        T: Fn(&mut Registers, Operand, Operand, &mut MMU),
    {
        let blen = operand1.get_bit_length();
        let blen2 = operand2.get_bit_length();
        if blen == 16 {
            operation2(&mut self.registers, operand1, operand2, &mut self.memory)
        } else if blen == 8 {
            operation(&mut self.registers, &mut self.memory, operand1, operand2);
        } else {
            panic!("Invalid bit length");
        }
    }

    fn execute_one_operand<F, T>(&mut self, operand1: Operand, operation: F, operation2: T)
    where
        F: Fn(&mut Registers, &mut MMU, Operand),
        T: Fn(&mut Registers, Operand),
    {
        let blen = operand1.get_bit_length();
        if blen == 16 {
            operation2(&mut self.registers, operand1)
        } else if blen == 8 {
            operation(&mut self.registers, &mut self.memory, operand1);
        } else {
            panic!("Invalid bit length");
        }
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

        println!("Opcode : {:02X}", opcode);
        println!("Program counter : {:02X}", self.registers.pc);
        // Execute the instruction
        self.execute_instruction(opcode);
    }
    fn is_flag(&mut self, operand: &str) -> bool {
        return operand.contains("Z") || operand.contains("NZ") || operand.contains("C");
    }

    fn get_operand(&mut self, operand: &str, op_im: bool, instr: &Instruction) -> Operand {
        if operand.contains("16") {
            let value = self.read_word();
            if *instr == Instruction::LD {
                Operand::Memory(value)
            } else {
                Operand::Immediate16(value)
            }
        } else if operand.contains("8") {
            // get the immediate value
            let imm = self.read_byte();
            Operand::Immediate(imm)
        } else if self.is_flag(operand)
            && (*instr == Instruction::JP
                || *instr == Instruction::CALL
                || *instr == Instruction::RET
                || *instr == Instruction::JR)
        {
            match operand {
                "Z" => Operand::Flag(FlagNames::Z),
                "H" => Operand::Flag(FlagNames::H),
                "C" => Operand::Flag(FlagNames::C),
                "N" => Operand::Flag(FlagNames::N),
                "NZ" => Operand::Flag(FlagNames::NZ),
                _ => panic!("Invalid Flag type"),
            }
        } else {
            // get the immediate value
            let op_reg = match_string_to_register(operand);

            if op_im {
                Operand::Register(op_reg)
            } else {
                let value = self.registers.get_register_value_16(op_reg);
                Operand::Memory(value)
            }
        }
    }

    fn execute_instruction(&mut self, op: u8) {
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
            let op1_imm: bool = operands[0]["immediate"].as_bool().unwrap();
            // get the second operand
            let op2 = &operands[1]["name"].as_str().unwrap();
            // get the immediate value
            let op2_imm: bool = operands[1]["immediate"].as_bool().unwrap();
            let imm: bool = operands[1]["immediate"].as_bool().unwrap();

            // if op1_imm is true, then it is an immediate value
            // let operand1 = Operand::Register(match_string_to_register(op1));
            let operand1 = self.get_operand(op1, op1_imm, &instr);
            // if op2_imm is true, then it is an immediate value

            let operand2 = {
                if imm {
                    if op2.contains("n8") {
                        // get the immediate value
                        let imm = self.read_byte();
                        Operand::Immediate(imm)
                    } else if op2.contains("n16") {
                        // get the immediate value
                        let imm = self.read_word();
                        Operand::Immediate16(imm)
                    } else {
                        self.get_operand(op2, op2_imm, &instr)
                    }
                } else {
                    self.get_operand(op2, op2_imm, &instr)
                }
            };
            if DEB {
                println!("{:?} {:?},{:?} \n", instr, operand1, operand2);
            }
            match instr {
                Instruction::ADD => {
                    // ADD instruction
                    self.execute_two_operand(operand1, operand2, add_8bit, add_16bit);
                }
                Instruction::ADC => {
                    // ADC instruction
                    adc_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                }
                Instruction::SUB => {
                    // SUB instruction
                    sub_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                }
                Instruction::LD => {
                    ld(&mut self.registers, operand1, operand2, &mut self.memory);
                }
                Instruction::AND => {
                    // AND instruction
                    and_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                }
                Instruction::OR => {
                    // OR instruction
                    or_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                }
                Instruction::XOR => {
                    // XOR instruction
                    xor_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                }
                Instruction::CP => {
                    // CP instruction
                    cp_8bit(&mut self.registers, &mut self.memory, operand1, operand2);
                }
                Instruction::CALL => {
                    let condition = operand1.read(&self.registers, &self.memory) == 1;
                    call(&mut self.registers, &mut self.memory, operand2, condition);
                }
                Instruction::JP => {
                    let condition = operand1.read(&self.registers, &self.memory) == 1;
                    jp(&mut self.registers, &mut self.memory, operand2, condition);
                }
                Instruction::JR => {
                    let condition = operand1.read(&self.registers, &self.memory) == 1;
                    jr(&mut self.registers, &mut self.memory, operand2, condition);
                }
                _ => {
                    panic!("Unknown instruction: {:?}", instr);
                }
            }
        } else if operands.len() == 1 {
            // get the first operand
            let op1 = &operands[0]["name"].as_str().unwrap();
            let op1_imm: bool = operands[0]["immediate"].as_bool().unwrap();
            // get instruction
            let instr = match_string_to_instruction((&a["mnemonic"]).as_str().unwrap());

            let operand1 = self.get_operand(op1, op1_imm, &instr);

            // imm
            let imm: bool = operands[0]["immediate"].as_bool().unwrap();
            if DEB {
                println!("{:?} {:?} \n", instr, operand1);
            }

            match instr {
                Instruction::INC => {
                    // INC instruction
                    let bit_len = operand1.get_bit_length();

                    if bit_len == 8 {
                        // 8 bit inc
                        inc_8bit(&mut self.registers, &mut self.memory, operand1);
                    } else if bit_len == 16 {
                        // 16 bit inc
                        inc_16bit(&mut self.registers, operand1, &mut self.memory);
                    } else {
                        panic!("Invalid operand size");
                    }
                }
                Instruction::DEC => {
                    // DEC instruction
                    if operand1.get_bit_length() == 8 {
                        // 8 bit dec
                        dec_8bit(&mut self.registers, &mut self.memory, operand1);
                    } else if operand1.get_bit_length() == 16 {
                        // 16 bit dec
                        dec_16bit(&mut self.registers, operand1, &mut self.memory);
                    } else {
                        panic!("Invalid operand size");
                    }
                }

                Instruction::PUSH => {
                    // PUSH instruction
                    // self.registers.push(operand1);
                    push(&mut self.registers, &mut self.memory, operand1);
                }

                Instruction::POP => {
                    // POP instruction
                    // self.registers.pop(operand1);
                    // self.pop(operand1);
                    pop(&mut self.registers, &mut self.memory, operand1);
                }

                Instruction::JP => {
                    // JP instruction
                    jp(&mut self.registers, &mut self.memory, operand1, true);
                }

                Instruction::JR => {
                    jr(&mut self.registers, &mut self.memory, operand1, true);
                }

                _ => {
                    panic!("Unknown instruction: {:?}", instr);
                }
            }
        } else if operands.len() == 0 {
            if DEB {
                println!("{:?} \n", instr);
            }
            match instr {
                Instruction::NOP => {
                    // NOP instruction
                    // do nothing
                }
                Instruction::RRCA => {
                    // RRCA instruction
                    // self.registers.rra();
                    rrc(
                        &mut self.registers,
                        &mut self.memory,
                        Operand::Register(RegisterNames::A),
                    );
                }
                Instruction::RLCA => {
                    // RLCA instruction
                    // self.registers.rla();
                    rlc(
                        &mut self.registers,
                        &mut self.memory,
                        Operand::Register(RegisterNames::A),
                    );
                }
                Instruction::RRA => {
                    // RRA instruction
                    // self.registers.rra();
                    rr(
                        &mut self.registers,
                        &mut self.memory,
                        Operand::Register(RegisterNames::A),
                    );
                }
                Instruction::RLA => {
                    // RLA instruction
                    // self.registers.rla();
                    rl(
                        &mut self.registers,
                        &mut self.memory,
                        Operand::Register(RegisterNames::A),
                    );
                }
                Instruction::CPL => {
                    // CPL instruction
                    cpl(&mut self.registers);
                }
                Instruction::DAA => {
                    // DAA instruction
                    // self.registers.daa();
                    daa(&mut self.registers);
                }

                Instruction::SCF => {
                    // SCF instruction
                    // self.registers.scf();
                    scf(&mut self.registers);
                }
                Instruction::CCF => {
                    // CCF instruction
                    // self.registers.ccf();
                    ccf(&mut self.registers);
                }

                Instruction::RET => {
                    // RET instruction
                    // self.registers.ret();
                    ret(&mut self.registers, &mut self.memory, true);
                }

                _ => {
                    panic!("Unknown instruction: {:?}", instr);
                }
            }
        } else {
            panic!("Invalid number of operands");
        }
    }

    pub fn print_registers(&self) {
        println!("Register values:");
        println!(
            "A:  0x{:02X}",
            self.registers.get_register_value_8(RegisterNames::A)
        );

        println!(
            "B:  0x{:02X}",
            self.registers.get_register_value_8(RegisterNames::B)
        );
        println!(
            "C:  0x{:02X}",
            self.registers.get_register_value_8(RegisterNames::C)
        );
        println!(
            "D:  0x{:02X}",
            self.registers.get_register_value_8(RegisterNames::D)
        );
        println!(
            "E:  0x{:02X}",
            self.registers.get_register_value_8(RegisterNames::E)
        );
        println!(
            "H:  0x{:02X}",
            self.registers.get_register_value_8(RegisterNames::H)
        );
        println!(
            "L:  0x{:02X}",
            self.registers.get_register_value_8(RegisterNames::L)
        );
        println!(
            "AF: 0x{:04X}",
            self.registers.get_register_value_16(RegisterNames::AF)
        );
        println!(
            "BC: 0x{:04X}",
            self.registers.get_register_value_16(RegisterNames::BC)
        );
        println!(
            "DE: 0x{:04X}",
            self.registers.get_register_value_16(RegisterNames::DE)
        );
        println!(
            "HL: 0x{:04X}",
            self.registers.get_register_value_16(RegisterNames::HL)
        );
        println!("SP: 0x{:04X}", self.registers.sp);
        println!("PC: 0x{:04X}", self.registers.pc);
        println!(
            "Flags: Z={}, N={}, H={}, C={}",
            self.registers.flag.get_z(),
            self.registers.flag.get_n(),
            self.registers.flag.get_h(),
            self.registers.flag.get_c()
        );
    }
}

// tests

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
        assert_eq!(
            cpu.registers.get_register_value_16(RegisterNames::BC),
            0x6969
        );
    }

    #[test]
    fn test_inc_bc() {
        let rom = vec![0x69; 0x8000]; // Dummy ROM data
        let mut cpu = CPU::new(rom);

        // Test a specific opcode execution
        cpu.execute_instruction(0x03); // INC BC instruction
        cpu.execute_instruction(0x03); // INC BC instruction

        assert_eq!(cpu.registers.get_register_value_16(RegisterNames::BC), 2);
    }

    #[test]
    fn test_add_a_b() {
        let rom = vec![0x69; 0x8000]; // Dummy ROM data
        let mut cpu = CPU::new(rom);

        // Set initial values for registers
        cpu.execute_instruction(0x3E); // LD A, 0x69
        cpu.execute_instruction(0x06); // LD B, 0x69

        // Test a specific opcode execution
        cpu.execute_instruction(0x80); // ADD A, B instruction
        assert_eq!(
            cpu.registers.get_register_value_8(RegisterNames::A),
            0x69 + 0x69
        );
    }

    // test push pop
    #[test]
    fn test_push_pop() {
        let rom = vec![0x69; 0x8000]; // Dummy ROM data
        let mut cpu = CPU::new(rom);

        // Set initial values for registers
        cpu.registers
            .set_register_value_16(RegisterNames::AF, 0x1234);
        cpu.registers.sp = 0xFFFE;

        // Test push
        cpu.execute_instruction(0xF5); // PUSH AF instruction
        assert_eq!(cpu.memory.read(cpu.registers.sp), 0x34);
        assert_eq!(cpu.memory.read(cpu.registers.sp + 1), 0x12);

        // Test pop
        cpu.execute_instruction(0xF1); // POP AF instruction
        assert_eq!(
            cpu.registers.get_register_value_16(RegisterNames::AF),
            0x1234
        );
    }
    // test add and if carry flag is set
    #[test]
    fn test_add_carry_flag() {
        let rom = vec![0x69; 0x8000]; // Dummy ROM data
        let mut cpu = CPU::new(rom);

        // Set initial values for registers
        cpu.registers.set_register_value_8(RegisterNames::A, 0xFF);
        cpu.registers.set_register_value_8(RegisterNames::B, 0x01);

        // Test ADD A, B instruction
        cpu.execute_instruction(0x80); // ADD A, B instruction

        // Check if the carry flag is set
        assert_eq!(cpu.registers.flag.get_c(), true);
        // Check the value of register A
        assert_eq!(cpu.registers.get_register_value_8(RegisterNames::A), 0x00);
    }
}
