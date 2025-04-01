
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
    /// Reads a byte from the RAM at the specified address.
    pub fn read(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }
    /// Writes a byte to the RAM at the specified address.
    pub fn write(&mut self, address: u16, value: u8) {
        self.ram[address as usize] = value;
    }
    /// Reads a byte from the RAM at the specified address.
    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read(address);
        let high = self.read(address + 1);
        ((high as u16) << 8) | (low as u16)
    }
    /// Writes a word (2 bytes) to the RAM at the specified address.
    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write(address, (value & 0x00FF) as u8);
        self.write(address + 1, ((value >> 8) & 0x00FF) as u8);
    }
    /// Reads a byte from the ROM at the specified address.
    pub fn read_rom(&self, address: u16) -> u8 {
        self.rom[address as usize]
    }
}

