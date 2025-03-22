#![crate_name = "puro_boy"]

mod gb;

fn main() {
    let c = gb::cpu::cpu::new();
    println!("Hello world!");
    println!("A: {}", c.clock);
}
 
