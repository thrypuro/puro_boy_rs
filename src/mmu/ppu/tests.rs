//! PPU targets, roughly in the order you should make them pass:
//!
//! 1. `decode`    - tile bytes -> pixels (already passing)
//! 2. `registers` - VRAM/OAM/LCD registers read back what was written
//! 3. `timing`    - modes 2 -> 3 -> 0 per line, LY, VBlank (no drawing yet!)
//! 4. `stat`      - STAT register and STAT interrupt
//! 5. `background`, `window`, `sprites` - actual pixels in the framebuffer
//!
//! Run one group with e.g. `cargo test ppu::tests::timing`.
//!
//! Reference: https://gbdev.io/pandocs/Rendering.html

use super::*;

const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const BGP: u16 = 0xFF47;
const OBP0: u16 = 0xFF48;
const OBP1: u16 = 0xFF49;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B;

const CYCLES_PER_LINE: u32 = 456;
const CYCLES_PER_FRAME: u32 = CYCLES_PER_LINE * 154; // 70224

const INT_VBLANK: u8 = 1 << 0;
const INT_STAT: u8 = 1 << 1;

// LCDC bits
const LCD_ON: u8 = 1 << 7;
const WIN_MAP_9C00: u8 = 1 << 6;
const WIN_ON: u8 = 1 << 5;
const TILES_8000: u8 = 1 << 4;
const BG_MAP_9C00: u8 = 1 << 3;
const OBJ_8X16: u8 = 1 << 2;
const OBJ_ON: u8 = 1 << 1;
const BG_ON: u8 = 1 << 0;

/// Identity palette: colour index n -> shade n.
const IDENTITY_PALETTE: u8 = 0b11_10_01_00;

/// Ticks in 4-cycle chunks (the smallest CPU step), returning every interrupt requested.
fn run(ppu: &mut PPU, cycles: u32) -> u8 {
    assert!(cycles % 4 == 0);
    let mut requested = 0;
    for _ in 0..cycles / 4 {
        requested |= ppu.tick(4);
    }
    requested
}

fn mode(ppu: &PPU) -> u8 {
    ppu.read(STAT) & 0b11
}

fn pixel(ppu: &PPU, x: usize, y: usize) -> u8 {
    ppu.framebuffer()[y * 160 + x]
}

/// Writes a tile where every pixel has colour index `colour` (0-3) at `addr`.
fn write_solid_tile(ppu: &mut PPU, addr: u16, colour: u8) {
    let lo = if colour & 1 != 0 { 0xFF } else { 0x00 };
    let hi = if colour & 2 != 0 { 0xFF } else { 0x00 };
    for row in 0..8 {
        ppu.write(addr + row * 2, lo);
        ppu.write(addr + row * 2 + 1, hi);
    }
}

/// Writes a tile whose left column is colour 3 and the rest colour 0. Useful for flips/scroll.
fn write_left_stripe_tile(ppu: &mut PPU, addr: u16) {
    for row in 0..8 {
        ppu.write(addr + row * 2, 0b1000_0000);
        ppu.write(addr + row * 2 + 1, 0b1000_0000);
    }
}

/// PPU with identity palettes, the given LCDC (LCD forced on), ready to render.
fn ppu_with(lcdc: u8) -> PPU {
    let mut ppu = PPU::new();
    ppu.write(BGP, IDENTITY_PALETTE);
    ppu.write(OBP0, IDENTITY_PALETTE);
    ppu.write(OBP1, IDENTITY_PALETTE);
    ppu.write(LCDC, lcdc | LCD_ON);
    ppu
}

fn write_sprite(ppu: &mut PPU, index: u16, y: u8, x: u8, tile: u8, flags: u8) {
    let base = 0xFE00 + index * 4;
    ppu.write(base, y);
    ppu.write(base + 1, x);
    ppu.write(base + 2, tile);
    ppu.write(base + 3, flags);
}

mod decode {
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

mod registers {
    use super::*;

