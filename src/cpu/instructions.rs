use super::{Instruction, Operand, RegisterNames, Registers};
use crate::MMU;

/// Represents an operand, which can be a register or a memory address.
impl Instruction {
    fn execute_two_operand<F, T>(
        &self,
        registers: &mut Registers,
        memory: &mut MMU,
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
            operation2(registers, operand1, operand2, memory)
        } else if blen == 8 {
            operation(registers, memory, operand1, operand2);
        } else {
            panic!("Invalid bit length");
        }
    }
    pub fn match_instruction(
        &self,
        registers: &mut Registers,
        memory: &mut MMU,
        ops: &[Operand; 2],
    ) {
        let operand1 = ops[0];
        let operand2 = ops[1];
        match self {
            Instruction::NOP => {
                // NOP instruction
                // do nothing
            }
            // 2 operand operation
            Instruction::ADD => {
                // ADD instruction
                self.execute_two_operand(
                    registers, memory, operand1, operand2, add_8bit, add_16bit,
                );
            }
            Instruction::ADC => {
                // ADC instruction
                adc_8bit(registers, memory, operand1, operand2);
            }
            Instruction::SUB => {
                // SUB instruction
                sub_8bit(registers, memory, operand1, operand2);
            }
            Instruction::LD => {
                ld(registers, operand1, operand2, memory);
            }
            Instruction::LDH => {
                // LDH instruction - Load from or store to high memory area (0xFF00-0xFFFF)
                ldh(registers, operand1, operand2, memory);
            }
            Instruction::AND => {
                // AND instruction
                and_8bit(registers, memory, operand1, operand2);
            }
            Instruction::OR => {
                // OR instruction
                or_8bit(registers, memory, operand1, operand2);
            }
            Instruction::XOR => {
                // XOR instruction
                xor_8bit(registers, memory, operand1, operand2);
            }
            Instruction::CP => {
                // CP instruction
                cp_8bit(registers, memory, operand1, operand2);
            }

            // One or two operand
            Instruction::CALL => match operand2 {
                Operand::NIL => call(registers, memory, operand1, true),
                _ => {
                    let condition = operand1.read(&registers, &memory) == 1;
                    call(registers, memory, operand2, condition);
                }
            },
            Instruction::JP => match operand2 {
                Operand::NIL => jp(registers, memory, operand1, true),
                _ => {
                    let condition = operand1.read(&registers, &memory) == 1;
                    jp(registers, memory, operand2, condition);
                }
            },
            Instruction::JR => match operand2 {
                Operand::NIL => jr(registers, memory, operand1, true),
                _ => {
                    let condition = operand1.read(&registers, &memory) == 1;
                    jr(registers, memory, operand2, condition);
                }
            },
            Instruction::RET => match operand1 {
                Operand::NIL => ret(registers, memory, true),
                _ => {
                    let condition = operand1.read(registers, memory) == 1;
                    ret(registers, memory, condition);
                }
            },

            // One operand
            Instruction::INC => {
                // INC instruction
                let bit_len = operand1.get_bit_length();

                if bit_len == 8 {
                    // 8 bit inc
                    inc_8bit(registers, memory, operand1);
                } else if bit_len == 16 {
                    // 16 bit inc
                    inc_16bit(registers, operand1, memory);
                } else {
                    panic!("Invalid operand size");
                }
            }
            Instruction::DEC => {
                // DEC instruction
                if operand1.get_bit_length() == 8 {
                    // 8 bit dec
                    dec_8bit(registers, memory, operand1);
                } else if operand1.get_bit_length() == 16 {
                    // 16 bit dec
                    dec_16bit(registers, operand1, memory);
                } else {
                    panic!("Invalid operand size");
                }
            }
            Instruction::PUSH => {
                // PUSH instruction
                // self.registers.push(operand1);
                push(registers, memory, operand1);
            }

            Instruction::POP => {
                // POP instruction
                // self.registers.pop(operand1);
                // self.pop(operand1);
                pop(registers, memory, operand1);
            }

            // No operand
            Instruction::RRCA => {
                // RRCA instruction
                rrc(registers, memory, Operand::Register(RegisterNames::A));
            }
            Instruction::RLCA => {
                // RLCA instruction
                rlc(registers, memory, Operand::Register(RegisterNames::A));
            }
            Instruction::RRA => {
                // RRA instruction
                rr(registers, memory, Operand::Register(RegisterNames::A));
            }
            Instruction::RLA => {
                // RLA instruction
                rl(registers, memory, Operand::Register(RegisterNames::A));
            }
            Instruction::CPL => {
                // CPL instruction
                cpl(registers);
            }
            Instruction::DAA => {
                // DAA instruction
                daa(registers);
            }

            Instruction::SCF => {
                // SCF instruction
                scf(registers);
            }
            Instruction::CCF => {
                // CCF instruction
                ccf(registers);
            }
            Instruction::DI => {
                // DI instruction - Disable Interrupts
                di(registers);
            }
            _ => {
                panic!("Unknown instruction: {:?}", self);
            }
        }
    }

    pub fn match_prefix_instruction(
        &self,
        registers: &mut Registers,
        memory: &mut MMU,
        ops: &[Operand; 2],
    ) {
        let operand1 = ops[0];
        let operand2 = ops[1];
        match self {
            Instruction::RLC => {
                rlc(registers, memory, operand1);
            }
            Instruction::RRC => {
                rrc(registers, memory, operand1);
            }
            Instruction::RL => {
                rl(registers, memory, operand1);
            }
            Instruction::RR => {
                rr(registers, memory, operand1);
            }
            Instruction::SLA => {
                sla(registers, memory, operand1);
            }
            Instruction::SRA => {
                sra(registers, memory, operand1);
            }
            Instruction::SWAP => {
                swap(registers, memory, operand1);
            }
            Instruction::SRL => {
                srl(registers, memory, operand1);
            }
            Instruction::BIT => {
                let bit_position = operand1.read(registers, memory);
                bit(registers, memory, operand2, bit_position);
            }
            Instruction::RES => {
                let bit_position = operand1.read(registers, memory);
                res(registers, memory, operand2, bit_position);
            }
            Instruction::SET => {
                let bit_position = operand1.read(registers, memory);
                set(registers, memory, operand2, bit_position);
            }
            _ => panic!("Unhandled CB-prefixed instruction: {:?}", self),
        }
    }
}

