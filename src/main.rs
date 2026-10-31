mod cpu;
mod error;
mod mmu;

use cpu::CPU;
use mmu::{Button, MMU};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{ScaleMode, Texture, WindowCanvas};
use sdl3::sys::pixels::SDL_PixelFormat;
use sdl3::sys::render::SDL_RendererLogicalPresentation;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::{env, fs, io};

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const DEFAULT_SCALE: u32 = 4;

/// T-cycles in one frame (154 lines x 456 cycles), and how long a frame lasts on a
/// real DMG (4194304 Hz / 70224 = ~59.73 frames per second).
const CYCLES_PER_FRAME: u32 = 70_224;
const FRAME_TIME: Duration = Duration::from_nanos(16_742_706);

/// Shades 0 (lightest) to 3 (darkest) as RGB. P cycles through them.
const PALETTES: [[[u8; 3]; 4]; 2] = [
    [
        [0xFF, 0xFF, 0xFF],
        [0xC0, 0xC0, 0xC0],
        [0x60, 0x60, 0x60],
        [0x00, 0x00, 0x00],
    ],
    [
        [0xE0, 0xF8, 0xD0],
        [0x88, 0xC0, 0x70],
        [0x34, 0x68, 0x56],
        [0x08, 0x18, 0x20],
    ],
];

/// Keyboard layout: arrows = D-pad, X = A, Z = B, Enter = Start, Backspace/RShift = Select.
fn button_for(key: Keycode) -> Option<Button> {
    Some(match key {
        Keycode::Right => Button::Right,
        Keycode::Left => Button::Left,
        Keycode::Up => Button::Up,
        Keycode::Down => Button::Down,
        Keycode::X => Button::A,
        Keycode::Z => Button::B,
        Keycode::Return => Button::Start,
        Keycode::Backspace | Keycode::RShift => Button::Select,
        _ => return None,
    })
}

/// Runs the CPU until the PPU enters VBlank (one frame). The cap keeps time moving
/// while the LCD is off and no VBlank ever comes.
fn run_frame(cpu: &mut CPU) {
    let mut cycles = 0u32;
    while cycles < CYCLES_PER_FRAME + 456 {
        let step = cpu.step() as u32;
        cycles += step;
        if cpu.memory.update_ppu(step) {
            break;
        }
    }
}

/// Uploads the PPU framebuffer to `texture` and presents it, scaled to the window.
fn draw(
    canvas: &mut WindowCanvas,
    texture: &mut Texture,
    framebuffer: &[u8],
    palette: &[[u8; 3]; 4],
) {
    texture
        .with_lock(None, |pixels, pitch| {
            for (y, row) in framebuffer.chunks_exact(SCREEN_WIDTH).enumerate() {
                for (x, &shade) in row.iter().enumerate() {
                    let offset = y * pitch + x * 3;
                    pixels[offset..offset + 3].copy_from_slice(&palette[(shade & 3) as usize]);
                }
            }
        })
        .expect("couldn't lock the screen texture");
    canvas.clear();
    canvas
        .copy(texture, None, None)
        .expect("couldn't draw the screen texture");
    canvas.present();
}

/// Battery-backed RAM lives next to the ROM as `<rom>.sav`.
fn save_path(rom_path: &Path) -> PathBuf {
    rom_path.with_extension("sav")
}

fn load_save(mmu: &mut MMU, path: &Path) {
    if mmu.cartridge().battery_ram().is_none() {
        return;
    }
    match fs::read(path) {
        Ok(data) if mmu.cartridge_mut().load_battery_ram(&data) => {
            println!("Loaded save {}", path.display())
        }
        Ok(_) => eprintln!("Ignoring {}: wrong size for this cartridge", path.display()),
        Err(_) => {} // no save yet
    }
}

