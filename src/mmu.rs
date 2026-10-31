mod ioreg;
mod ppu;

use crate::error::{EmuError, EmuRes};
use ioreg::IOReg;
use ppu::PPU;

const ROM_SIZE: usize = 0x8000;
const EXTRAM_SIZE: usize = (0xBFFF - 0xA000) + 1;
const WRAM_SIZE: usize = (0xDFFF - 0xC000) + 1;
const HRAM_SIZE: usize = (0xFFFE - 0xFF80) + 1;

// Memory Map
#[derive(Debug, PartialEq, Eq)]
pub struct MMU {
    // ROM (0,0x7fff) represented a vec, but usually 0x800
    rom: [u8; ROM_SIZE],
    // contains VRAM, tiles and tileset (0x8000 , 0x9fff)
    ppu: PPU,
    // External ram (from Cart) (0xA000 ,0xBFFF)
    ext_ram: [u8; EXTRAM_SIZE],
    // Work ram WRAM(0xC000,0xDFFF)
    wram: [u8; WRAM_SIZE],
    // Echo ram (0xE000,0xFDFF) is treated as prohibited

    // IO Registers (LCD registers live in the PPU)
    ioreg: IOReg,
    // Last value written to the OAM DMA register (0xFF46)
    dma: u8,
    // High Ram
    hram: [u8; HRAM_SIZE],
    // Interrupts Enable Register
    ie: u8,
}

fn to_array<T: Default, const N: usize>(v: Vec<T>) -> [T; N] {
    let mut it = v.into_iter();
    std::array::from_fn(|_| it.next().unwrap_or_default())
}

impl MMU {
    /// Builds the memory map from a ROM image. ROMs smaller than 32KB are padded with 0.
    pub fn new(rom: Vec<u8>) -> MMU {
        MMU {
            rom: to_array(rom),
            ppu: PPU::new(),
            ext_ram: [0u8; EXTRAM_SIZE],
            wram: [0u8; WRAM_SIZE],
            ioreg: IOReg::new(),
            dma: 0,
            hram: [0u8; HRAM_SIZE],
            ie: 0,
        }
    }
    /// Advances every clocked component (PPU, timer) by `cycles` T-cycles and
    /// ORs any interrupts they request into IF (0xFF0F).
    pub fn tick(&mut self, cycles: u8) {
        let requested = self.ppu.tick(cycles);
        self.ioreg.if_reg |= requested;
        self.ioreg.tick(cycles);
    }
    /// Every byte sent over the serial port so far (Blargg test ROMs print here).
    pub fn serial_output(&self) -> &str {
        self.ioreg.serial_output()
    }

    /// Reads a byte from the RAM at the specified address.
    pub fn read(&self, address: u16) -> EmuRes<u8> {
        let addr = address as usize;
        match address {
            0x0000..=0x7FFF => Ok(self.rom[addr]),
            0x8000..=0x9FFF => Ok(self.ppu.read(addr)), // PPU need to implement
            0xA000..=0xBFFF => Ok(self.ext_ram[addr - 0xA000]),
            0xC000..=0xDFFF => Ok(self.wram[addr - 0xC000]),
            0xFE00..=0xFE9F => Ok(self.ppu.read(addr)),
            0xFF46 => Ok(self.dma),
            0xFF40..=0xFF4B => Ok(self.ppu.read(addr)),
            0xFF00..=0xFF7F => Ok(self.ioreg.read(addr)),
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
            0x8000..=0x9FFF => self.ppu.write(addr, value),

            0xA000..=0xBFFF => self.ext_ram[addr - 0xA000] = value,
            0xC000..=0xDFFF => self.wram[addr - 0xC000] = value,

            0xFE00..=0xFE9F => self.ppu.write(addr, value),
            0xFF46 => self.oam_dma(value)?,
            0xFF40..=0xFF4B => self.ppu.write(addr, value),
            0xFF00..=0xFF7F => self.ioreg.write(addr, value),
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
        let high = address
            .checked_add(1)
            .and_then(|next| self.read(next).ok())
            .ok_or(EmuError::ProhibitedReadWord)?;
        Ok(u16::from_le_bytes([low, high]))
    }
    /// Writes a word (2 bytes) to the RAM at the specified address.
    pub fn write_word(&mut self, address: u16, value: u16) -> EmuRes<()> {
        let [low, high] = value.to_le_bytes();
        self.write(address, low)
            .map_err(|_| EmuError::ProhibitedWriteWord)?;
        let next = address
            .checked_add(1)
            .ok_or(EmuError::ProhibitedWriteWord)?;
        self.write(next, high)
            .map_err(|_| EmuError::ProhibitedWriteWord)
    }
    /// Copies 160 bytes from `page << 8` into OAM. Real DMA takes 640 cycles;
    /// this does it instantly.
    fn oam_dma(&mut self, page: u8) -> EmuRes<()> {
        self.dma = page;
        let source = (page as u16) << 8;
        for i in 0..0xA0u16 {
            let value = self.read(source + i)?;
            self.ppu.write(0xFE00 + i as usize, value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