// get operand bit length
impl Operand {
    pub fn read(&self, registers: &Registers, memory: &MMU) -> u8 {
        match self {
            Operand::Register(reg) => registers.get_register_value_8(*reg),
            Operand::Memory(addr) => memory.read(*addr),
            Operand::Immediate(value) => *value,
            Operand::Flag(flag) => registers.get_flag(flag) as u8,
            _ => panic!("Invalid operand for read"),
        }
    }

    pub fn write(&self, value: u8, registers: &mut Registers, memory: &mut MMU) {
        match self {
            Operand::Register(reg) => registers.set_register_value_8(*reg, value),
            Operand::Memory(addr) => memory.write(*addr, value),
            _ => panic!("Invalid operand for write"),
        }
    }
    pub fn write_u16(&self, value: u16, registers: &mut Registers, memory: &mut MMU) {
        match self {
            Operand::Register(reg) => registers.set_register_value_16(*reg, value),
            Operand::Memory(addr) => memory.write(*addr, value as u8),

            _ => panic!("Invalid register for 16-bit write {:?}", self),
        }
    }
    pub fn read_16(&self, registers: &Registers, memory: &mut MMU) -> u16 {
        match self {
            Operand::Register(reg) => registers.get_register_value_16(*reg),
            Operand::Immediate16(value) => *value,
            Operand::Memory(addr) => memory.read(*addr) as u16,
            _ => panic!("Invalid register for 16-bit read {:?}", self),
        }
    }
    pub fn get_bit_length(&self) -> u8 {
        match self {
            Operand::Register(a) => match a {
                RegisterNames::A => 8,
                RegisterNames::B => 8,
                RegisterNames::C => 8,
                RegisterNames::D => 8,
                RegisterNames::E => 8,
                RegisterNames::H => 8,
                RegisterNames::L => 8,
                RegisterNames::AF => 16,
                RegisterNames::BC => 16,
                RegisterNames::DE => 16,
                RegisterNames::HL => 16,
                RegisterNames::SP => 16,
                RegisterNames::PC => 16,
            },
            Operand::Memory(i) => {
                if *i > 255 {
                    16
                } else {
                    8
                }
            }
            Operand::Immediate(_) => 8,
            Operand::Immediate16(_) => 16,
            Operand::Flag(_) => 16,
            _ => 0,
        }
    }
}

