use crate::gb::registers::{Registers, RegisterNames};
use crate::gb::mmu::Memory;
/// Represents an operand, which can be a register or a memory address.
pub enum Operand {
    Register(RegisterNames),
    Memory(u16), // Memory address
}

impl Operand {
    pub fn read(&self, registers: &Registers, memory: &Memory) -> u8 {
        match self {
            Operand::Register(reg) => registers.get_register_value_8(*reg),
            Operand::Memory(addr) => memory.read_byte(*addr),
        }
    }

    pub fn write(&self, value: u8, registers: &mut Registers, memory: &mut Memory) {
        match self {
            Operand::Register(reg) => registers.set_register_value_8(*reg, value),
            Operand::Memory(addr) => memory.write_byte(*addr, value),
        }
    }
    pub fn write_u16(&self, value: u16, registers: &mut Registers) {
        match self {
            Operand::Register(RegisterNames::AF) => registers.af = value,
            Operand::Register(RegisterNames::BC) => registers.bc = value,
            Operand::Register(RegisterNames::DE) => registers.de = value,
            Operand::Register(RegisterNames::HL) => registers.hl = value,
            Operand::Register(RegisterNames::SP) => registers.sp = value,
            Operand::Register(RegisterNames::PC) => registers.pc = value,
            _ => panic!("Invalid register for 16-bit write"),
        }
    }
    pub fn read_16(&self, registers: &Registers) -> u16 {
        match self {
            Operand::Register(RegisterNames::AF) => registers.af,
            Operand::Register(RegisterNames::BC) => registers.bc,
            Operand::Register(RegisterNames::DE) => registers.de,
            Operand::Register(RegisterNames::HL) => registers.hl,
            Operand::Register(RegisterNames::SP) => registers.sp,
            Operand::Register(RegisterNames::PC) => registers.pc,
            _ => panic!("Invalid register for 16-bit read"),
        }
    }
}

/// Trait for all instructions.
pub trait Instruction {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory);
}

// Macro for defining arithmetic instructions.
macro_rules! define_arithmetic_instruction {
    ($name:ident, $operation:expr) => {
        pub struct $name {
            op1: Operand,
            op2: Operand,
        }

        impl $name {
            pub fn new(op1: Operand, op2: Operand) -> Self {
                Self { op1, op2 }
            }
        }

        impl Instruction for $name {
            fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
                let value1 = self.op1.read(registers, memory);
                let value2 = self.op2.read(registers, memory);
                let result = $operation(value1, value2);

                self.op1.write(result, registers, memory);
                // registers.update_flags(value1, value2, result);
                registers.pc += 1;
            }
        }
    };
}
// Define Add, Sub, And, Or, Xor, Cp instructions
define_arithmetic_instruction!(Add, |a : u8 , b : u8| a.wrapping_add(b));
define_arithmetic_instruction!(Sub, |a : u8, b : u8| a.wrapping_sub(b));

define_arithmetic_instruction!(And, |a : u8 , b : u8| a & b);
define_arithmetic_instruction!(Or, |a : u8 , b : u8| a | b);
define_arithmetic_instruction!(Xor, |a : u8 , b : u8| a ^ b);

// Define Inc, Dec 
define_arithmetic_instruction!(Inc, |a : u8 , _| a.wrapping_add(1));
define_arithmetic_instruction!(Dec, |a : u8, _| a.wrapping_sub(1));


// Nop instruction: Does nothing but increments the program counter.
pub struct Nop;

    impl Instruction for Nop {
        fn execute(&self, registers: &mut Registers, _memory: &mut Memory) {
            registers.pc += 1;
        }
    }

// Load instruction: Loads a value into a register or memory address.
pub struct Load {
    op1: Operand,
    op2: Operand,
}

impl Load {
    pub fn new(op1: Operand, op2: Operand) -> Self {
        Self { op1, op2 }
    }
}

impl Instruction for Load {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let value = self.op2.read(registers, memory);
        self.op1.write(value, registers, memory);
        registers.pc += 1;
    }
}

// The Xor instruction is already defined using the macro.

// cp instruction: Compares two operands by subtracting the second operand from the first operand, but does not store the result.

pub struct Cp {
    op1: Operand,
    op2: Operand,
}

impl Cp {
    pub fn new(op1: Operand, op2: Operand) -> Self {
        Self { op1, op2 }
    }
}

impl Instruction for Cp {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let value1 = self.op1.read(registers, memory);
        let value2 = self.op2.read(registers, memory);

        let result = value1.wrapping_sub(value2);

        // registers.update_flags(value1, value2, result);
        registers.pc += 1;
    }
}

// push instruction: Pushes a value onto the stack.

pub struct Push {
    op: Operand,
}

impl Push {
    pub fn new(op: Operand) -> Self {
        Self { op }
    }
}

impl Instruction for Push {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let value = self.op.read(registers, memory);

        registers.sp -= 1;
        memory.write_byte(registers.sp, value);
        registers.pc += 1;
    }
}
// Pop instruction: Pops a value from the stack.

pub struct Pop {
    op: Operand,
}

impl Pop {
    pub fn new(op: Operand) -> Self {
        Self { op }
    }
}

impl Instruction for Pop {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        let value = memory.read_byte(registers.sp);
        registers.sp += 1;
        self.op.write(value, registers, memory);
        registers.pc += 1;
    }
}


// call instruction: Calls a subroutine at the specified memory address.

pub struct Call {
    addr: u16,
}

impl Call {
    pub fn new(addr: u16) -> Self {
        Self { addr }
    }
}

impl Instruction for Call {
    fn execute(&self, registers: &mut Registers, memory: &mut Memory) {
        // Push the return address onto the stack
        let return_addr = registers.pc + 3;
        registers.sp -= 2;
        memory.write_byte(registers.sp, (return_addr >> 8) as u8);
        memory.write_byte(registers.sp + 1, (return_addr & 0xFF) as u8);
        registers.pc = self.addr;
    }
}
