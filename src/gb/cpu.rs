use crate::gb::instructions::{Instruction, Add, Nop, Or, Operand};
use crate::gb::registers::{Registers, RegisterNames};
use crate::gb::mmu::Memory;

pub struct CPU {
    registers: Registers,
    memory: Memory,
    instructions: Vec<Box<dyn Instruction>>,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            memory: Memory::new(),
            instructions: vec![],
        }
    }

    pub fn load_instructions(&mut self) {
        self.instructions.push(Box::new(Nop));
        self.instructions.push(Box::new(Add::new(
            Operand::Register(RegisterNames::A),
            Operand::Register(RegisterNames::B),
        )));
        self.instructions.push(Box::new(Or::new(
            Operand::Register(RegisterNames::A),
            Operand::Memory(0xC000), // Example memory address
        )));
    }

    pub fn execute_instructions(&mut self) {
        for instruction in &self.instructions {
            instruction.execute(&mut self.registers, &mut self.memory);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu() {
        let mut cpu = CPU::new();
        cpu.load_instructions();
        cpu.execute_instructions();
    }
}