
pub struct MMU {
    ram : [u8; 0xFFFF],
    rom : Vec<u8>,
    
}

impl MMU {
    
    pub fn new(rom: Vec<u8>) -> MMU {
        MMU {
            ram: [0; 0xFFFF],
            rom,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }
}