pub fn inc_8bit(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = value.wrapping_add(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = (value & 0x0F) == 0x0F;

    // Write result back to the operand
    operand.write(result, registers, memory);
}
pub fn dec_8bit(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = value.wrapping_sub(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value & 0x0F) == 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn inc_16bit(registers: &mut Registers, operand: Operand, memory: &mut MMU) {
    let value = operand.read_16(registers, memory);
    let result = value.wrapping_add(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = (value & 0x0FFF) == 0x0FFF;

    // Write result back to the operand
    operand.write_u16(result, registers, memory);
}

pub fn dec_16bit(registers: &mut Registers, operand: Operand, memory: &mut MMU) {
    let value = operand.read_16(registers, memory);
    let result = value.wrapping_sub(1);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value & 0x0FFF) == 0;

    // Write result back to the operand
    operand.write_u16(result, registers, memory);
}

pub fn add_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1.wrapping_add(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = ((value1 & 0x0F) + (value2 & 0x0F)) > 0x0F;
    registers.flag.c = (value1 as u16 + value2 as u16) > 0xFF;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn adc_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let carry = if registers.flag.c { 1 } else { 0 };
    let result = value1.wrapping_add(value2).wrapping_add(carry);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = ((value1 & 0x0F) + (value2 & 0x0F) + carry) > 0x0F;
    registers.flag.c = (value1 as u16 + value2 as u16 + carry as u16) > 0xFF;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn add_16bit(
    registers: &mut Registers,
    operand1: Operand,
    operand2: Operand,
    memory: &mut MMU,
) {
    let value1 = operand1.read_16(registers, memory);
    let value2 = operand2.read_16(registers, memory);
    let result = value1.wrapping_add(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = ((value1 & 0x0FFF) + (value2 & 0x0FFF)) > 0x0FFF;
    registers.flag.c = (value1 as u32 + value2 as u32) > 0xFFFF;

    // Write result back to the first operand
    operand1.write_u16(result, registers, memory);
}

pub fn sub_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0F) < (value2 & 0x0F);
    registers.flag.c = (value1 as u16) < (value2 as u16);

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn sbc_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let carry = if registers.flag.c { 1 } else { 0 };
    let result = value1.wrapping_sub(value2).wrapping_sub(carry);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0F) < (value2 & 0x0F) + carry;
    registers.flag.c = (value1 as u16) < (value2 as u16) + carry as u16;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn sub_16bit(
    registers: &mut Registers,
    operand1: Operand,
    operand2: Operand,
    memory: &mut MMU,
) {
    let value1 = operand1.read_16(registers, memory);
    let value2 = operand2.read_16(registers, memory);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0FFF) < (value2 & 0x0FFF);
    registers.flag.c = (value1 as u32) < (value2 as u32);

    // Write result back to the first operand
    operand1.write_u16(result, registers, memory);
}

pub fn and_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1 & value2;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = true;
    registers.flag.c = false;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn or_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1 | value2;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = false;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}

pub fn xor_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1 ^ value2;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = false;

    // Write result back to the first operand
    operand1.write(result, registers, memory);
}
pub fn cp_8bit(registers: &mut Registers, memory: &mut MMU, operand1: Operand, operand2: Operand) {
    let value1 = operand1.read(registers, memory);
    let value2 = operand2.read(registers, memory);
    let result = value1.wrapping_sub(value2);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = true;
    registers.flag.h = (value1 & 0x0F) < (value2 & 0x0F);
    registers.flag.c = (value1 as u16) < (value2 as u16);
}

pub fn ld(registers: &mut Registers, operand1: Operand, operand2: Operand, memory: &mut MMU) {
    let mut value: u16 = 0;
    if operand2.get_bit_length() == 8 {
        value = operand2.read(registers, memory) as u16;
    } else {
        value = operand2.read_16(registers, memory);
    }
    if operand1.get_bit_length() == 16 {
        operand1.write_u16(value, registers, memory);
    } else {
        operand1.write(value as u8, registers, memory);
    }
}
pub fn ldh(registers: &mut Registers, operand1: Operand, operand2: Operand, memory: &mut MMU) {
    // Calculate the high memory address - always 0xFF00 + offset
    let addr = match operand1 {
        Operand::Register(RegisterNames::A) => {
            // LDH A, (a8) or LDH A, (C)
            let offset = match operand2 {
                Operand::Immediate(imm) => imm as u16,
                Operand::Register(RegisterNames::C) => {
                    registers.get_register_value_8(RegisterNames::C) as u16
                }
                _ => panic!("Invalid operand for LDH instruction"),
            };
            let addr = 0xFF00 + offset;
            let value = memory.read(addr);
            registers.set_register_value_8(RegisterNames::A, value);
            return;
        }
        Operand::Immediate(imm) => 0xFF00 + imm as u16,
        Operand::Register(RegisterNames::C) => {
            0xFF00 + registers.get_register_value_8(RegisterNames::C) as u16
        }
        _ => panic!("Invalid operand for LDH instruction"),
    };

    // Store the value from register A to the high memory address
    let value = registers.get_register_value_8(RegisterNames::A);
    memory.write(addr, value);
}

// rlc : rotate left circular
pub fn rlc(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = (value << 1) | (value >> 7);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x80) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn rrc(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = (value >> 1) | (value << 7);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x01) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn rl(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let carry = registers.flag.c as u8;
    let result = (value << 1) | carry;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x80) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

