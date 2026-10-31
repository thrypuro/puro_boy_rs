//! The cartridge: ROM, optional external RAM, and the memory bank controller (MBC)
//! that maps banks of both into 0x0000-0x7FFF and 0xA000-0xBFFF.
//!
//! Reference: https://gbdev.io/pandocs/MBCs.html

const ROM_BANK_SIZE: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;

// Cartridge header fields
const HEADER_TITLE: std::ops::Range<usize> = 0x0134..0x0144;
const HEADER_TYPE: usize = 0x0147;
const HEADER_RAM_SIZE: usize = 0x0149;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbcKind {
    None,
    Mbc1,
    Mbc3,
    Mbc5,
}

/// Bank-switching state. Each MBC only uses some of these fields.
#[derive(Debug, Default, PartialEq, Eq)]
struct BankRegs {
    ram_enabled: bool,
    /// Selected switchable ROM bank (before masking to the ROM size).
    rom_bank: usize,
    /// MBC1: 2-bit secondary register. MBC3/MBC5: RAM bank (or RTC register on MBC3).
    upper: usize,
    /// MBC1 banking mode: false = simple, true = advanced (upper bits also bank 0x0000 and RAM).
    mode: bool,
    /// MBC3 real-time-clock registers 0x08-0x0C. Stored but the clock does not tick.
    rtc: [u8; 5],
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    kind: MbcKind,
    has_battery: bool,
    regs: BankRegs,
}

impl Cartridge {
    /// Builds a cartridge from a ROM image, reading the MBC type and RAM size from its
    /// header. ROMs smaller than 32KB (test programs) are padded with 0.
    pub fn new(mut rom: Vec<u8>) -> Self {
        if rom.len() < 2 * ROM_BANK_SIZE {
            rom.resize(2 * ROM_BANK_SIZE, 0);
        }

        let (kind, has_battery) = match rom[HEADER_TYPE] {
            0x00 | 0x08 => (MbcKind::None, false),
            0x09 => (MbcKind::None, true),
            0x01 | 0x02 => (MbcKind::Mbc1, false),
            0x03 => (MbcKind::Mbc1, true),
            0x11 | 0x12 => (MbcKind::Mbc3, false),
            0x0F | 0x10 | 0x13 => (MbcKind::Mbc3, true),
            0x19 | 0x1A | 0x1C | 0x1D => (MbcKind::Mbc5, false),
            0x1B | 0x1E => (MbcKind::Mbc5, true),
            other => {
                log::warn!("unsupported cartridge type {other:#04X}, treating it as ROM only");
                (MbcKind::None, false)
            }
        };

        let ram_size = match rom[HEADER_RAM_SIZE] {
            0x02 => RAM_BANK_SIZE,
            0x03 => 4 * RAM_BANK_SIZE,
            0x04 => 16 * RAM_BANK_SIZE,
            0x05 => 8 * RAM_BANK_SIZE,
            // ROM-only carts may still have plain RAM wired at 0xA000 (and the test
            // programs rely on it), so give them one bank.
            _ if kind == MbcKind::None => RAM_BANK_SIZE,
            _ => 0,
        };

        Cartridge {
            rom,
            ram: vec![0; ram_size],
            kind,
            has_battery,
            regs: BankRegs {
                rom_bank: 1,
                ..BankRegs::default()
            },
        }
    }

    /// The game title from the header, for the window title.
    pub fn title(&self) -> String {
        self.rom[HEADER_TITLE]
            .iter()
            .take_while(|&&b| b != 0)
            .filter(|b| b.is_ascii_graphic() || **b == b' ')
            .map(|&b| b as char)
            .collect::<String>()
            .trim()
            .to_string()
    }

    pub fn kind(&self) -> MbcKind {
        self.kind
    }

    /// External RAM contents, if the cartridge keeps them powered with a battery.
    pub fn battery_ram(&self) -> Option<&[u8]> {
        (self.has_battery && !self.ram.is_empty()).then_some(&self.ram[..])
    }

    /// Restores battery-backed RAM from a save file. Mismatched sizes are ignored.
    pub fn load_battery_ram(&mut self, data: &[u8]) -> bool {
        if self.has_battery && data.len() == self.ram.len() {
            self.ram.copy_from_slice(data);
            true
        } else {
            false
        }
    }

    fn rom_banks(&self) -> usize {
        self.rom.len() / ROM_BANK_SIZE
    }

    fn ram_banks(&self) -> usize {
        (self.ram.len() / RAM_BANK_SIZE).max(1)
    }

    /// Which ROM banks are mapped at 0x0000 and 0x4000.
    fn rom_bank_numbers(&self) -> (usize, usize) {
        let r = &self.regs;
        let (low, high) = match self.kind {
            MbcKind::None => (0, 1),
            MbcKind::Mbc1 => {
                let upper = r.upper << 5;
                (if r.mode { upper } else { 0 }, upper | r.rom_bank)
            }
            MbcKind::Mbc3 | MbcKind::Mbc5 => (0, r.rom_bank),
        };
        // Bank numbers wrap when they exceed the ROM size (the high pins aren't connected).
        let banks = self.rom_banks();
        (low % banks, high % banks)
    }

