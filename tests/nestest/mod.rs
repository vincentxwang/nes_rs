//! This is a test based on Kevin Horton's NES CPU test here: https://www.qmtpro.com/~nes/misc/nestest.txt
//! nestestmaster.log is the expected output 
//! The last few lines seem to deal with the I/O register and have been removed. We've also moved cycles "down" one slot.
//! Cycle and PPU afe NOT implemented yet.

#[cfg(test)]
mod nestest {
    use std::fs;
    use std::io::BufRead;

    use nes_rs::cartridge::Cartridge;
    use nes_rs::bus::Bus;
    use nes_rs::cpu::{trace, CPU};

    #[test]
    fn nestest() {

        let bytes: Vec<u8> = std::fs::read("tests/nestest/nestest.nes").unwrap();
        let rom = Cartridge::new(&bytes).unwrap();
    
        let bus = Bus::new(rom);
        let mut cpu = CPU::new(bus);
        cpu.reset();
        cpu.program_counter = 0xC000;
    
        let master: String = fs::read_to_string("tests/nestest/nestestmaster.log").unwrap();
    
        let cursor = std::io::Cursor::new(master);
        let mut lines_iter = cursor.lines().map(|l| l.unwrap());
    
        cpu.run_with_callback(move |cpu| {
            let line = lines_iter.next();
            if line.is_none() {
                return
            } else {
                // get the string without ppu information
                let test = line.as_ref().unwrap()[..73].to_owned() + &line.unwrap()[85..];
                assert_eq!(test, trace::trace(cpu));
            }
        });
    }
}
