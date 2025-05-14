#![crate_name = "puro_boy"]
use std::env;
use std::fs;
use std::io;
mod gb;

fn main() {

    // let args : Vec<String>  = env::args().collect();
    //
    // let arg1 : &String = &args[1];
    // // let arg2 : &String = &args[2];
    println!("Enter the path to the ROM file:");

    // Read the ROM file path from the user
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
    for _ in 1..100
    {
        cpu.step();
    }
    
    // println!("A: {}", c);
}
 
