#![crate_name = "puro_boy"]
use sdl2::render::{Canvas, Texture, WindowCanvas};

use std::fs;
use std::io;
mod gb;
use gb::cpu::CPU;
use sdl2::event::{self, Event};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::Duration;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

fn run_cpu(cpu: &mut CPU) {
    loop {
        cpu.step();

        cpu.print_registers();
    }
}

fn create_window(cpu: CPU) {
    //
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Puro boy", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .expect("Couldn't build window");
    let mut canvas = window.into_canvas().build().expect("Couldnt build canvas");

    let a = [
        0x7C, 0x7C, 0x00, 0xC6, 0xC6, 0x00, 0x00, 0xFE, 0xC6, 0xC6, 0x00, 0xC6, 0xC6, 0x00, 0x00,
        0x00,
    ];

    let b = gb::ppu::get_tile(&a);

    // let c = gb::ppu::loadtileset(cpu.memory.rom);

    // println!(" {:?}", c.tiles);

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Couldnt intialise event pump");

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    'running: loop {
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
        // Clear the canvas once per frame
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        let mut i: u8 = 0;
        let mut org = Point::new(0, 0);

        // for tile in &c.tiles {
        //     gb::ppu::render_tile(tile.data, &mut canvas, org);
        //     org += Point::new(8, 8);
        //     i += 1;

        //     // if i == 10 {
        //     //     break;
        //     // }
        // }

        // Clear the canvas once per frame
        // canvas.set_draw_color(Color::RGB(255, 255, 255));
        // canvas.clear();
        //
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
pub fn main() {
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
    let mut cpu = CPU::new(rom);
    let mut ppu = gb::ppu::loadtileset(&cpu.memory.rom);

    // create_window(cpu);
    // run_cpu();
}
