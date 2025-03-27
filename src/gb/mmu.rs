
pub struct Memory {
    ram : [u8; 0xFFFF],
}

impl Memory {
    
     pub fn new() -> Memory {
        Memory {
            ram: [0; 0xFFFF],
        }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }
}

