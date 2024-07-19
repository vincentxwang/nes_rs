/// NES Bus
///
/// Reference: <http://wiki.nesdev.com/w/index.php/CPU_memory_map>

use crate::cartridge::Cartridge;
use crate::cpu::Mem;
use crate::joypad::Joypad;
use crate::ppu::PPU;
use crate::bus::dma::DMA;

mod dma;

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
pub const WRAM_START: u16 = 0x0000;
pub const WRAM_END: u16 = 0x1FFF;
pub const PPU_START: u16 = 0x2000;
pub const PPU_MIRRORS_START: u16 = 0x2008;
pub const PPU_MIRRORS_END: u16 = 0x3FFF;
pub const PRG_RAM_START: u16 = 0x6000;
pub const PRG_RAM_END: u16 = 0x7FFF;
pub const PRG_ROM_START: u16 = 0x8000;
pub const PRG_ROM_END: u16 = 0xFFFF;

pub struct Bus {
    pub cpu_wram: [u8; WRAM_SIZE],
    prg_ram: Vec<u8>,
    prg_rom: Vec<u8>,
    pub ppu: PPU,
    pub cycles: usize,

    pub joypad: Joypad,

    // dma: DMA,
}


// 2K Work RAM
const WRAM_SIZE: usize = 0x0800; 
const PRG_RAM_SIZE: usize = 0x2000;

impl Bus {
    pub fn new(cartridge: Cartridge) -> Bus {
        Bus {
            cpu_wram: [0; WRAM_SIZE],
            prg_ram: [0; PRG_RAM_SIZE].to_vec(),
            prg_rom: cartridge.prg_rom,
            ppu: PPU::new(cartridge.chr_rom, cartridge.screen_mirroring),
            cycles: 7,
            joypad: Joypad::new(),

            // dma: DMA::new(),
        }
    }

    // With CHR-ROM, but with empty callback function.
    pub fn default(rom: Cartridge) -> Self {
        Bus::new(rom)
    }

    pub fn tick(&mut self, cycles: usize) {
        self.ppu.tick(cycles * 3);

        // TODO: implement DMA. for now we just naively write with OAM data

        // if self.dma.dma_transfer {
        //     // If not synced, wait a cycle
        //     if self.dma.dma_is_not_sync {
        //         if self.cycles % 2 == 1 {
        //             self.dma.dma_is_not_sync = false;
        //         }
        //     } else {
        //         // On even clock cycles, read from CPU
        //         if self.cycles % 2 == 0 {
        //             self.dma.data = self.mem_read((self.dma.page as u16) << 8 | self.dma.addr as u16)
        //         // On odd clock cycles, write to OAM
        //         } else {
        //             self.ppu.oam_data[self.dma.addr as usize] = self.dma.data;
        //             self.dma.addr = self.dma.addr.wrapping_add(1);

        //             // If dma.addr wraps around back to 0x00, we are done
        //             if self.dma.addr == 0x00 {
        //                 self.dma.dma_transfer = false;
        //                 self.dma.dma_is_not_sync = true;
        //             }
        //         }
        //     }
        // } else {
        //     self.cycles += cycles;
        // }
   }

    pub fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= PRG_ROM_START;
        // Mirror in case PRG ROM takes up only 16kB instead of 32kB.
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn read_prg_ram(&self, mut addr: u16) -> u8 {
        addr -= PRG_RAM_START;
        self.prg_ram[addr as usize]
    }

    fn write_to_prg_ram(&mut self, mut addr: u16, val: u8) {
        addr -= PRG_RAM_START;
        self.prg_ram[addr as usize] = val;
    }

    pub fn pull_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
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

            0x2002 => self.ppu.read_status(),

            0x2004 => self.ppu.read_oam_data(),

            0x2007 => self.ppu.read_data(),

            0x4016 => self.joypad.read(),

            PPU_MIRRORS_START..=PPU_MIRRORS_END => {
                // Mirrors $2008 - $4000 into $2000 - $2008
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            },

            PRG_RAM_START..=PRG_RAM_END => self.read_prg_ram(addr),

            PRG_ROM_START..=PRG_ROM_END => self.read_prg_rom(addr),

            _ => {
                println!("Ignoring mem_read at BUS address {}", addr);
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

            0x2001 => self.ppu.write_to_mask(data),

            0x2002 => panic!("Attempt to write to PPU status register"),

            0x2003 => self.ppu.write_to_oam_addr(data),

            0x2004 => self.ppu.write_to_oam_data(data),

            0x2005 => self.ppu.write_to_scroll(data),

            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
                // println!("mem_write to 0x2006 with {}", data);
            }
            
            0x2007 => {
                self.ppu.write_to_data(data);
                // println!("mem_write to 0x2007 with {}", data);
            }
            
            // Lazy DMA. TODO: handle cycle accuracy with this.
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }

                self.ppu.write_oam_dma(&buffer);
            }

            0x4016 => self.joypad.write(data),

            PPU_MIRRORS_START..=PPU_MIRRORS_END => {
                // Mirrors PPU mirrors ($2008 - $4000) into $2000 - $2008
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }

            PRG_RAM_START..=PRG_RAM_END => self.write_to_prg_ram(addr, data),

            PRG_ROM_START..=PRG_ROM_END => {
                println!("Ignoring: Write {} to PRG-ROM space at BUS address {}", data, addr);
            }
            
            _ => {
                println!("Ignoring attempt to write {} to BUS address {}", data, addr);
            }
        }
    }
}