    #[test]
    fn vram_reads_back_what_was_written() {
        let mut ppu = PPU::new();
        ppu.write(0x8000, 0x12);
        ppu.write(0x9FFF, 0x34);
        assert_eq!(ppu.read(0x8000), 0x12);
        assert_eq!(ppu.read(0x9FFF), 0x34);
    }

    #[test]
    fn oam_reads_back_what_was_written() {
        let mut ppu = PPU::new();
        ppu.write(0xFE00, 0xAB);
        ppu.write(0xFE9F, 0xCD);
        assert_eq!(ppu.read(0xFE00), 0xAB);
        assert_eq!(ppu.read(0xFE9F), 0xCD);
    }

    #[test]
    fn lcd_registers_read_back_what_was_written() {
        let mut ppu = PPU::new();
        for (reg, value) in [
            (SCY, 0x11),
            (SCX, 0x22),
            (LYC, 0x33),
            (BGP, 0x44),
            (OBP0, 0x55),
            (OBP1, 0x66),
            (WY, 0x77),
            (WX, 0x88),
        ] {
            ppu.write(reg, value);
            assert_eq!(ppu.read(reg), value, "register {reg:#06X}");
        }
    }

    #[test]
    fn framebuffer_is_160_by_144() {
        let ppu = PPU::new();
        assert_eq!(ppu.framebuffer().len(), 160 * 144);
    }
}

mod timing {
    use super::*;

    #[test]
    fn lcd_off_does_nothing() {
        let mut ppu = PPU::new();
        ppu.write(LCDC, 0);
        let requested = run(&mut ppu, CYCLES_PER_FRAME);
        assert_eq!(requested, 0, "no interrupts while the LCD is off");
        assert_eq!(ppu.read(LY), 0, "LY stays 0 while the LCD is off");
        assert_eq!(mode(&ppu), 0, "mode reads 0 while the LCD is off");
    }

    #[test]
    fn line_starts_in_oam_scan() {
        let mut ppu = ppu_with(0);
        run(&mut ppu, 4);
        assert_eq!(mode(&ppu), 2);
    }

    #[test]
    fn oam_scan_lasts_80_cycles_then_drawing() {
        let mut ppu = ppu_with(0);
        run(&mut ppu, 76);
        assert_eq!(mode(&ppu), 2);
        run(&mut ppu, 8); // 84
        assert_eq!(mode(&ppu), 3);
    }

    #[test]
    fn drawing_lasts_172_cycles_then_hblank() {
        let mut ppu = ppu_with(0);
        run(&mut ppu, 248); // 80 + 168
        assert_eq!(mode(&ppu), 3);
        run(&mut ppu, 8); // 256 = 80 + 172 + 4
        assert_eq!(mode(&ppu), 0);
    }

    #[test]
    fn ly_increments_every_456_cycles() {
        let mut ppu = ppu_with(0);
        run(&mut ppu, 452);
        assert_eq!(ppu.read(LY), 0);
        run(&mut ppu, 8);
        assert_eq!(ppu.read(LY), 1);
        assert_eq!(mode(&ppu), 2, "each visible line starts in OAM scan");
    }

    #[test]
    fn vblank_starts_at_line_144() {
        let mut ppu = ppu_with(0);
        run(&mut ppu, CYCLES_PER_LINE * 144 + 4);
        assert_eq!(ppu.read(LY), 144);
        assert_eq!(mode(&ppu), 1);
    }

    #[test]
    fn vblank_interrupt_requested_when_entering_line_144() {
        let mut ppu = ppu_with(0);
        let before = run(&mut ppu, CYCLES_PER_LINE * 144 - 4);
        assert_eq!(before & INT_VBLANK, 0, "too early");
        let at = run(&mut ppu, 8);
        assert_ne!(at & INT_VBLANK, 0, "VBlank interrupt not requested");
    }

