use crate::cartridge::Cartridge;
use crate::cpu::Mem;
use crate::ppu::PPU;

/// NES Bus
///
/// <http://wiki.nesdev.com/w/index.php/CPU_memory_map>
///
/// |-----------------| $FFFF |-----------------|
/// | PRG-ROM         |       |                 |
/// |-----------------| $8000 |-----------------|
/// | PRG-RAM or SRAM |       | PRG-RAM or SRAM |
/// |-----------------| $6000 |-----------------|
/// | Expansion       |       | Expansion       |
/// | Modules         |       | Modules         |
/// |-----------------| $4020 |-----------------|
/// | APU/Input       |       |                 |
/// | Registers       |       |                 |
/// |- - - - - - - - -| $4000 |                 |
/// | PPU Mirrors     |       | I/O Registers   |
/// | $2000-$2007     |       |                 |
/// |- - - - - - - - -| $2008 |                 |
/// | PPU Registers   |       |                 |
/// |-----------------| $2000 |-----------------|
/// | WRAM Mirrors    |       |                 |
/// | $0000-$07FF     |       |                 |
/// |- - - - - - - - -| $0800 |                 |
/// | WRAM            |       | 2K Internal     |
/// |- - - - - - - - -| $0200 | Work RAM        |
/// | Stack           |       |                 |
/// |- - - - - - - - -| $0100 |                 |
/// | Zero Page       |       |                 |
/// |-----------------| $0000 |-----------------|

// Memmory map constants. Includes mirrors.
const WRAM_START: u16 = 0x0000;
const WRAM_END: u16 = 0x1FFF;
const PPU_START: u16 = 0x2000;
const PPU_MIRRORS_START: u16 = 0x2008;
const PPU_MIRRORS_END: u16 = 0x3FFF;
const PRG_ROM_START: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub struct Bus {
    cpu_wram: [u8; WRAM_SIZE],
    prg_rom: Vec<u8>,
    ppu: PPU,
}

const WRAM_SIZE: usize = 0x0800; // 2K Work

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            cpu_wram: [0; WRAM_SIZE],
            prg_rom: cartridge.prg_rom,
            ppu: PPU::new(cartridge.chr_rom, cartridge.screen_mirroring),
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= PRG_ROM_START;
        // Mirror in case PRG ROM takes up only 16kB instead of 32kB.
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.prg_rom[addr as usize]
    }
}

impl Mem for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            // WRAP start (0x0000 -> 0x1fff)
            WRAM_START..=WRAM_END => {
                // Take the last 11 bits.
                let mirror_down_addr = addr & 0b111_1111_1111;
                self.cpu_wram[mirror_down_addr as usize]
            }

            // PPU start (0x2000 -> 0x3fff)
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempting to read from write-only PPU address {:x}", addr); 
            }
            0x2007 => self.ppu.read_data(),

            PPU_MIRRORS_START..=PPU_MIRRORS_END => {
                // Mirrors $2008 - $4000 into $2000 - $2008
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            },

            PRG_ROM_START..=PRG_ROM_END => self.read_prg_rom(addr),

            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            WRAM_START..=WRAM_END => {
                // Only accept 11 bits instead of 13 for RAM
                let mirror_down_addr = addr & 0b111_1111_1111;
                self.cpu_wram[mirror_down_addr as usize] = data;
            }
            
            0x2000 => self.ppu.write_to_controller(data),

            0x2006 => self.ppu.write_to_ppu_addr(data),
            
            0x2007 => self.ppu.write_to_data(data),

            PPU_MIRRORS_START..=PPU_MIRRORS_END => {
                // Mirrors PPU mirrors ($2008 - $4000) into $2000 - $2008
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            PRG_ROM_START..=PRG_ROM_END => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