    /// Offset into `ram` for an address in 0xA000-0xBFFF, or None if RAM is unavailable.
    fn ram_offset(&self, addr: u16) -> Option<usize> {
        if self.ram.is_empty() {
            return None;
        }
        let bank = match self.kind {
            MbcKind::None => 0,
            _ if !self.regs.ram_enabled => return None,
            MbcKind::Mbc1 if self.regs.mode => self.regs.upper,
            MbcKind::Mbc1 => 0,
            MbcKind::Mbc3 if self.regs.upper >= 0x08 => return None, // RTC selected
            MbcKind::Mbc3 | MbcKind::Mbc5 => self.regs.upper,
        };
        let offset = (bank % self.ram_banks()) * RAM_BANK_SIZE + (addr as usize - 0xA000);
        Some(offset % self.ram.len())
    }

    /// Reads 0x0000-0x7FFF.
    pub fn read_rom(&self, addr: u16) -> u8 {
        let (low, high) = self.rom_bank_numbers();
        let addr = addr as usize;
        match addr {
            0x0000..=0x3FFF => self.rom[low * ROM_BANK_SIZE + addr],
            _ => self.rom[high * ROM_BANK_SIZE + (addr - ROM_BANK_SIZE)],
        }
    }

    /// Writes to 0x0000-0x7FFF set MBC registers. They never change ROM.
    pub fn write_rom(&mut self, addr: u16, value: u8) {
        let r = &mut self.regs;
        match (self.kind, addr) {
            (MbcKind::None, _) => {}

            (_, 0x0000..=0x1FFF) => r.ram_enabled = value & 0x0F == 0x0A,

            (MbcKind::Mbc1, 0x2000..=0x3FFF) => {
                // 5-bit register; 0 is treated as 1 (so bank 0x20 can't be selected here)
                r.rom_bank = (value & 0x1F).max(1) as usize;
            }
            (MbcKind::Mbc1, 0x4000..=0x5FFF) => r.upper = (value & 0x03) as usize,
            (MbcKind::Mbc1, _) => r.mode = value & 0x01 != 0,

            (MbcKind::Mbc3, 0x2000..=0x3FFF) => r.rom_bank = (value & 0x7F).max(1) as usize,
            (MbcKind::Mbc3, 0x4000..=0x5FFF) => r.upper = value as usize,
            (MbcKind::Mbc3, _) => {} // RTC latch: the clock isn't emulated

            (MbcKind::Mbc5, 0x2000..=0x2FFF) => r.rom_bank = (r.rom_bank & 0x100) | value as usize,
            (MbcKind::Mbc5, 0x3000..=0x3FFF) => {
                r.rom_bank = (r.rom_bank & 0xFF) | ((value as usize & 1) << 8)
            }
            (MbcKind::Mbc5, 0x4000..=0x5FFF) => r.upper = (value & 0x0F) as usize,
            (MbcKind::Mbc5, _) => {}
        }
    }

    /// Reads 0xA000-0xBFFF. Disabled or missing RAM reads 0xFF.
    pub fn read_ram(&self, addr: u16) -> u8 {
        if let Some(i) = self.rtc_register() {
            return self.regs.rtc[i];
        }
        self.ram_offset(addr).map_or(0xFF, |i| self.ram[i])
    }

    /// Writes 0xA000-0xBFFF. Ignored while RAM is disabled.
    pub fn write_ram(&mut self, addr: u16, value: u8) {
        if let Some(i) = self.rtc_register() {
            self.regs.rtc[i] = value;
        } else if let Some(i) = self.ram_offset(addr) {
            self.ram[i] = value;
        }
    }

