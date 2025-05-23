use sdl3::pixels::Color;
use sdl3::rect::Point;
use sdl3::render::WindowCanvas;

// Constants
pub const SCREEN_WIDTH: u32 = 160;
pub const SCREEN_HEIGHT: u32 = 144;
pub const TILE_SIZE: u32 = 8;

// Game Boy has 4 shades of "color"
pub const GAMEBOY_COLORS: [sdl3::pixels::Color; 4] = [
    Color::RGB(255, 255, 255), // White
    Color::RGB(192, 192, 192), // Light gray
    Color::RGB(96, 96, 96),    // Dark gray
    Color::RGB(0, 0, 0),       // Black
];

// PPU Mode constants
const MODE_HBLANK: u8 = 0;
const MODE_VBLANK: u8 = 1;
const MODE_OAM_SCAN: u8 = 2;
const MODE_DRAWING: u8 = 3;

// Timing constants (in CPU cycles)
const OAM_SCAN_CYCLES: u32 = 80;
const DRAWING_CYCLES: u32 = 172;
const HBLANK_CYCLES: u32 = 204;
const SCANLINE_CYCLES: u32 = OAM_SCAN_CYCLES + DRAWING_CYCLES + HBLANK_CYCLES;
const VBLANK_CYCLES: u32 = SCANLINE_CYCLES * 10;
const FRAME_CYCLES: u32 = SCANLINE_CYCLES * 154;

#[derive(Debug, PartialEq, Eq)]
pub struct Tile {
    pub data: [[u8; 8]; 8],
}

#[derive(Debug, PartialEq, Eq)]
pub struct PPU {
    // Tile data (0x8000-0x97FF)
    pub tiles: Vec<Tile>,

    // Background tile maps (0x9800-0x9FFF)
    bg_tilemap: [[u8; 32]; 32],
    window_tilemap: [[u8; 32]; 32],

    // Current state
    mode: u8,
    scan_line: u8,
    cycle_counter: u32,

    // LCD Control Register (0xFF40)
    lcd_enabled: bool,
    window_tile_map: bool, // false=0x9800-0x9BFF, true=0x9C00-0x9FFF
    window_enabled: bool,
    bg_window_tile_data: bool, // false=0x8800-0x97FF, true=0x8000-0x8FFF
    bg_tile_map: bool,         // false=0x9800-0x9BFF, true=0x9C00-0x9FFF
    sprite_size: bool,         // false=8x8, true=8x16
    sprites_enabled: bool,
    bg_window_priority: bool, // false=off, true=on

    // LCD Status Register (0xFF41)
    lyc_interrupt: bool,
    oam_interrupt: bool,
    vblank_interrupt: bool,
    hblank_interrupt: bool,
    lyc_equal: bool,

    // Position and scrolling registers
    scroll_y: u8,   // 0xFF42
    scroll_x: u8,   // 0xFF43
    ly_compare: u8, // 0xFF45
    window_y: u8,   // 0xFF4A
    window_x: u8,   // 0xFF4B

    // Palette data
    bg_palette: u8,   // 0xFF47
    obj_palette0: u8, // 0xFF48
    obj_palette1: u8, // 0xFF49