    #[test]
    fn vblank_interrupt_once_per_frame() {
        let mut ppu = ppu_with(0);
        let mut count = 0;
        for _ in 0..(CYCLES_PER_FRAME * 3) / 4 {
            if ppu.tick(4) & INT_VBLANK != 0 {
                count += 1;
            }
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn ly_wraps_to_0_after_line_153() {
        let mut ppu = ppu_with(0);
        run(&mut ppu, CYCLES_PER_LINE * 153 + 4);
        assert_eq!(ppu.read(LY), 153);
        run(&mut ppu, CYCLES_PER_LINE);
        assert_eq!(ppu.read(LY), 0);
        assert_eq!(mode(&ppu), 2);
    }
}

mod stat {
    use super::*;

    #[test]
    fn coincidence_flag_set_when_ly_equals_lyc() {
        let mut ppu = ppu_with(0);
        ppu.write(LYC, 1);
        run(&mut ppu, 4);
        assert_eq!(ppu.read(STAT) & 0b100, 0, "LY=0, LYC=1");
        run(&mut ppu, CYCLES_PER_LINE);
        assert_ne!(ppu.read(STAT) & 0b100, 0, "LY=1, LYC=1");
        run(&mut ppu, CYCLES_PER_LINE);
        assert_eq!(ppu.read(STAT) & 0b100, 0, "LY=2, LYC=1");
    }

    #[test]
    fn writing_stat_does_not_change_mode_or_coincidence_bits() {
        let mut ppu = ppu_with(0);
        ppu.write(LYC, 5);
        run(&mut ppu, 4); // mode 2, LY != LYC
        ppu.write(STAT, 0xFF);
        assert_eq!(ppu.read(STAT) & 0b111, 0b010);
        assert_eq!(
            ppu.read(STAT) & 0b0111_1000,
            0b0111_1000,
            "enable bits are writable"
        );
    }

    #[test]
    fn lyc_interrupt_when_enabled() {
        let mut ppu = ppu_with(0);
        ppu.write(LYC, 10);
        ppu.write(STAT, 1 << 6);
        let before = run(&mut ppu, CYCLES_PER_LINE * 10 - 4);
        assert_eq!(before & INT_STAT, 0, "too early");
        let at = run(&mut ppu, 8);
        assert_ne!(at & INT_STAT, 0, "LY=LYC STAT interrupt not requested");
    }

    #[test]
    fn no_stat_interrupt_when_nothing_enabled() {
        let mut ppu = ppu_with(0);
        ppu.write(STAT, 0);
        assert_eq!(run(&mut ppu, CYCLES_PER_FRAME) & INT_STAT, 0);
    }

    #[test]
    fn hblank_interrupt_when_enabled() {
        let mut ppu = ppu_with(0);
        ppu.write(STAT, 1 << 3);
        assert_eq!(run(&mut ppu, 80 + 168) & INT_STAT, 0, "still drawing");
        assert_ne!(run(&mut ppu, 8) & INT_STAT, 0, "entered HBlank");
    }

    #[test]
    fn vblank_stat_interrupt_when_enabled() {
        let mut ppu = ppu_with(0);
        ppu.write(STAT, 1 << 4);
        run(&mut ppu, CYCLES_PER_LINE * 144 - 4);
        assert_ne!(run(&mut ppu, 8) & INT_STAT, 0);
    }
}

mod background {
    use super::*;

    #[test]
    fn blank_vram_renders_white() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        run(&mut ppu, CYCLES_PER_FRAME);
        assert!(ppu.framebuffer().iter().all(|&p| p == 0));
    }

    #[test]
    fn first_tile_in_map_draws_top_left() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        write_solid_tile(&mut ppu, 0x8010, 3); // tile 1
        ppu.write(0x9800, 1); // map (0, 0) -> tile 1
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 3);
        assert_eq!(pixel(&ppu, 7, 7), 3);
        assert_eq!(pixel(&ppu, 8, 0), 0, "next tile is tile 0 (blank)");
        assert_eq!(pixel(&ppu, 0, 8), 0, "next row is tile 0 (blank)");
    }

