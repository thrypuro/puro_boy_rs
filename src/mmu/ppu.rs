use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::render::WindowCanvas;

pub const GAMEBOY_COLORS: [sdl3::pixels::Color; 4] = [
    Color::RGB(255, 255, 255),
    Color::RGB(192, 192, 192),
    Color::RGB(96, 96, 96),
    Color::RGB(0, 0, 0),
];

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = (0xFE9F - 0xFE00) + 1;

const VRAM_START: usize = 0x8000;
const OAM_START: usize = 0xFE00;

// Timing, in T-cycles
const OAM_SCAN_CYCLES: u16 = 80;
const DRAWING_CYCLES: u16 = 172;
const CYCLES_PER_LINE: u16 = 456;
const LINES_PER_FRAME: u8 = 154;

// IF bits the PPU can request
const INT_VBLANK: u8 = 1 << 0;
const INT_STAT: u8 = 1 << 1;

// LCDC bits
const LCDC_LCD_ON: u8 = 1 << 7;
const LCDC_WIN_MAP: u8 = 1 << 6;
const LCDC_WIN_ON: u8 = 1 << 5;
const LCDC_TILES_8000: u8 = 1 << 4;
const LCDC_BG_MAP: u8 = 1 << 3;
const LCDC_OBJ_TALL: u8 = 1 << 2;
const LCDC_OBJ_ON: u8 = 1 << 1;
const LCDC_BG_ON: u8 = 1 << 0;

// STAT interrupt enable bits
const STAT_LYC_INT: u8 = 1 << 6;
const STAT_OAM_INT: u8 = 1 << 5;
const STAT_VBLANK_INT: u8 = 1 << 4;
const STAT_HBLANK_INT: u8 = 1 << 3;

// OAM attribute bits
const OBJ_BEHIND_BG: u8 = 1 << 7;
const OBJ_Y_FLIP: u8 = 1 << 6;
const OBJ_X_FLIP: u8 = 1 << 5;
const OBJ_PALETTE_1: u8 = 1 << 4;

const MAX_SPRITES_PER_LINE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tile {
    pub data: [[u8; 8]; 8],
}
#[derive(Debug, PartialEq, Eq)]
pub struct PPU {
    pub tiles: Vec<Tile>,
    // Vram to store the pixel
    vram: [u8; VRAM_SIZE],
    // Object Attribute Memory
    oam: [u8; OAM_SIZE],

    // LCD registers (0xFF40-0xFF4B, minus DMA at 0xFF46)
    lcdc: u8,
    stat: u8, // only the interrupt enable bits (3-6) are stored
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,

    mode: Mode,
    // Cycles into the current line
    line_cycles: u16,
    // Window has its own line counter that only advances on lines it was drawn
    window_line: u8,
    // STAT interrupt fires on the rising edge of this
    stat_line: bool,

    framebuffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            tiles: Vec::new(),
            vram: [0u8; VRAM_SIZE],
            oam: [0u8; OAM_SIZE],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            mode: Mode::HBlank,
            line_cycles: 0,
            window_line: 0,
            stat_line: false,
            framebuffer: [0u8; SCREEN_WIDTH * SCREEN_HEIGHT],
        }
    }

    pub fn push_tile(&mut self, t: Tile) {
        self.tiles.push(t);
    }

    pub fn turn_lcd_on(&mut self) {
        self.write(0xFF40, self.lcdc | LCDC_LCD_ON);
    }

    fn lcd_on(&self) -> bool {
        self.lcdc & LCDC_LCD_ON != 0
    }

    /// Reads VRAM (0x8000-0x9FFF), OAM (0xFE00-0xFE9F) or an LCD register (0xFF40-0xFF4B).
    pub fn read(&self, address: usize) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[address - VRAM_START],
            0xFE00..=0xFE9F => self.oam[address - OAM_START],
            0xFF40 => self.lcdc,
            0xFF41 => {
                let coincidence = if self.ly == self.lyc { 1 << 2 } else { 0 };
                0x80 | self.stat | coincidence | self.mode as u8
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF, // Should never get here
        }
    }
    /// Writes VRAM (0x8000-0x9FFF), OAM (0xFE00-0xFE9F) or an LCD register (0xFF40-0xFF4B).
    pub fn write(&mut self, address: usize, value: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[address - VRAM_START] = value,
            0xFE00..=0xFE9F => self.oam[address - OAM_START] = value,
            0xFF40 => {
                let was_on = self.lcd_on();
                self.lcdc = value;
                if was_on != self.lcd_on() {
                    // Both switching on and off restart from the top of the frame
                    self.ly = 0;
                    self.line_cycles = 0;
                    self.window_line = 0;
                    self.stat_line = false;
                    self.mode = if self.lcd_on() {
                        Mode::OamScan
                    } else {
                        Mode::HBlank
                    };
                }
            }
            0xFF41 => self.stat = value & 0b0111_1000,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // LY is read only
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {} // Should never get here
        }
    }
    /// Advances the PPU by `cycles` T-cycles. Returns the interrupts it requests,
    /// using the IF bit layout: bit 0 = VBlank, bit 1 = STAT.
    pub fn tick(&mut self, cycles: u8) -> u8 {
        if !self.lcd_on() {
            return 0;
        }
        let mut requested = 0;
        for _ in 0..cycles {
            requested |= self.step();
        }
        requested
    }

    /// Advances one T-cycle.
    fn step(&mut self) -> u8 {
        let mut requested = 0;
        self.line_cycles += 1;

        if self.line_cycles == CYCLES_PER_LINE {
            self.line_cycles = 0;
            self.ly = (self.ly + 1) % LINES_PER_FRAME;
            if self.ly == SCREEN_HEIGHT as u8 {
                self.mode = Mode::VBlank;
                requested |= INT_VBLANK;
            } else if self.ly == 0 {
                self.window_line = 0;
                self.mode = Mode::OamScan;
            } else if self.ly < SCREEN_HEIGHT as u8 {
                self.mode = Mode::OamScan;
            }
        } else if self.ly < SCREEN_HEIGHT as u8 {
            if self.line_cycles == OAM_SCAN_CYCLES {
                self.mode = Mode::Drawing;
            } else if self.line_cycles == OAM_SCAN_CYCLES + DRAWING_CYCLES {
                self.render_scanline();
                self.mode = Mode::HBlank;
            }
        }

        let stat_line = self.stat_condition();
        if stat_line && !self.stat_line {
            requested |= INT_STAT;
        }
        self.stat_line = stat_line;
        requested
    }

    /// The OR of every enabled STAT interrupt source.
    fn stat_condition(&self) -> bool {
        (self.stat & STAT_LYC_INT != 0 && self.ly == self.lyc)
            || match self.mode {
                Mode::HBlank => self.stat & STAT_HBLANK_INT != 0,
                Mode::VBlank => self.stat & STAT_VBLANK_INT != 0,
                Mode::OamScan => self.stat & STAT_OAM_INT != 0,
                Mode::Drawing => false,
            }
    }

    /// 160x144 shades (0 = white .. 3 = black) after palettes are applied, row-major.
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }

    /// Colour index (0-3) of pixel (x, y) in the tile at `tile_addr`.
    fn tile_pixel(&self, tile_addr: usize, x: u8, y: u8) -> u8 {
        let row = tile_addr - VRAM_START + y as usize * 2;
        let lo = self.vram[row] >> (7 - x) & 1;
        let hi = self.vram[row + 1] >> (7 - x) & 1;
        hi << 1 | lo
    }

    /// Address of a BG/window tile, honouring LCDC.4's signed addressing mode.
    fn bg_tile_addr(&self, index: u8) -> usize {
        if self.lcdc & LCDC_TILES_8000 != 0 {
            VRAM_START + index as usize * 16
        } else {
            (0x9000 + (index as i8 as isize) * 16) as usize
        }
    }

    /// Colour index at (x, y) of the 256x256 BG/window map at `map_base`.
    fn map_pixel(&self, map_base: usize, x: u8, y: u8) -> u8 {
        let map_addr = map_base + (y as usize / 8) * 32 + x as usize / 8;
        let index = self.vram[map_addr - VRAM_START];
        self.tile_pixel(self.bg_tile_addr(index), x % 8, y % 8)
    }

    fn render_scanline(&mut self) {
        let ly = self.ly;
        // Colour indices before the palette, needed for sprite priority
        let mut bg_colours = [0u8; SCREEN_WIDTH];

        if self.lcdc & LCDC_BG_ON != 0 {
            let bg_map = if self.lcdc & LCDC_BG_MAP != 0 { 0x9C00 } else { 0x9800 };
            let y = ly.wrapping_add(self.scy);
            for (x, colour) in bg_colours.iter_mut().enumerate() {
                *colour = self.map_pixel(bg_map, (x as u8).wrapping_add(self.scx), y);
            }

            let window_x = self.wx as i16 - 7;
            if self.lcdc & LCDC_WIN_ON != 0 && ly >= self.wy && window_x < SCREEN_WIDTH as i16 {
                let win_map = if self.lcdc & LCDC_WIN_MAP != 0 { 0x9C00 } else { 0x9800 };
                for x in window_x.max(0)..SCREEN_WIDTH as i16 {
                    bg_colours[x as usize] =
                        self.map_pixel(win_map, (x - window_x) as u8, self.window_line);
                }
                self.window_line += 1;
            }
        }

        let row = ly as usize * SCREEN_WIDTH;
        for (x, &colour) in bg_colours.iter().enumerate() {
            self.framebuffer[row + x] = apply_palette(self.bgp, colour);
        }

        if self.lcdc & LCDC_OBJ_ON != 0 {
            self.render_sprites(&bg_colours);
        }
    }

    fn render_sprites(&mut self, bg_colours: &[u8; SCREEN_WIDTH]) {
        let ly = self.ly as i16;
        let height: i16 = if self.lcdc & LCDC_OBJ_TALL != 0 { 16 } else { 8 };

        // OAM scan: the first 10 sprites in OAM order that cover this line
        let mut sprites: Vec<usize> = (0..40)
            .map(|i| i * 4)
            .filter(|&o| {
                let top = self.oam[o] as i16 - 16;
                ly >= top && ly < top + height
            })
            .take(MAX_SPRITES_PER_LINE)
            .collect();
        // Lower X wins, ties go to whichever is first in OAM (sort is stable)
        sprites.sort_by_key(|&o| self.oam[o + 1]);

        let row = ly as usize * SCREEN_WIDTH;
        for x in 0..SCREEN_WIDTH as i16 {
            for &o in &sprites {
                let left = self.oam[o + 1] as i16 - 8;
                if x < left || x >= left + 8 {
                    continue;
                }
                let flags = self.oam[o + 3];
                let mut tile_y = ly - (self.oam[o] as i16 - 16);
                if flags & OBJ_Y_FLIP != 0 {
                    tile_y = height - 1 - tile_y;
                }
                let mut tile_x = x - left;
                if flags & OBJ_X_FLIP != 0 {
                    tile_x = 7 - tile_x;
                }
                let mut tile = self.oam[o + 2];
                if height == 16 {
                    tile &= 0xFE;
                }
                let tile_addr = VRAM_START + tile as usize * 16;
                let colour = self.tile_pixel(tile_addr, tile_x as u8, tile_y as u8);
                if colour == 0 {
                    continue; // transparent, let a lower priority sprite show
                }
                if flags & OBJ_BEHIND_BG == 0 || bg_colours[x as usize] == 0 {
                    let palette = if flags & OBJ_PALETTE_1 != 0 { self.obp1 } else { self.obp0 };
                    self.framebuffer[row + x as usize] = apply_palette(palette, colour);
                }
                break;
            }
        }
    }
}

fn apply_palette(palette: u8, colour: u8) -> u8 {
    (palette >> (colour * 2)) & 0b11
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