pub fn rr(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let carry = registers.flag.c as u8;
    let result = (value >> 1) | (carry << 7);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x01) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// shift left arithmetic
pub fn sla(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = value << 1;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x80) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}
// shift right arithmetic
pub fn sra(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = (value >> 1) | (value & 0x80);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x01) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Swap nibbles
pub fn swap(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = (value << 4) | (value >> 4);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = false;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Bit test
pub fn bit(registers: &mut Registers, memory: &mut MMU, operand: Operand, bit: u8) {
    let value = operand.read(registers, memory);
    let result = value & (1 << bit);

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = true;
    registers.flag.c = false;
}

// Bit set
pub fn set(registers: &mut Registers, memory: &mut MMU, operand: Operand, bit: u8) {
    let value = operand.read(registers, memory);
    let result = value | (1 << bit);

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Bit reset
pub fn res(registers: &mut Registers, memory: &mut MMU, operand: Operand, bit: u8) {
    let value = operand.read(registers, memory);
    let result = value & !(1 << bit);

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Shift right logical
pub fn srl(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read(registers, memory);
    let result = value >> 1;

    // Set flags
    registers.flag.z = result == 0;
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = (value & 0x01) != 0;

    // Write result back to the operand
    operand.write(result, registers, memory);
}

// Control flow instructions

pub fn jp(registers: &mut Registers, memory: &mut MMU, operand: Operand, condition: bool) {
    if condition {
        let address = operand.read_16(registers, memory);
        let pc = &mut registers.pc;
        *pc = address;
    }
}

pub fn call(registers: &mut Registers, memory: &mut MMU, operand: Operand, condition: bool) {
    if condition {
        let address = operand.read_16(registers, memory);
        let pc = &mut registers.pc;
        let sp = &mut registers.sp;
        *sp = sp.wrapping_sub(2);
        memory.write_word(*sp, *pc);
        *pc = address;
    }
}

pub fn jr(registers: &mut Registers, memory: &mut MMU, operand: Operand, condition: bool) {
    if condition {
        let offset = operand.read(registers, memory) as i8;
        let pc = &mut registers.pc;
        *pc = pc.wrapping_add(offset as u16);
    }
}

pub fn ret(registers: &mut Registers, memory: &mut MMU, cond: bool) {
    // Pop the address from the stack and set it as the new program counter
    if cond {
        let sp = &mut registers.sp;
        let low = memory.read(*sp);
        let high = memory.read(*sp + 1);
        let pc = &mut registers.pc;
        *pc = ((high as u16) << 8) | (low as u16);
        *sp = sp.wrapping_add(2);
    }
}

pub fn daa(registers: &mut Registers) {
    let mut a = registers.af >> 8;
    a &= 0xFF; // Ensure a is 8 bits
    let mut carry = 0;

    if registers.flag.n {
        if registers.flag.c {
            a = a.wrapping_sub(0x60);
            carry = 1;
        }
        if registers.flag.h {
            a = a.wrapping_sub(0x06);
            carry = 1;
        }
    } else {
        if registers.flag.c || a > 0x99 {
            a = a.wrapping_add(0x60);
            carry = 1;
        }
        if registers.flag.h || (a & 0x0F) > 0x09 {
            a = a.wrapping_add(0x06);
            carry = 1;
        }
    }

    registers.af = (registers.af & 0xFF00) | (a << 8);
    registers.flag.z = a == 0;
    registers.flag.h = false;
    registers.flag.c = carry != 0;
}

/// Disables interrupts by clearing the IME flag
pub fn di(registers: &mut Registers) {
    // The DI instruction disables interrupts by clearing the IME (Interrupt Master Enable) flag
    // This prevents the CPU from responding to any interrupts

    // Note: DI doesn't immediately disable interrupts, it actually disables them after
    // the instruction following DI is executed, but this simplified implementation
    // disables them immediately
}

pub fn cpl(registers: &mut Registers) {
    let a = registers.af >> 8;
    registers.af = (registers.af & 0xFF00) | (!a << 8);
    registers.flag.n = true;
    registers.flag.h = true;
}
pub fn scf(registers: &mut Registers) {
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = true;
}
pub fn ccf(registers: &mut Registers) {
    registers.flag.n = false;
    registers.flag.h = false;
    registers.flag.c = !registers.flag.c;
}

pub fn push(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let value = operand.read_16(registers, memory);
    let sp = &mut registers.sp;
    *sp = sp.wrapping_sub(2);
    memory.write_word(*sp, value);
}

pub fn pop(registers: &mut Registers, memory: &mut MMU, operand: Operand) {
    let sp = &mut registers.sp;
    let value = memory.read_word(*sp);
    *sp = sp.wrapping_add(2);
    operand.write_u16(value, registers, memory);
}