    #[test]
    fn map_is_32_tiles_wide() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        write_solid_tile(&mut ppu, 0x8010, 2);
        ppu.write(0x9800 + 32 + 3, 1); // map row 1, column 3
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 3 * 8, 8), 2);
        assert_eq!(pixel(&ppu, 3 * 8 + 7, 15), 2);
        assert_eq!(pixel(&ppu, 3 * 8, 0), 0);
    }

    #[test]
    fn tile_pixels_use_both_bitplanes() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        // row 0 of tile 1: colours [0, 1, 2, 3, 0, 1, 2, 3]
        ppu.write(0x8010, 0b0101_0101);
        ppu.write(0x8011, 0b0011_0011);
        ppu.write(0x9800, 1);
        run(&mut ppu, CYCLES_PER_FRAME);

        let row: Vec<u8> = (0..8).map(|x| pixel(&ppu, x, 0)).collect();
        assert_eq!(row, [0, 1, 2, 3, 0, 1, 2, 3]);
    }

    #[test]
    fn bgp_palette_maps_colours() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        ppu.write(BGP, 0b00_01_10_11); // inverted
        write_solid_tile(&mut ppu, 0x8010, 1);
        ppu.write(0x9800, 1);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 2, "colour 1 -> shade 2");
        assert_eq!(pixel(&ppu, 8, 0), 3, "colour 0 -> shade 3");
    }

    #[test]
    fn scx_scrolls_horizontally() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        write_left_stripe_tile(&mut ppu, 0x8010);
        ppu.write(0x9801, 1); // map column 1 -> stripe at bg x = 8
        ppu.write(SCX, 3);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 5, 0), 3, "bg x 8 is now at screen x 5");
        assert_eq!(pixel(&ppu, 8, 0), 0);
    }

    #[test]
    fn scy_scrolls_vertically() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        write_solid_tile(&mut ppu, 0x8010, 3);
        ppu.write(0x9800 + 32, 1); // map row 1 -> bg y 8..16
        ppu.write(SCY, 4);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 3), 0);
        assert_eq!(pixel(&ppu, 0, 4), 3, "bg y 8 is now at screen y 4");
        assert_eq!(pixel(&ppu, 0, 11), 3);
        assert_eq!(pixel(&ppu, 0, 12), 0);
    }

    #[test]
    fn scroll_wraps_around_the_256px_map() {
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        write_solid_tile(&mut ppu, 0x8010, 3);
        ppu.write(0x9800, 1); // map (0, 0)
        ppu.write(SCX, 252);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 3, 0), 0, "bg x 255");
        assert_eq!(pixel(&ppu, 4, 0), 3, "bg x 0 wrapped round");
    }

    #[test]
    fn lcdc_bit3_selects_map_at_9c00() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | BG_MAP_9C00);
        write_solid_tile(&mut ppu, 0x8010, 3);
        ppu.write(0x9800, 1); // should be ignored
        ppu.write(0x9C01, 1);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 0);
        assert_eq!(pixel(&ppu, 8, 0), 3);
    }

    #[test]
    fn signed_tile_addressing_when_lcdc_bit4_clear() {
        // With LCDC.4 = 0, tile index is an i8 relative to 0x9000.
        let mut ppu = ppu_with(BG_ON);
        write_solid_tile(&mut ppu, 0x9000, 1); // index 0
        write_solid_tile(&mut ppu, 0x8800, 2); // index 0x80 (-128)
        write_solid_tile(&mut ppu, 0x8FF0, 3); // index 0xFF (-1)
        ppu.write(0x9800, 0x00);
        ppu.write(0x9801, 0x80);
        ppu.write(0x9802, 0xFF);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 1);
        assert_eq!(pixel(&ppu, 8, 0), 2);
        assert_eq!(pixel(&ppu, 16, 0), 3);
    }

    #[test]
    fn bg_disabled_renders_white() {
        let mut ppu = ppu_with(TILES_8000); // BG_ON clear
        write_solid_tile(&mut ppu, 0x8000, 3); // tile 0 everywhere
        run(&mut ppu, CYCLES_PER_FRAME);
        assert!(ppu.framebuffer().iter().all(|&p| p == 0));
    }

    #[test]
    fn scx_change_mid_frame_affects_later_lines_only() {
        // Rendering must happen per scanline, not once per frame.
        let mut ppu = ppu_with(BG_ON | TILES_8000);
        write_left_stripe_tile(&mut ppu, 0x8000); // tile 0 everywhere: stripe every 8px
        run(&mut ppu, CYCLES_PER_LINE * 72);
        ppu.write(SCX, 1);
        run(&mut ppu, CYCLES_PER_FRAME - CYCLES_PER_LINE * 72);

        assert_eq!(pixel(&ppu, 0, 0), 3, "top half unscrolled");
        assert_eq!(pixel(&ppu, 0, 100), 0, "bottom half scrolled by 1");
        assert_eq!(pixel(&ppu, 7, 100), 3);
    }
}

