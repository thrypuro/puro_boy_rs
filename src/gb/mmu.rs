
struct memory {
    ram : [u8; 0xFFFF],
}


impl memory {
    
     fn new() -> memory {
        memory {
            ram: [0; 0xFFFF],
        }
    }
    fn read_byte(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }
}

