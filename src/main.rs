mod cpu;
mod mmu;

use cpu::CPU;
use env_logger;
use mmu::MMU;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::render::WindowCanvas;
use sdl3::EventPump;
use std::fs;
use std::io;
use std::time::Duration;

fn create_window(width: u32, height: u32) -> (WindowCanvas, EventPump) {
    let sdl_context = sdl3::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Puro boy", width, height)
        .position_centered()
        .build()
        .expect("Couldn't build window");
    let canvas = window.into_canvas();

    let event_pump = sdl_context
        .event_pump()
        .expect("Couldn't initialize event pump");

    (canvas, event_pump)
}

fn run_emu(rom: Vec<u8>) {
    // Create a window with Game Boy resolution (160x144)
    let (mut canvas, mut event_pump) = create_window(160, 144);

    // Initialize MMU with ROM
    let mut mmu = MMU::new(rom);

    // Initialize CPU
    let mut cpu = CPU::new(&mut mmu);

    // Enable the LCD
    // mmu.get_ppu_mut().turn_lcd_on();
    cpu.memory.ppu.turn_lcd_on();

    // Main emulation loop
    'running: loop {
        // Handle events
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

        // Execute a number of CPU cycles
        // A single frame is ~70,000 cycles at 4MHz
        let cycles_per_frame = 70000;
        let mut cycles_this_frame = 0;

        while cycles_this_frame < cycles_per_frame {
            // Execute one CPU instruction
            cpu.step();

            // Add the cycles for this instruction (for simplicity, using 4 cycles)
            cycles_this_frame += 4;

            // Update the PPU
            if cpu.memory.update_ppu(4) {
                // If a frame is ready, render it
                cpu.memory.get_ppu_mut().render(&mut canvas);
            }
        }

        canvas.clear();

        // Limit to ~60 FPS
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        // Print CPU registers for debugging
        cpu.print_registers();
    }
}

pub fn main() {
    env_logger::init();
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

    log::debug!("Loaded ROM: {} ({} bytes)", rom_path, rom.len());

    // Run the emulator with the loaded ROM
    run_emu(rom);
}