fn write_save(mmu: &MMU, path: &Path) {
    if let Some(ram) = mmu.cartridge().battery_ram() {
        match fs::write(path, ram) {
            Ok(()) => println!("Saved {}", path.display()),
            Err(err) => eprintln!("Couldn't write {}: {err}", path.display()),
        }
    }
}

fn run_emu(rom: Vec<u8>, rom_path: &Path) {
    let mut mmu = MMU::new(rom);
    let title = match mmu.cartridge().title() {
        t if t.is_empty() => "Puro boy".to_string(),
        t => format!("Puro boy - {t}"),
    };
    log::info!("Cartridge: {title} ({:?})", mmu.cartridge().kind());
    let save = save_path(rom_path);
    load_save(&mut mmu, &save);

    let sdl_context = sdl3::init().expect("couldn't initialise SDL");
    let video = sdl_context.video().expect("couldn't initialise SDL video");
    let window = video
        .window(
            &title,
            SCREEN_WIDTH as u32 * DEFAULT_SCALE,
            SCREEN_HEIGHT as u32 * DEFAULT_SCALE,
        )
        .position_centered()
        .resizable()
        .build()
        .expect("couldn't create the window");
    let mut canvas = window.into_canvas();
    // Keep the 160x144 image at whole-number scales, letterboxed in black.
    canvas
        .set_logical_size(
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            SDL_RendererLogicalPresentation::INTEGER_SCALE,
        )
        .expect("couldn't set the logical size");
    canvas.set_draw_color(Color::BLACK);

    let texture_creator = canvas.texture_creator();
    let format = PixelFormat::try_from(SDL_PixelFormat::RGB24).expect("RGB24 pixel format");
    let mut texture = texture_creator
        .create_texture_streaming(format, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .expect("couldn't create the screen texture");
    texture.set_scale_mode(ScaleMode::Nearest);

    let mut event_pump = sdl_context.event_pump().expect("couldn't get SDL events");
    let mut cpu = CPU::new(&mut mmu);
    let mut palette = 0;
    let mut fast_forward = false;
    let mut next_frame = Instant::now();

    println!("Arrows: D-pad  X: A  Z: B  Enter: Start  Backspace: Select");
    println!("Tab (hold): fast forward  P: palette  Esc: quit");

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => match key {
                    Keycode::Tab => fast_forward = true,
                    Keycode::P => palette = (palette + 1) % PALETTES.len(),
                    _ => {
                        if let Some(button) = button_for(key) {
                            cpu.memory.set_button(button, true);
                        }
                    }
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Tab => fast_forward = false,
                    _ => {
                        if let Some(button) = button_for(key) {
                            cpu.memory.set_button(button, false);
                        }
                    }
                },
                _ => {}
            }
        }

        run_frame(&mut cpu);
        draw(
            &mut canvas,
            &mut texture,
            cpu.memory.ppu().framebuffer(),
            &PALETTES[palette],
        );

        // Pace to the real frame rate. If we fell far behind (e.g. the window was
        // dragged), resync instead of racing to catch up.
        next_frame += FRAME_TIME;
        let now = Instant::now();
        if fast_forward || next_frame + FRAME_TIME * 4 < now {
            next_frame = now;
        } else if next_frame > now {
            std::thread::sleep(next_frame - now);
        }
    }

    write_save(cpu.memory, &save);
}

/// The ROM path from the command line, or asked for on stdin.
fn rom_path() -> PathBuf {
    if let Some(path) = env::args_os().nth(1) {
        return PathBuf::from(path);
    }
    println!("Enter the path to the ROM file:");
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read input");
    PathBuf::from(line.trim())
}

pub fn main() {
    env_logger::init();
    let path = rom_path();

    let rom = match fs::read(&path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "{:?}: {}: {err}",
                error::EmuError::RomNotInitialised,
                path.display()
            );
            return;
        }
    };
    log::debug!("Loaded ROM: {} ({} bytes)", path.display(), rom.len());

    run_emu(rom, &path);
}
