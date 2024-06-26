use nes_rs::{bus::Bus, cartridge::Cartridge, cpu::{trace, CPU}};

fn main() {
    let bytes: Vec<u8> = std::fs::read("src/nestest.nes").unwrap();
    let rom = Cartridge::new(&bytes).unwrap();

    let bus = Bus::new(rom);
    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.program_counter = 0xC000;

    cpu.run_with_callback(move |cpu| {
        println!("{}", trace(cpu));
    });
}
