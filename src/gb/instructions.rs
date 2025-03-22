use crate::gb::registers::Registers;


pub enum  Register_names{
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

pub fn add(registers: &mut Registers, r1 : register_names, r2 : register_names ) {

    
   

}

pub fn nop(registers: &mut Registers) {
    registers.pc += 1;
}
