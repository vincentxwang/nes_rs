//! This is a test based on Kevin Horton's NES CPU test here: https://www.qmtpro.com/~nes/misc/nestest.txt
//! nestestmaster.log is the expected output 
//! The last few lines seem to deal with the I/O register and have been removed. We've also moved cycles "down" one slot.
//! Cycle and PPU afe NOT implemented yet.

#[cfg(test)]
mod blarggcpu {
    
    use nes_rs::cartridge::Cartridge;
    use nes_rs::bus::Bus;
    use nes_rs::cpu::{trace, Mem, CPU};

    #[test]
    fn main() {

        let bytes: Vec<u8> = std::fs::read("tests/blarggcpu/rom_singles/01-basics.nes").unwrap();
        let rom = Cartridge::new(&bytes).unwrap();
        
        let bus = Bus::default(rom);

        let mut cpu = CPU::new(bus);
        cpu.reset();
        // cpu.program_counter = 0xE000;
    
        // let master: String = fs::read_to_string("tests/nestest/nestestmaster.log").unwrap();
    
        // let cursor = std::io::Cursor::new(master);
        // let mut lines_iter = cursor.lines().map(|l| l.unwrap());
    
        cpu.run_with_callback(move |cpu| {
            println!("{}", trace::trace(cpu));
            println!("{}", cpu.mem_read(0x6000));
        });
    }
}
