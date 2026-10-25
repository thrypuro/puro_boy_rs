mod ioreg;
mod ppu;

use crate::error::{EmuError, EmuRes};
use ioreg::IOReg;
use ppu::PPU;

const EXTRAM_SIZE: usize = (0xBFFF - 0xA000) + 1;
const WRAM_SIZE: usize = (0xDFFF - 0xC000) + 1;
const ERAM_SIZE: usize = (0xFDFF - 0xE000) + 1;
const OAM_SIZE: usize = (0xFE9F - 0xFE00) + 1;
const HRAM_SIZE: usize = (0xFFFE - 0xFF80) + 1;

// Memory Map
#[derive(Debug, PartialEq, Eq)]
pub struct MMU {
    // ROM (0,0x7fff) represented a vec, but usually 0x800
    rom: Vec<u8>,
    // contains VRAM, tiles and tileset (0x8000 , 0x9fff)
    ppu: PPU,
    // External ram (from Cart) (0xA000 ,0xBFFF)
    ext_ram: [u8; EXTRAM_SIZE],
    // Work ram WRAM(0xC000,0xDFFF)
    wram: [u8; WRAM_SIZE],
    // Echo ram (prohibited), Copy of WRAM until 0xDDFF
    eram: [u8; ERAM_SIZE],
    // Object Attribute Memory
    oam: [u8; OAM_SIZE],
    // IO Registers
    ioreg: IOReg,
    // High Ram
    hram: [u8; HRAM_SIZE],
    // Interrupts Enable Register
    ie: u8,
}

impl MMU {
    /// Builds the memory map from a ROM image. ROMs smaller than 32KB are padded with 0.
    pub fn new(rom: Vec<u8>) -> MMU {
        MMU {
            rom: rom,
            ppu: PPU::new(),
            ext_ram: [0u8; EXTRAM_SIZE],
            wram: [0u8; WRAM_SIZE],
            eram: [0u8; ERAM_SIZE],
            oam: [0u8; OAM_SIZE],
            ioreg: IOReg { joypad: 0 },
            hram: [0u8; HRAM_SIZE],
            ie: 0,
        }
    }
    /// Advances every clocked component (PPU, timer) by `cycles` T-cycles and
    /// ORs any interrupts they request into IF (0xFF0F).
    pub fn tick(&mut self, cycles: u8) {
        todo!("MMU::tick")
    }
    /// Every byte sent over the serial port so far (Blargg test ROMs print here).
    pub fn serial_output(&self) -> &str {
        todo!("MMU::serial_output")
    }

    /// Reads a byte from the RAM at the specified address.
    pub fn read(&self, address: u16) -> EmuRes<u8> {
        let addr = address as usize;
        match address {
            0x0000..=0x7FFF => Ok(self.rom[addr]),
            0x8000..=0x9FFF => Ok(0), // PPU need to implement
            0xA000..=0xBFFF => Ok(self.ext_ram[addr - 0xA000]),
            0xC000..=0xDFFF => Ok(self.wram[addr - 0xC000]),
            0xE000..=0xFDFF => Ok(self.eram[addr - 0xE000]),
            0xFE00..=0xFE9F => Ok(self.oam[addr - 0xFE00]),
            0xFEA0..=0xFEFF => Err(EmuError::ProhibitedRead),
            0xFF00..=0xFF7F => Ok(0), // ioreg need to implement
            0xFF80..=0xFFFE => Ok(self.hram[addr - 0xFF80]),
            0xFFFF => Ok(self.ie),
            _ => Err(EmuError::ProhibitedRead),
        }
    }
    /// Writes a byte to the RAM at the specified address.
    pub fn write(&mut self, address: u16, value: u8) -> EmuRes<()> {
        // self.ram[(address) as usize] = value;
        // println!("ram is {}", self.ram[address as usize]);
        let addr = address as usize;
        match address {
            0x0000..=0x7FFF => {} // Rom is immutable
            0x8000..=0x9FFF => {}
            0xA000..=0xBFFF => self.ext_ram[addr - 0xA000] = value,
            0xC000..=0xDFFF => {
                self.wram[addr - 0xC000] = value;
                if address <= 0xDDFF {
                    self.eram[addr - 0xC000] = value; // Echo copies whatever is on WRam
                }
            }

            0xFE00..=0xFE9F => self.oam[addr - 0xFE00] = value,
            0xFEA0..=0xFEFF => return Err(EmuError::ProhibitedWrite),
            0xFF00..=0xFF7F => todo!(), // ioreg need to implement
            0xFF80..=0xFFFE => self.hram[addr - 0xFF80] = value,
            0xFFFF => self.ie = value,
            _ => return Err(EmuError::ProhibitedWrite),
        }
        Ok(())
    }
    /// Reads a byte from the RAM at the specified address.
    pub fn read_word(&self, address: u16) -> EmuRes<u16> {
        let low = self
            .read(address)
            .map_err(|_| EmuError::ProhibitedReadWord)?;
        let high = self
            .read(address + 1)
            .map_err(|_| EmuError::ProhibitedReadWord)?;
        Ok(u16::from_le_bytes([low, high]))
    }
    /// Writes a word (2 bytes) to the RAM at the specified address.
    pub fn write_word(&mut self, address: u16, value: u16) -> EmuRes<()> {
        let [low, high] = value.to_le_bytes();
        self.write(address, low)
            .map_err(|_| EmuError::ProhibitedWriteWord)?;
        self.write(address + 1, high)
            .map_err(|_| EmuError::ProhibitedWriteWord)
    }
}

#[cfg(test)]
mod tests;
