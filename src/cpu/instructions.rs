mod alu;
mod bits;
mod jump;
mod load;
mod logical;
mod misc;
mod operand;
mod rotate;
mod stack;

use super::{Instruction, Operand, RegisterNames, Registers};
use crate::MMU;
use alu::*;
use bits::*;
use jump::*;
use load::*;
use logical::*;
use misc::*;
use rotate::*;
use stack::*;

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
                Operand::NIL => jp(registers, memory, operand2, true),
                _ => {
                    let condition = operand1.read(&registers, &memory) == 1;
                    jp(registers, memory, operand2, condition);
                }
            },
            Instruction::JR => match operand2 {
                Operand::NIL => jr(registers, memory, operand2, true),
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
}
