mod cart;
mod ioreg;
mod ppu;

use ppu::PPU;

const ROM_BANK_SIZE: usize = 0x4000;
const VRAM_SIZE: usize = 0x2000;
const EXTERNAL_RAM_SIZE: usize = 0x2000;
const WRAM_SIZE: usize = 0x2000;
const ECHO_RAM_SIZE: usize = 0x1E00;
const OAM_SIZE: usize = 0xA0;
const IO_REGISTERS_SIZE: usize = 0x80;
const HRAM_SIZE: usize = 0x7F;

#[derive(Debug, PartialEq, Eq)]
pub struct MMU {
    // Memory Map
    // ROM Bank 0 (0x0000-0x3FFF)
    rom_bank0: [u8; ROM_BANK_SIZE],
    // ROM Bank 1-n (0x4000-0x7FFF)
    rom_bank1: [u8; ROM_BANK_SIZE],

    // External RAM (0xA000-0xBFFF)
    ext_ram: [u8; EXTERNAL_RAM_SIZE],
    // Work RAM (0xC000-0xDFFF)
    wram: [u8; WRAM_SIZE],
    // Echo RAM - mirror of WRAM (0xE000-0xFDFF)
    eram: [u8; ECHO_RAM_SIZE],
    // OAM - Sprite Attribute Table (0xFE00-0xFE9F)
    oam: [u8; OAM_SIZE],
    // I/O Registers (0xFF00-0xFF7F)
    io_registers: [u8; IO_REGISTERS_SIZE],
    // High RAM (0xFF80-0xFFFE)
    hram: [u8; HRAM_SIZE],
    // Interrupt Enable Register (0xFFFF)
    ie_register: u8,
    // PPU
    pub ppu: PPU,
}

impl MMU {
    pub fn new(rom: Vec<u8>) -> MMU {
        let mut rom_bank0 = [0; ROM_BANK_SIZE];
        let mut rom_bank1 = [0; ROM_BANK_SIZE];

        // Copy ROM data to ROM banks
        for i in 0..ROM_BANK_SIZE {
            if i < rom.len() {
                rom_bank0[i] = rom[i];
            }
        }

        for i in 0..ROM_BANK_SIZE {
            let index = i + ROM_BANK_SIZE;
            if index < rom.len() {
                rom_bank1[i] = rom[index];
            }
        }

        let mut mmu = MMU {
            rom_bank0,
            rom_bank1,

            ext_ram: [0; EXTERNAL_RAM_SIZE],
            wram: [0; WRAM_SIZE],
            eram: [0; ECHO_RAM_SIZE],
            oam: [0; OAM_SIZE],
            io_registers: [0; IO_REGISTERS_SIZE],
            hram: [0; HRAM_SIZE],
            ie_register: 0,
            ppu: PPU::new(),
        };

        // Initialize PPU with tile data from ROM if it exists
        mmu.init_ppu();

        mmu
    }

    /// Initialize PPU with tile data from ROM
    fn init_ppu(&mut self) {
        // Pass relevant tile data to PPU
        self.ppu.init(&self.rom_bank0, &self.rom_bank1);
    }

    /// Reads a byte from memory at the specified address.
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom_bank0[address as usize],
            0x4000..=0x7FFF => self.rom_bank1[(address - 0x4000) as usize],
            0x8000..=0x9FFF => {
                let vram_addr = (address - 0x8000) as usize;
                self.ppu.vram[vram_addr]
            }
            0xA000..=0xBFFF => self.ext_ram[(address - 0xA000) as usize],
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize], // Echo RAM, mirrors WRAM
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0, // Unusable memory
            0xFF00..=0xFF7F => self.io_registers[(address - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie_register,
        }
    }

    /// Writes a byte to memory at the specified address.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {
                // ROM is read-only, but some games use writes to this region for memory bank switching
                // For now, we ignore these writes
            }
            0x8000..=0x9FFF => {
                let vram_addr = (address - 0x8000) as usize;
                self.ppu.vram[vram_addr] = value;
                // Update tile data in PPU if VRAM is written to
                if address >= 0x8000 && address < 0x9800 {
                    self.ppu.update_tile(address, value);
                }
            }
            0xA000..=0xBFFF => self.ext_ram[(address - 0xA000) as usize] = value,
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value, // Echo RAM, mirrors WRAM
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = value,
            0xFEA0..=0xFEFF => {} // Unusable memory, ignore writes
            0xFF00..=0xFF7F => {
                self.io_registers[(address - 0xFF00) as usize] = value;
                // Special handling for specific I/O registers
                match address {
                    0xFF40 => self.ppu.update_lcd_control(value),
                    0xFF41 => self.ppu.update_lcd_status(value),
                    0xFF42 => self.ppu.set_scroll_y(value),
                    0xFF43 => self.ppu.set_scroll_x(value),
                    0xFF44 => {} // LY - read only
                    0xFF45 => self.ppu.set_ly_compare(value),
                    0xFF47 => self.ppu.set_bg_palette(value),
                    0xFF48 => self.ppu.set_obj_palette0(value),
                    0xFF49 => self.ppu.set_obj_palette1(value),
                    0xFF4A => self.ppu.set_window_y(value),
                    0xFF4B => self.ppu.set_window_x(value),
                    _ => {}
                }
            }
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie_register = value,
        }
    }

    /// Reads a word (2 bytes) from memory at the specified address.
    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read(address);
        let high = self.read(address + 1);
        ((high as u16) << 8) | (low as u16)
    }

    /// Writes a word (2 bytes) to memory at the specified address.
    pub fn write_word(&mut self, address: u16, value: u16) {
        self.write(address, (value & 0x00FF) as u8);
        self.write(address + 1, ((value >> 8) & 0x00FF) as u8);
    }

    /// Reads a byte from the ROM at the specified address.
    pub fn read_rom(&self, address: u16) -> u8 {
        if address < 0x4000 {
            self.rom_bank0[address as usize]
        } else if address < 0x8000 {
            self.rom_bank1[(address - 0x4000) as usize]
        } else {
            0 // Invalid ROM address
        }
    }

    /// Get a reference to the PPU for rendering
    pub fn get_ppu(&self) -> &PPU {
        &self.ppu
    }

    /// Get a mutable reference to the PPU for rendering
    pub fn get_ppu_mut(&mut self) -> &mut PPU {
        &mut self.ppu
    }

    /// Update the PPU for one cycle
    pub fn update_ppu(&mut self, cycles: u32) -> bool {
        self.ppu.update(cycles)
    }
}
