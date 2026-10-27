use crate::error::{EmuError, EmuRes};
use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::render::WindowCanvas;

pub const GAMEBOY_COLORS: [sdl3::pixels::Color; 4] = [
    Color::RGB(255, 255, 255),
    Color::RGB(192, 192, 192),
    Color::RGB(96, 96, 96),
    Color::RGB(0, 0, 0),
];

const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = (0xFE9F - 0xFE00) + 1;

const VRAM_START: usize = 0x8000;
const OAM_START: usize = 0xFE00;
#[derive(Debug, PartialEq, Eq)]
pub struct Tile {
    pub data: [[u8; 8]; 8],
}
#[derive(Debug, PartialEq, Eq)]
pub struct PPU {
    pub tiles: Vec<Tile>,
    tilemap: [[u8; 32]; 32],
    scan_line: u8,
    // Vram to store the pixel
    vram: [u8; VRAM_SIZE],
    // Object Attribute Memory
    oam: [u8; OAM_SIZE],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            tiles: Vec::new(),
            tilemap: [[0; 32]; 32],
            scan_line: 0,
            vram: [0u8; VRAM_SIZE],
            oam: [0u8; OAM_SIZE],
        }
    }

    pub fn push_tile(&mut self, t: Tile) {
        self.tiles.push(t);
    }

    pub fn turn_lcd_on(&mut self) {}

    /// Reads VRAM (0x8000-0x9FFF), OAM (0xFE00-0xFE9F) or an LCD register (0xFF40-0xFF4B).
    pub fn read(&self, address: usize) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[address - VRAM_START],
            0xFE00..=0xFE9F => self.oam[address - OAM_START],
            _ => 0, // Should never get here
        }
    }
    /// Writes VRAM (0x8000-0x9FFF), OAM (0xFE00-0xFE9F) or an LCD register (0xFF40-0xFF4B).
    pub fn write(&mut self, address: usize, value: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[address - VRAM_START] = value,
            0xFE00..=0xFE9F => self.oam[address - OAM_START] = value,
            _ => {} // Should never get here
        }
    }
    /// Advances the PPU by `cycles` T-cycles. Returns the interrupts it requests,
    /// using the IF bit layout: bit 0 = VBlank, bit 1 = STAT.
    pub fn tick(&mut self, cycles: u8) -> u8 {
        todo!("PPU::tick")
    }
    /// 160x144 shades (0 = white .. 3 = black) after palettes are applied, row-major.
    pub fn framebuffer(&self) -> &[u8] {
        todo!("PPU::framebuffer")
    }
}

pub fn render_tile(tile: [[u8; 8]; 8], canvas: &mut WindowCanvas, position: Point) {
    // Draw the 8x8 tile
    for y in 0..8u32 {
        for x in 0..8u32 {
            let ci = tile[y as usize][x as usize];
            let c_col = GAMEBOY_COLORS[ci as usize];
            canvas.set_draw_color(c_col);
            let p = Point::new(
                x as i32, // Center the tile
                y as i32,
            ) + position;
            canvas.draw_point(p).expect("Failed to draw point");
        }
    }

    // Present the canvas once per frame
    // canvas.present();
}

pub fn get_pixelrow(b1: u8, b2: u8) -> [u8; 8] {
    let mut c1: [u8; 8] = [0; 8];
    for i in 0..8 {
        let a1 = (b1 >> i) & 1;
        let a2 = (b2 >> i) & 1;
        c1[7 - i] = a2 * 2 + a1;
    }
    c1
}

pub fn get_tile(t1: &[u8]) -> [[u8; 8]; 8] {
    let mut i = 0;
    let mut cs = [[0u8; 8]; 8];
    while i < 16 {
        cs[i / 2] = get_pixelrow(t1[i], t1[i + 1]);
        i += 2;
    }
    println!("cs {:?}", cs);
    cs
}

// pub fn loadtileset(rom: &Vec<u8>) -> ppu {
//     let mut tileset = ppu::new();
//     for i in (0..(384)).step_by(16) {
//         let t = tile {
//             data: get_tile(&rom[i..i + 16]),
//         };
//         tileset.push_tile(t);
//     }
//     tileset
// }

#[cfg(test)]
mod tests;
