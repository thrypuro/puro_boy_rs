#![crate_name = "puro_boy"]
use rand;
use sdl2::render::{Canvas, Texture, WindowCanvas};
use sdl2::sys::Window;
use std::fs;
use std::io;
mod gb;
use sdl2::event::{self, Event};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::Duration;

const width: u32 = 128;
const height: u32 = 128;

fn run_cpu() {
    println!("Enter the path to the ROM file:");
    let mut rom_path = String::new();
    io::stdin()
        .read_line(&mut rom_path)
        .expect("Failed to read input");
    let rom_path = rom_path.trim(); // Remove any trailing newline or whitespace

    // Attempt to load the ROM file
    let rom = match fs::read(rom_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to read ROM file: {}", err);
            return;
        }
    };

    let mut cpu = gb::cpu::CPU::new(rom);
    loop {
        cpu.step();

        cpu.print_registers();
    }
}

fn create_window() {
    //

    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Puro boy", width, height)
        .position_centered()
        .build()
        .expect("Couldn't build window");
    let mut canvas = window.into_canvas().build().expect("Couldnt build canvas");
    let mut lastx = 0;
    let mut lasty = 0;
    let a = [
        0x7C, 0x7C, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0xFE, 0xC6, 0xC6, 0x00, 0xC6, 0xC6, 0x00, 0x00,
        0x00,
    ];

    let b = gb::ppu::get_tile(a);

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Couldnt intialise event pump");

    let mut i: u8 = 0;
    let org = Point::new(0, 0);
    'running: loop {
        i = i.wrapping_add(1);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::MouseButtonDown { x, y, .. } => {
                    // (mouse event code omitted)
                }
                _ => {}
            }
        }

        gb::ppu::render_tile(b, &mut canvas);

        // Clear the canvas once per frame
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
pub fn main() {
    create_window();
    // run_cpu();
}
