use super::instructions::*;
use super::mmu::MMU;
use super::{
    get_opcodes, match_string_to_flag, match_string_to_instruction, match_string_to_register,
    FlagNames, Instruction, Operand, RegisterNames, Registers, DEB,
};
use json;
use log;

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
        if blen == 16 {
            operation2(&mut self.registers, operand1, operand2, &mut self.memory)
        } else if blen == 8 {
            operation(&mut self.registers, &mut self.memory, operand1, operand2);
        } else {
            panic!("Invalid bit length");
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

        log::debug!("Opcode : {:02X}", opcode);
        log::debug!("Program counter : {:02X}", self.registers.pc);
        // Execute the instruction
        self.execute_instruction(opcode);
    }
    fn is_flag(&mut self, operand: &str, instr: &Instruction) -> bool {
        return (operand.contains("Z") || operand.contains("NZ") || operand.contains("C"))
            && (*instr == Instruction::JP
                || *instr == Instruction::CALL
                || *instr == Instruction::RET
                || *instr == Instruction::JR);
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
        } else if self.is_flag(operand, instr) {
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
        // take the hex string of op
        let op = format!("0x{:02X}", op);
        // print OPCODE
        let a = self.opcodes["unprefixed"][&op].clone();

        // get the instruction from the opcode
        let instr = match_string_to_instruction((&a["mnemonic"]).as_str().unwrap());

        let operands = &a["operands"];
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
            log::debug!("{:?} {:?},{:?} \n", instr, operand1, operand2);

            ops[0] = operand1;
            ops[1] = operand2;
        } else if operands.len() == 1 {
            // get the first operand
            let op1 = &operands[0]["name"].as_str().unwrap();
            let op1_imm: bool = operands[0]["immediate"].as_bool().unwrap();
            // get instruction
            let instr = match_string_to_instruction((&a["mnemonic"]).as_str().unwrap());

            let operand1 = self.get_operand(op1, op1_imm, &instr);

            // imm
            let imm: bool = operands[0]["immediate"].as_bool().unwrap();
            log::debug!("{:?} {:?} \n", instr, operand1);
            ops[0] = operand1;
        }

        instr.match_instruction(&mut self.registers, &mut self.memory, &ops);
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
