use sdl2::event::{self, Event};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, Texture, WindowCanvas};
const width: u32 = 128;
const height: u32 = 128;
pub const GAMEBOY_COLORS: [sdl2::pixels::Color; 4] = [
    Color::RGB(255, 255, 255),
    Color::RGB(192, 192, 192),
    Color::RGB(96, 96, 96),
    Color::RGB(0, 0, 0),
];
#[derive(Debug, PartialEq, Eq)]
pub struct tile {
    pub data: [[u8; 8]; 8],
}
#[derive(Debug, PartialEq, Eq)]
pub struct ppu {
    vram: Vec<u8>,
    pub tiles: Vec<tile>,
    tilemap: [[u8; 32]; 32],
}

impl ppu {
    pub fn new() -> Self {
        ppu {
            vram: vec![0; 8192],
            tiles: Vec::new(),
            tilemap: [[0; 32]; 32],
        }
    }

    pub fn push_tile(&mut self, t: tile) {
        self.tiles.push(t);
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

pub fn loadtileset(rom: &Vec<u8>) -> ppu {
    let mut tileset = ppu::new();
    for i in (0..(384)).step_by(16) {
        let t = tile {
            data: get_tile(&rom[i..i + 16]),
        };
        tileset.push_tile(t);
    }
    tileset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel() {
        assert_eq!(get_pixelrow(0x7c, 0x7c), [0, 3, 3, 3, 3, 3, 0, 0]);
    }

    #[test]
    fn get_pixel_a() {
        let a = [
            0x7C, 0x7C, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0xFE, 0xC6, 0xC6, 0x00, 0xC6, 0xC6, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(
            get_tile(&a),
            [
                [0, 3, 3, 3, 3, 3, 0, 0],
                [2, 2, 0, 0, 0, 2, 2, 0],
                [1, 1, 0, 0, 0, 1, 1, 0],
                [2, 2, 2, 2, 2, 2, 2, 0],
                [3, 3, 0, 0, 0, 3, 3, 0],
                [2, 2, 0, 0, 0, 2, 2, 0],
                [1, 1, 0, 0, 0, 1, 1, 0],
                [0, 0, 0, 0, 0, 0, 0, 0]
            ]
        );
    }
}