    /// MBC3 maps an RTC register into 0xA000-0xBFFF when 0x08-0x0C is selected.
    fn rtc_register(&self) -> Option<usize> {
        let r = &self.regs;
        (self.kind == MbcKind::Mbc3 && r.ram_enabled && (0x08..=0x0C).contains(&r.upper))
            .then(|| r.upper - 0x08)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ROM with `banks` 16KB banks, each byte holding its bank number, and the given
    /// header type / RAM size.
    fn banked_rom(banks: usize, cart_type: u8, ram_size: u8) -> Vec<u8> {
        let mut rom: Vec<u8> = (0..banks * ROM_BANK_SIZE)
            .map(|i| (i / ROM_BANK_SIZE) as u8)
            .collect();
        rom[HEADER_TYPE] = cart_type;
        rom[HEADER_RAM_SIZE] = ram_size;
        rom
    }

    #[test]
    fn rom_only_maps_both_banks_and_ignores_writes() {
        let mut cart = Cartridge::new(banked_rom(2, 0x00, 0));
        cart.write_rom(0x2000, 5);
        assert_eq!(cart.read_rom(0x0000), 0);
        assert_eq!(cart.read_rom(0x4000), 1);
    }

    #[test]
    fn mbc1_switches_rom_bank() {
        let mut cart = Cartridge::new(banked_rom(8, 0x01, 0));
        assert_eq!(cart.read_rom(0x4000), 1, "bank 1 by default");
        cart.write_rom(0x2000, 5);
        assert_eq!(cart.read_rom(0x4000), 5);
        assert_eq!(cart.read_rom(0x0000), 0, "bank 0 stays fixed");
    }

    #[test]
    fn mbc1_bank_0_selects_bank_1() {
        let mut cart = Cartridge::new(banked_rom(8, 0x01, 0));
        cart.write_rom(0x2000, 0);
        assert_eq!(cart.read_rom(0x4000), 1);
    }

    #[test]
    fn mbc1_bank_number_wraps_to_rom_size() {
        let mut cart = Cartridge::new(banked_rom(4, 0x01, 0));
        cart.write_rom(0x2000, 6); // only 4 banks: 6 % 4 = 2
        assert_eq!(cart.read_rom(0x4000), 2);
    }

    #[test]
    fn mbc1_upper_bits_extend_rom_bank() {
        let mut cart = Cartridge::new(banked_rom(64, 0x01, 0));
        cart.write_rom(0x2000, 0x03);
        cart.write_rom(0x4000, 0x01);
        assert_eq!(cart.read_rom(0x4000), 0x23);
        assert_eq!(
            cart.read_rom(0x0000),
            0,
            "simple mode keeps bank 0 at 0x0000"
        );
        cart.write_rom(0x6000, 1);
        assert_eq!(
            cart.read_rom(0x0000),
            0x20,
            "advanced mode banks 0x0000 too"
        );
    }

    #[test]
    fn mbc1_ram_needs_enabling() {
        let mut cart = Cartridge::new(banked_rom(4, 0x03, 0x03));
        cart.write_ram(0xA000, 0x42);
        assert_eq!(cart.read_ram(0xA000), 0xFF, "disabled RAM reads 0xFF");
        cart.write_rom(0x0000, 0x0A);
        cart.write_ram(0xA000, 0x42);
        assert_eq!(cart.read_ram(0xA000), 0x42);
        cart.write_rom(0x0000, 0x00);
        assert_eq!(cart.read_ram(0xA000), 0xFF);
    }

    #[test]
    fn mbc1_ram_banks_only_switch_in_advanced_mode() {
        let mut cart = Cartridge::new(banked_rom(4, 0x03, 0x03));
        cart.write_rom(0x0000, 0x0A);
        cart.write_ram(0xA000, 0x11);
        cart.write_rom(0x4000, 0x01);
        assert_eq!(cart.read_ram(0xA000), 0x11, "simple mode: still bank 0");
        cart.write_rom(0x6000, 1);
        cart.write_ram(0xA000, 0x22);
        cart.write_rom(0x4000, 0x00);
        assert_eq!(cart.read_ram(0xA000), 0x11);
        cart.write_rom(0x4000, 0x01);
        assert_eq!(cart.read_ram(0xA000), 0x22);
    }

    #[test]
    fn mbc3_7_bit_rom_bank_and_ram_banks() {
        let mut cart = Cartridge::new(banked_rom(128, 0x13, 0x03));
        cart.write_rom(0x2000, 0x45);
        assert_eq!(cart.read_rom(0x4000), 0x45);
        cart.write_rom(0x0000, 0x0A);
        cart.write_rom(0x4000, 0x02);
        cart.write_ram(0xA000, 0x77);
        cart.write_rom(0x4000, 0x00);
        assert_ne!(cart.read_ram(0xA000), 0x77);
        cart.write_rom(0x4000, 0x02);
        assert_eq!(cart.read_ram(0xA000), 0x77);
    }

    #[test]
    fn mbc5_9_bit_rom_bank_and_bank_0() {
        let mut cart = Cartridge::new(banked_rom(512, 0x19, 0));
        cart.write_rom(0x2000, 0x00);
        assert_eq!(cart.read_rom(0x4000), 0, "MBC5 can map bank 0 at 0x4000");
        cart.write_rom(0x2000, 0x05);
        cart.write_rom(0x3000, 0x01);
        assert_eq!(cart.read_rom(0x4000), (0x105 % 256) as u8);
        assert_eq!(cart.rom_bank_numbers().1, 0x105);
    }

    #[test]
    fn battery_ram_round_trip() {
        let mut cart = Cartridge::new(banked_rom(4, 0x03, 0x02));
        cart.write_rom(0x0000, 0x0A);
        cart.write_ram(0xA123, 0x99);
        let save = cart.battery_ram().expect("battery cart").to_vec();

        let mut fresh = Cartridge::new(banked_rom(4, 0x03, 0x02));
        assert!(fresh.load_battery_ram(&save));
        fresh.write_rom(0x0000, 0x0A);
        assert_eq!(fresh.read_ram(0xA123), 0x99);
    }

    #[test]
    fn no_battery_means_no_save() {
        let cart = Cartridge::new(banked_rom(4, 0x01, 0x02));
        assert!(cart.battery_ram().is_none());
    }
}
