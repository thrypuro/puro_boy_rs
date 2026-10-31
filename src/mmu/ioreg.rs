// IF bits requested by the IO registers
pub const INT_TIMER: u8 = 1 << 2;
pub const INT_SERIAL: u8 = 1 << 3;

const IO_START: usize = 0xFF00;
const IO_SIZE: usize = (0xFF7F - 0xFF00) + 1;

/// Cycles between TIMA overflowing and TMA being reloaded / the interrupt firing.
const TIMA_RELOAD_DELAY: u8 = 4;

/// Everything in 0xFF00-0xFF7F except the LCD registers (PPU) and DMA (MMU).
#[derive(Debug, PartialEq, Eq)]
pub struct IOReg {
    // Joypad select bits (4-5) of 0xFF00. No buttons are wired up yet.
    pub joypad: u8,
    // Serial data (0xFF01) and control (0xFF02)
    sb: u8,
    sc: u8,
    serial_out: String,
    // Timer: DIV (0xFF04) is the top byte of this 16 bit counter
    sys_counter: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    // Cycles left until a pending TIMA overflow reloads TMA
    tima_reload: Option<u8>,
    // Interrupt flag (0xFF0F)
    pub if_reg: u8,
    // Registers we don't emulate yet (sound, etc.) are just stored
    other: [u8; IO_SIZE],
}

impl IOReg {
    pub fn new() -> Self {
        IOReg {
            joypad: 0x30,
            sb: 0,
            sc: 0,
            serial_out: String::new(),
            sys_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            tima_reload: None,
            if_reg: 0,
            other: [0u8; IO_SIZE],
        }
    }

    pub fn serial_output(&self) -> &str {
        &self.serial_out
    }

    pub fn read(&self, address: usize) -> u8 {
        match address {
            // Nothing pressed: the low nibble reads 1s
            0xFF00 => 0xC0 | (self.joypad & 0x30) | 0x0F,
            0xFF01 => self.sb,
            0xFF02 => self.sc | 0x7E,
            0xFF04 => (self.sys_counter >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8,
            0xFF0F => self.if_reg | 0xE0,
            _ => self.other[address - IO_START],
        }
    }

    pub fn write(&mut self, address: usize, value: u8) {
        match address {
            0xFF00 => self.joypad = value & 0x30,
            0xFF01 => self.sb = value,
            0xFF02 => {
                self.sc = value & 0x81;
                // Transfers with the internal clock complete instantly
                if value & 0x81 == 0x81 {
                    self.serial_out.push(self.sb as char);
                    self.sb = 0xFF;
                    self.sc &= 0x01;
                    self.if_reg |= INT_SERIAL;
                }
            }
            0xFF04 => self.set_sys_counter(0),
            0xFF05 => {
                // Writing TIMA during the reload delay cancels the reload
                self.tima = value;
                self.tima_reload = None;
            }
            0xFF06 => self.tma = value,
            0xFF07 => {
                let before = self.timer_bit();
                self.tac = value & 0x07;
                self.check_falling_edge(before);
            }
            0xFF0F => self.if_reg = value & 0x1F,
            _ => self.other[address - IO_START] = value,
        }
    }

    /// Advances the timer by `cycles` T-cycles, setting IF bit 2 on overflow.
    pub fn tick(&mut self, cycles: u8) {
        for _ in 0..cycles {
            if let Some(left) = self.tima_reload {
                if left <= 1 {
                    self.tima = self.tma;
                    self.if_reg |= INT_TIMER;
                    self.tima_reload = None;
                } else {
                    self.tima_reload = Some(left - 1);
                }
            }
            self.set_sys_counter(self.sys_counter.wrapping_add(1));
        }
    }

    /// The counter bit TIMA watches, ANDed with the timer enable bit.
    fn timer_bit(&self) -> bool {
        let bit = match self.tac & 0b11 {
            0b00 => 9, // 1024 cycles
            0b01 => 3, // 16 cycles
            0b10 => 5, // 64 cycles
            _ => 7,    // 256 cycles
        };
        self.tac & 0b100 != 0 && (self.sys_counter >> bit) & 1 != 0
    }

    fn set_sys_counter(&mut self, value: u16) {
        let before = self.timer_bit();
        self.sys_counter = value;
        self.check_falling_edge(before);
    }

    /// TIMA increments when the watched bit goes from 1 to 0.
    fn check_falling_edge(&mut self, before: bool) {
        if before && !self.timer_bit() {
            let (tima, overflow) = self.tima.overflowing_add(1);
            self.tima = tima;
            if overflow {
                self.tima_reload = Some(TIMA_RELOAD_DELAY);
            }
        }
    }
}
