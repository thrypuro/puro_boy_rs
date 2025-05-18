#![crate_name = "puro_boy"]
use rand;
use rand::Rng;
use std::fs;
use std::io;
mod gb;
use sdl2::event::{self, Event};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

const width: u32 = 600;
const height: u32 = 800;

const random_colors: [sdl2::pixels::Color; 2] =
    [sdl2::pixels::Color::BLUE, sdl2::pixels::Color::BLACK];

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
    // todo!();
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
    // canvas.set_draw_color(Color::RGB(0, 0, 0));

    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context
        .event_pump()
        .expect("Couldnt intialise event pump");
    let mut i: u8 = 0;
    'running: loop {
        i = i.wrapping_add(1);
        // canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        // canvas.clear();
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
                    // let color = sdl2::pixels::Color::RGB(x as u8, y as u8, 255);
                    let color = random_colors[rand::thread_rng().random_range(0..2)];
                    canvas.set_draw_color(color);

                    let p1 = sdl2::rect::Point::new(lastx, lasty);
                    let p2 = sdl2::rect::Point::new(x, y);
                    let _ = canvas.draw_line(p1, p2);
                    lastx = x;
                    lasty = y;
                    println!("mouse btn down at ({},{})", x, y);
                    canvas.present();
                }
                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.clear();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
pub fn main() {
    // create_window();
    run_cpu();
}
