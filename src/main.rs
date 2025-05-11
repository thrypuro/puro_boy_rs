#![crate_name = "puro_boy"]
use std::env;
use std::fs;
mod gb;

fn main() {

    let args : Vec<String>  = env::args().collect();

    let arg1 : &String = &args[1];
    // let arg2 : &String = &args[2];

    let rom = fs::read(arg1).unwrap();
    
    let mut cpu = gb::cpu::CPU::new(rom);
    for i in 1..100
    {
        cpu.step();
    }
    
    // println!("A: {}", c);
}
 
