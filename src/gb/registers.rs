struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,    
}
struct Reg {
    pub A : u8,
    pub B : u8,
}

pub struct Registers {
    pub A : u8,
    pub BC : Reg,
    pub DE : Reg,
    pub HL : Reg, 
    pub SP : Reg, 
    pub PC : Reg,
    pub flags : Flags,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            A: 0,
            BC: Reg::new(),
            DE: Reg::new(),
            HL: Reg::new(),
            SP: Reg::new(),
            PC: Reg::new(),
            flags: Flags {
                z: false,
                n: false,
                h: false,
                c: false,
            }
        }
    }
}


impl Reg {

    pub fn new() -> Reg {
        Reg {
            A: 0,
            B: 0,
        }
    }
    pub fn inc(&mut self) {
        // check for overflow
        if self.B == 0xFF {
            self.A += 1;
            self.B = 0;
        } else {
            self.B += 1;
        }
    }


}

