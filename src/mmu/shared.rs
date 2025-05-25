pub struct io_register {
    gamepad_input: u8,
    serial_transfer: [u8; 2],
    div_reg: u8,
    tima: u8,
    tma: u8,
    tac: u8,
    interrupts: u8,
    audio: [u8; 0x16],
    lcd_control: u8,
    lcd_status: u8,
    scroll_y: u8,
    scroll_x: u8,
    ly_coord: u8,
    ly_compare: u8,
    dma_transfer: u8,
    bg_palette: u8,
    obj_palette: [u8; 2],
    window_y: u8,
    window_x: u8,
}

impl io_register {
    pub fn match_io_register(&mut self, address: u16, value: u8) {
        match address {
            0xFF00 => self.gamepad_input = value,
            0xFF01..=0xFF02 => self.serial_transfer[(address - 0xFF01) as usize] = value,
            0xFF04 => self.div_reg = value,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            0xFF0F => self.interrupts = value,
            0xFF10..=0xFF25 => self.audio[(address - 0xFF10) as usize] = value,
            0xFF40 => self.lcd_control = value,
            0xFF41 => self.lcd_status = value,
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => self.ly_coord = value,
            0xFF45 => self.ly_compare = value,
            0xFF46 => self.dma_transfer = value,
            0xFF47 => self.bg_palette = value,
            0xFF48..=0xFF49 => self.obj_palette[(address - 0xFF48) as usize] = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            _ => panic!("Unhandled IO register write: {:04X}", address),
        }
    }

    pub fn get_io_register(&self, address: u16) -> u8 {
        match address {
            0xFF00 => self.gamepad_input,
            0xFF01..=0xFF02 => self.serial_transfer[(address - 0xFF01) as usize],
            0xFF04 => self.div_reg,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            0xFF0F => self.interrupts,
            0xFF10..=0xFF25 => self.audio[(address - 0xFF10) as usize],
            0xFF40 => self.lcd_control,
            0xFF41 => self.lcd_status,
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly_coord,
            0xFF45 => self.ly_compare,
            0xFF46 => self.dma_transfer,
            0xFF47 => self.bg_palette,
            0xFF48..=0xFF49 => self.obj_palette[(address - 0xFF48) as usize],
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _ => panic!("Unhandled IO register read: {:04X}", address),
        }
    }
}
