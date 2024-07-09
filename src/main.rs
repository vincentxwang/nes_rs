use nes_rs::{bus::Bus, cartridge::Cartridge, cpu::{trace, CPU}};
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    let bytes: Vec<u8> = std::fs::read("tests/nestest.nes").unwrap();
    let rom = Cartridge::new(&bytes).unwrap();

    let bus = Bus::new(rom);
    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.program_counter = 0xC000;

    cpu.run_with_callback(move |cpu| {
        println!("{}", trace::trace(cpu));
    });
}
