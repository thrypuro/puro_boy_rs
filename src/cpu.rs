mod instructions;
mod registers;
mod shared;

use crate::MMU;
use json::{self, JsonValue};
use log;
use shared::{
    get_opcodes, is_flag, match_string_preinstruction, match_string_to_flag,
    match_string_to_instruction, match_string_to_register, Instruction, Operand, RegisterNames,
    Registers,
};

pub struct CPU<'a> {
    registers: Registers,
    pub memory: &'a mut MMU,
    halted: bool,
    opcodes: json::JsonValue,
    ime: bool, // Interrupt Master Enable flag
}

impl<'a> CPU<'a> {
    /// Creates a new CPU instance with the given ROM data.
    pub fn new(mmu: &'a mut MMU) -> Self {
        Self {
            registers: Registers::new(),
            memory: mmu,
            halted: false,
            opcodes: get_opcodes(),
            ime: false,
        }
    }

    // read word from rom
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

        // log::debug!("Opcode : {:02X}", opcode);
        // log::debug!("Program counter : {:02X}", self.registers.pc);
        // Execute the instruction
        self.execute_instruction(opcode);
    }

    fn get_operand(&mut self, operand: &str, op_im: bool, instr: &Instruction) -> Operand {
        if operand.contains("16") {
            let value = self.read_word();
            if *instr == Instruction::LD && !op_im {
                Operand::Memory(value)
            } else {
                Operand::Immediate16(value)
            }
        } else if operand.contains("8") {
            // get the immediate value
            let imm = self.read_byte();
            Operand::Immediate(imm)
        } else if is_flag(operand, instr) {
            Operand::Flag(match_string_to_flag(operand))
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
        // Check if it's a prefixed instruction (0xCB)
        if op == 0xCB {
            self.execute_prefixed_instruction();
            return;
        }

        // take the hex string of op
        let op: String = format!("0x{:02X}", op);
        // print OPCODE
        let a: JsonValue = self.opcodes["unprefixed"][op].clone();

        // get the instruction from the opcode
        let instr: Instruction = match_string_to_instruction((&a["mnemonic"]).as_str().unwrap());

        // Check for PREFIX instruction (0xCB)
        if instr == Instruction::PREFIX {
            self.execute_prefixed_instruction();
            return;
        }

        let operands: &JsonValue = &a["operands"];
        let mut ops: [Operand; 2] = [Operand::NIL; 2];
        if operands.len() == 2 {
            // get the first operand
            let op1 = &operands[0]["name"].as_str().unwrap();
            let op1_imm: bool = operands[0]["immediate"].as_bool().unwrap();
            // get the second operand
            let op2 = &operands[1]["name"].as_str().unwrap();
            // get the immediate value
            let op2_imm: bool = operands[1]["immediate"].as_bool().unwrap();

            // if op1_imm is true, then it is an immediate value
            // let operand1 = Operand::Register(match_string_to_register(op1));
            let operand1 = self.get_operand(op1, op1_imm, &instr);
            // if op2_imm is true, then it is an immediate value
            let operand2 = self.get_operand(op2, op2_imm, &instr);
            ops[0] = operand1;
            ops[1] = operand2;
        } else if operands.len() == 1 {
            // get the first operand
            let op1 = &operands[0]["name"].as_str().unwrap();
            let op1_imm: bool = operands[0]["immediate"].as_bool().unwrap();
            // get instruction
            let instr = match_string_to_instruction((&a["mnemonic"]).as_str().unwrap());

            let operand1 = self.get_operand(op1, op1_imm, &instr);
            ops[0] = operand1;
        }

        // For debugging
        match ops[1] {
            Operand::NIL => match ops[0] {
                Operand::NIL => log::debug!("{:?} \n", instr),
                _ => log::debug!("{:?} {:?} \n", instr, ops[0]),
            },
            _ => {
                log::debug!("{:?} {:?},{:?} \n", instr, ops[0], ops[1])
            }
        }

        instr.match_instruction(&mut self.registers, &mut self.memory, &ops);
    }

    fn execute_prefixed_instruction(&mut self) {
        // Read the second byte of the prefixed instruction
        let op = self.read_byte();
        let op_str = format!("0x{:02X}", op);

        // Get the instruction from the prefixed opcodes
        let a: JsonValue = self.opcodes["cbprefixed"][op_str].clone();
        let instr_str = (&a["mnemonic"]).as_str().unwrap();

        // Match the CB prefixed instruction
        let instr = match_string_preinstruction(instr_str);

        // Process operands
        let operands: &JsonValue = &a["operands"];
        let mut ops: [Operand; 2] = [Operand::NIL; 2];

        if operands.len() == 2 {
            let op1 = &operands[0]["name"].as_str().unwrap();
            let op1_imm: bool = operands[0]["immediate"].as_bool().unwrap();
            let op2 = &operands[1]["name"].as_str().unwrap();
            let op2_imm: bool = operands[1]["immediate"].as_bool().unwrap();

            let operand1 = if op1.contains("bit") {
                // For BIT, SET, RES instructions, first operand is bit number
                let bit_num = op1[3..].parse::<u8>().unwrap();
                Operand::Immediate(bit_num)
            } else {
                self.get_operand(op1, op1_imm, &instr)
            };

            let operand2 = self.get_operand(op2, op2_imm, &instr);
            ops[0] = operand1;
            ops[1] = operand2;
        } else if operands.len() == 1 {
            let op1 = &operands[0]["name"].as_str().unwrap();
            let op1_imm: bool = operands[0]["immediate"].as_bool().unwrap();

            let operand1 = self.get_operand(op1, op1_imm, &instr);
            ops[0] = operand1;
        }

        log::debug!("CB Prefixed: {:?} {:?},{:?}\n", instr, ops[0], ops[1]);

        // Execute the CB prefixed instruction
        // match instr {
        //     Instruction::RLC => self.execute_rlc(&ops),
        //     Instruction::RRC => self.execute_rrc(&ops),
        //     Instruction::RL => self.execute_rl(&ops),
        //     Instruction::RR => self.execute_rr(&ops),
        //     Instruction::SLA => self.execute_sla(&ops),
        //     Instruction::SRA => self.execute_sra(&ops),
        //     Instruction::SWAP => self.execute_swap(&ops),
        //     Instruction::SRL => self.execute_srl(&ops),
        //     Instruction::BIT => self.execute_bit(&ops),
        //     Instruction::RES => self.execute_res(&ops),
        //     Instruction::SET => self.execute_set(&ops),
        //     _ => panic!("Unhandled CB-prefixed instruction: {:?}", instr),
        // }
        //
        instr.match_prefix_instruction(&mut self.registers, &mut self.memory, &ops);
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

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
