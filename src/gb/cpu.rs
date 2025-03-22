use crate::gb::registers::Registers;

pub struct cpu {
    registers : Registers,
    pub clock : u64,
}


impl cpu {

    pub fn new() -> cpu {
        cpu {
            registers: Registers::new(),
            clock: 0,
        }
    }
}