mod window {
    use super::*;

    #[test]
    fn window_at_wx7_wy0_covers_the_screen() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | WIN_ON | WIN_MAP_9C00);
        write_solid_tile(&mut ppu, 0x8010, 2);
        ppu.write(0x9C00, 1); // window map (0, 0)
        ppu.write(WX, 7);
        ppu.write(WY, 0);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 2);
        assert_eq!(pixel(&ppu, 8, 0), 0);
    }

    #[test]
    fn window_is_offset_by_wx_minus_7_and_wy() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | WIN_ON | WIN_MAP_9C00);
        write_solid_tile(&mut ppu, 0x8010, 2);
        ppu.write(0x9C00, 1);
        ppu.write(WX, 7 + 40);
        ppu.write(WY, 30);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 39, 30), 0, "left of window");
        assert_eq!(pixel(&ppu, 40, 29), 0, "above window");
        assert_eq!(pixel(&ppu, 40, 30), 2, "window origin");
        assert_eq!(pixel(&ppu, 47, 37), 2);
    }

    #[test]
    fn window_ignores_scroll() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | WIN_ON | WIN_MAP_9C00);
        write_solid_tile(&mut ppu, 0x8010, 2);
        ppu.write(0x9C00, 1);
        ppu.write(WX, 7);
        ppu.write(WY, 0);
        ppu.write(SCX, 50);
        ppu.write(SCY, 50);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 2);
    }

    #[test]
    fn window_disabled_shows_background() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | WIN_MAP_9C00); // WIN_ON clear
        write_solid_tile(&mut ppu, 0x8010, 2);
        ppu.write(0x9C00, 1);
        ppu.write(WX, 7);
        ppu.write(WY, 0);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 0);
    }
}

mod sprites {
    use super::*;

    // OAM coordinates are offset: screen (0, 0) is sprite (x = 8, y = 16).

