use gb::mmu::MMU;
use gb::ppu::PPU;
use std::fs;
use std::io;
mod gb;
use gb::cpu::CPU;
use sdl3::event::{self, Event};
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::time::Duration;

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

fn create_window(cpu: &mut CPU) {
    //
    let sdl_context = sdl3::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Puro boy", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .expect("Couldn't build window");
    let mut canvas = window.into_canvas();

    let mut event_pump = sdl_context
        .event_pump()
        .expect("Couldnt intialise event pump");

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

                _ => {}
            }
        }
        // Clear the canvas once per frame
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

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
    // let mut mmu = MMU::new(rom);
    // let mut cpu = CPU::new(&mut mmu);

    // create_window(&mut cpu);
}