    // Framebuffer
    framebuffer: [[u8; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],

    // VRAM
    pub vram: [u8; 0x2000],

    // OAM (Sprite Attribute Table)
    oam: [u8; 0xA0],

    // Flag to indicate if a frame is ready to be rendered
    frame_ready: bool,
    // Optional canvas for rendering (for testing purposes)
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            tiles: Vec::with_capacity(384),
            bg_tilemap: [[0; 32]; 32],
            window_tilemap: [[0; 32]; 32],
            mode: MODE_OAM_SCAN,
            scan_line: 0,
            cycle_counter: 0,
            lcd_enabled: false,
            window_tile_map: false,
            window_enabled: false,
            bg_window_tile_data: false,
            bg_tile_map: false,
            sprite_size: false,
            sprites_enabled: false,
            bg_window_priority: false,
            lyc_interrupt: false,
            oam_interrupt: false,
            vblank_interrupt: false,
            hblank_interrupt: false,
            lyc_equal: false,
            scroll_y: 0,
            scroll_x: 0,
            ly_compare: 0,
            window_y: 0,
            window_x: 0,
            bg_palette: 0xE4, // Default Game Boy palette
            obj_palette0: 0,
            obj_palette1: 0,
            framebuffer: [[0; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            frame_ready: false,
        }
    }

    /// Initialize PPU with ROM data
    pub fn init(&mut self, rom_bank0: &[u8], _rom_bank1: &[u8]) {}

    pub fn push_tile(&mut self, t: Tile) {
        self.tiles.push(t);
    }

    /// Update LCD Control Register (0xFF40)
    pub fn update_lcd_control(&mut self, value: u8) {
        self.lcd_enabled = (value & 0x80) != 0;
        self.window_tile_map = (value & 0x40) != 0;
        self.window_enabled = (value & 0x20) != 0;
        self.bg_window_tile_data = (value & 0x10) != 0;
        self.bg_tile_map = (value & 0x08) != 0;
        self.sprite_size = (value & 0x04) != 0;
        self.sprites_enabled = (value & 0x02) != 0;
        self.bg_window_priority = (value & 0x01) != 0;
    }

    /// Update LCD Status Register (0xFF41)
    pub fn update_lcd_status(&mut self, value: u8) {
        self.lyc_interrupt = (value & 0x40) != 0;
        self.oam_interrupt = (value & 0x20) != 0;
        self.vblank_interrupt = (value & 0x10) != 0;
        self.hblank_interrupt = (value & 0x08) != 0;
        // Mode and LYC equal bits are read-only
    }

    // Setter methods for PPU registers
    pub fn set_scroll_y(&mut self, value: u8) {
        self.scroll_y = value;
    }
    pub fn set_scroll_x(&mut self, value: u8) {
        self.scroll_x = value;
    }
    pub fn set_ly_compare(&mut self, value: u8) {
        self.ly_compare = value;
    }
    pub fn set_window_y(&mut self, value: u8) {
        self.window_y = value;
    }
    pub fn set_window_x(&mut self, value: u8) {
        self.window_x = value;
    }

    pub fn set_bg_palette(&mut self, value: u8) {
        self.bg_palette = value;
    }
    pub fn set_obj_palette0(&mut self, value: u8) {
        self.obj_palette0 = value;
    }
    pub fn set_obj_palette1(&mut self, value: u8) {
        self.obj_palette1 = value;
    }

    /// Get the color from a palette based on the color index
    fn get_color_from_palette(&self, palette: u8, color_idx: u8) -> u8 {
        let shift = color_idx * 2;
        (palette >> shift) & 0x03
    }

    /// Update a tile when VRAM is written to
    pub fn update_tile(&mut self, address: u16, value: u8) {
        todo!();
    }

    /// Update the PPU state for the given number of cycles
    /// Returns true if a frame is ready to be rendered
    pub fn update(&mut self, cycles: u32) -> bool {
        false
    }

    /// Render a single scanline to the framebuffer
    fn render_scanline(&mut self) {
        if self.bg_window_priority {
            self.render_background_scanline();
        }

        if self.window_enabled {
            self.render_window_scanline();
        }

        if self.sprites_enabled {
            self.render_sprites_scanline();
        }
    }

    /// Render the background layer for the current scanline
    fn render_background_scanline(&mut self) {
        todo!()
    }

    /// Render the window layer for the current scanline
    fn render_window_scanline(&mut self) {
        todo!()
    }

    /// Render sprites for the current scanline
    fn render_sprites_scanline(&mut self) {
        todo!()
    }

    /// Render the framebuffer to the provided canvas
    pub fn render(&mut self, canvas: &mut WindowCanvas) {
        todo!()
    }

    /// Check if a frame is ready to be rendered
    pub fn is_frame_ready(&self) -> bool {
        self.frame_ready
    }

    /// Turn on the LCD display
    pub fn turn_lcd_on(&mut self) {
        self.lcd_enabled = true;
    }

    /// Turn off the LCD display
    pub fn turn_lcd_off(&mut self) {
        self.lcd_enabled = false;
    }
}

/// Render a single tile to the canvas
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
}

/// Convert two bytes into a row of 8 pixels (2 bits per pixel)
pub fn get_pixelrow(b1: u8, b2: u8) -> [u8; 8] {
    let mut c1: [u8; 8] = [0; 8];
    for i in 0..8 {
        let a1 = (b1 >> (7 - i)) & 1;
        let a2 = (b2 >> (7 - i)) & 1;
        c1[i] = a2 * 2 + a1;
    }
    c1
}

/// Convert 16 bytes into an 8x8 tile
pub fn get_tile(t1: &[u8]) -> [[u8; 8]; 8] {
    let mut i = 0;
    let mut cs = [[0u8; 8]; 8];
    while i < 16 {
        cs[i / 2] = get_pixelrow(t1[i], t1[i + 1]);
        i += 2;
    }
    cs
}

/// Load a full tileset from ROM data
pub fn load_tileset(rom: &[u8]) -> Vec<Tile> {
    let mut tiles = Vec::new();
    let max_tiles = (rom.len() / 16).min(384); // Max 384 tiles or what fits in ROM

    for i in (0..max_tiles * 16).step_by(16) {
        if i + 16 <= rom.len() {
            let t = Tile {
                data: get_tile(&rom[i..i + 16]),
            };
            tiles.push(t);
        }
    }

    tiles
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
