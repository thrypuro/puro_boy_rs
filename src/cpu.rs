mod alu;


pub struct cpu {
    registers : alu::Registers,
    memory : [u8; 65536],
    clock : u64,
}


impl cpu {
    pub fn new() -> cpu {
        cpu {
            registers: alu::Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0,
                l: 0,
                f: 0,
                pc: 0,
                sp: 0,
            },
            memory: [0; 65536],
            clock: 0,
        }
    }
}