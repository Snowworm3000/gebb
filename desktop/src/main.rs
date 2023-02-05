use gb_core::cpu::Cpu;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut args: Vec<_> = env::args().collect();
    if args.len() != 2 { // TODO: Add this back for release build
        // println!("Usage: cargo run path/to/game");
        // return;
        args = vec![String::from(""), String::from("cpu_instrs.gb")];
    }

    let mut gb = Cpu::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    gb.load(&buffer);

    'gameloop: loop {
        gb.tick();
    }
}
