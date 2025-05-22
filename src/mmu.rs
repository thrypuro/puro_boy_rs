use crate::ppu::PPU;

#[derive(Debug, PartialEq, Eq)]
pub struct MMU {
    // Memory Map
    // ROM (0,0x7fff)
    rom: [u8; 0x8000],
    // contains VRAM, tiles and tileset (0x8000 , 0x9fff)
    ppu: PPU,
    // External ram (from Cart) (0xA000 ,0xBFFF)
    ext_ram: [u8; (0xBFFF - 0xA000) + 1],
    // Work ram WRAM(0xC000,0xDFFF)
    wram: [u8; (0xDFFF - 0xC000) + 1],
    // Echo ram (prohibited)
}

impl MMU {
    // pub fn new(rom: Vec<u8>) -> MMU {
    //     // let mut m = MMU {
    //     //     ram: [0; 0xFFFF + 1],
    //     // };
    //     // println!("Length of rom {}", &rom.len());
    //     // for i in 0..0x8000 {
    //     //     m.ram[i] = rom[i];
    //     // }
    //     // m
    // }
    /// Reads a byte from the RAM at the specified address.
    pub fn read(&self, address: u16) -> u8 {
        // self.ram[(address) as usize]
        0
    }
    /// Writes a byte to the RAM at the specified address.
    pub fn write(&mut self, address: u16, value: u8) {
        // self.ram[(address) as usize] = value;
        // println!("ram is {}", self.ram[address as usize]);
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