    #[test]
    fn sprite_draws_at_offset_position() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        write_solid_tile(&mut ppu, 0x8010, 3);
        write_sprite(&mut ppu, 0, 16 + 20, 8 + 10, 1, 0);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 9, 20), 0);
        assert_eq!(pixel(&ppu, 10, 20), 3);
        assert_eq!(pixel(&ppu, 17, 27), 3);
        assert_eq!(pixel(&ppu, 18, 27), 0);
    }

    #[test]
    fn sprites_disabled_are_not_drawn() {
        let mut ppu = ppu_with(BG_ON | TILES_8000); // OBJ_ON clear
        write_solid_tile(&mut ppu, 0x8010, 3);
        write_sprite(&mut ppu, 0, 16, 8, 1, 0);
        run(&mut ppu, CYCLES_PER_FRAME);
        assert_eq!(pixel(&ppu, 0, 0), 0);
    }

    #[test]
    fn sprite_colour_0_is_transparent() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        write_solid_tile(&mut ppu, 0x8000, 1); // bg is all colour 1
                                               // tile 2 left as all zeroes
        write_sprite(&mut ppu, 0, 16, 8, 2, 0);
        run(&mut ppu, CYCLES_PER_FRAME);
        assert_eq!(pixel(&ppu, 0, 0), 1);
    }

    #[test]
    fn sprite_uses_obp0_or_obp1() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        ppu.write(OBP0, 0b00_00_01_00); // colour 1 -> shade 1
        ppu.write(OBP1, 0b00_00_10_00); // colour 1 -> shade 2
        write_solid_tile(&mut ppu, 0x8010, 1);
        write_sprite(&mut ppu, 0, 16, 8, 1, 0);
        write_sprite(&mut ppu, 1, 16, 8 + 16, 1, 1 << 4);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 1);
        assert_eq!(pixel(&ppu, 16, 0), 2);
    }

    #[test]
    fn sprite_x_flip() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        write_left_stripe_tile(&mut ppu, 0x8010);
        write_sprite(&mut ppu, 0, 16, 8, 1, 1 << 5);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 0);
        assert_eq!(pixel(&ppu, 7, 0), 3);
    }

    #[test]
    fn sprite_y_flip() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        ppu.write(0x8010, 0xFF); // only row 0 of tile 1 set
        ppu.write(0x8011, 0xFF);
        write_sprite(&mut ppu, 0, 16, 8, 1, 1 << 6);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 0);
        assert_eq!(pixel(&ppu, 0, 7), 3);
    }

    #[test]
    fn sprite_behind_bg_only_shows_over_bg_colour_0() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        write_solid_tile(&mut ppu, 0x8010, 1);
        write_solid_tile(&mut ppu, 0x8020, 3);
        ppu.write(0x9800, 1); // bg tile (0,0) is colour 1, the rest colour 0
        write_sprite(&mut ppu, 0, 16, 8 + 4, 2, 1 << 7); // straddles both
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 4, 0), 1, "bg colour 1 wins");
        assert_eq!(pixel(&ppu, 8, 0), 3, "bg colour 0 -> sprite shows");
    }

    #[test]
    fn tall_sprites_use_two_tiles_and_ignore_bit_0() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON | OBJ_8X16);
        write_solid_tile(&mut ppu, 0x8020, 1); // tile 2 (top)
        write_solid_tile(&mut ppu, 0x8030, 2); // tile 3 (bottom)
        write_sprite(&mut ppu, 0, 16, 8, 3, 0); // index 3 -> uses 2 and 3
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 0, 0), 1);
        assert_eq!(pixel(&ppu, 0, 8), 2);
        assert_eq!(pixel(&ppu, 0, 15), 2);
        assert_eq!(pixel(&ppu, 0, 16), 0);
    }

    #[test]
    fn at_most_10_sprites_per_line() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        write_solid_tile(&mut ppu, 0x8010, 3);
        for i in 0..11 {
            write_sprite(&mut ppu, i, 16, 8 + (i as u8) * 8, 1, 0);
        }
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 9 * 8, 0), 3, "10th sprite drawn");
        assert_eq!(pixel(&ppu, 10 * 8, 0), 0, "11th sprite dropped");
    }

    #[test]
    fn lower_x_wins_when_sprites_overlap() {
        let mut ppu = ppu_with(BG_ON | TILES_8000 | OBJ_ON);
        write_solid_tile(&mut ppu, 0x8010, 1);
        write_solid_tile(&mut ppu, 0x8020, 2);
        write_sprite(&mut ppu, 0, 16, 8 + 4, 1, 0); // further right, earlier in OAM
        write_sprite(&mut ppu, 1, 16, 8, 2, 0);
        run(&mut ppu, CYCLES_PER_FRAME);

        assert_eq!(pixel(&ppu, 5, 0), 2, "sprite with smaller X is on top");
    }
}
