//! An implementation of the NES picture processing unit.
//! Reference: https://www.nesdev.org/wiki/PPU
//! https://www.nesdev.org/wiki/PPU_memory_map

use crate::cartridge::Mirroring;
use registers::controller::PPUCTRL;
use registers::mask::PPUMASK;
use registers::addr::PPUADDR;
use registers::scroll::PPUSCROLL;
use registers::status::PPUSTATUS;

pub mod registers;

// Memory map constants.
const CHR_ROM_START: u16 = 0x0000;
const CHR_ROM_END: u16 = 0x1fff;
const VRAM_START: u16 = 0x2000;
const ATTRIBUTE_TABLE_START: u16 = 0x23c0;
const VRAM_END: u16 = 0x2fff;
const UNUSED_START: u16 = 0x3000;
const UNUSED_END: u16 = 0x3eff;
const PALETTE_TABLE_START: u16 = 0x3f00;
const PALETTE_TABLE_END: u16 = 0x3fff;

const NAMETABLE_SIZE: u16 = 0x0400;

// Storage size constants.
const PALETTE_TABLE_SIZE: usize = 32;
const VRAM_SIZE: usize = 2048;
const OAM_DATA_SIZE: usize = 256;

pub struct PPU {
    // $0000 - $1FFF is usually mapped to the CHR-ROM
    pub chr_rom: Vec<u8>,
    // $2000 - $2FFF is usually mapped to an internal vRAM
    pub vram: [u8; VRAM_SIZE],
    pub palette_table: [u8; PALETTE_TABLE_SIZE],
    // Divide by 4 because each OAMByte represents 4 bytes.
    pub oam_data: [u8; OAM_DATA_SIZE],
 
    pub controller: PPUCTRL,
    pub ppu_addr: PPUADDR,
    pub mirroring: Mirroring,
    pub ppu_mask: PPUMASK,
    pub oam_addr: u8,
    pub ppu_scroll: PPUSCROLL,
    pub status: PPUSTATUS,

    pub scanline: u16,
    pub cycles: usize,

    pub nmi_interrupt: Option<u8>,

    pub chr_ram: Option<Vec<u8>>,

    // For PPUDATA
    internal_data_buffer: u8,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {

        let chr_ram;

        if chr_rom.len() == 0 {
            chr_ram = Some(vec![0; 0x2000]);
        } else {
            chr_ram = None;
        }

        PPU {
            chr_rom,
            mirroring,
            controller: PPUCTRL::new(),
            palette_table: [0; PALETTE_TABLE_SIZE],
            vram: [0; VRAM_SIZE],
            oam_data: [0; OAM_DATA_SIZE],
            ppu_addr: PPUADDR::new(),
            ppu_mask: PPUMASK::new(),
            ppu_scroll: PPUSCROLL::new(),
            status: PPUSTATUS::new(),
            oam_addr: 0,

            scanline: 0,
            cycles: 21,

            // Simplification of NMI_occurred and NMI_output
            nmi_interrupt: None,

            internal_data_buffer: 0,

            chr_ram,
        }
    }

    pub fn default() -> Self {
        PPU {
            chr_rom: vec![0; 1],
            mirroring: Mirroring::Horizontal,
            controller: PPUCTRL::new(),
            palette_table: [0; PALETTE_TABLE_SIZE],
            vram: [1; VRAM_SIZE],
            oam_data: [0; OAM_DATA_SIZE],
            ppu_addr: PPUADDR::new(),
            ppu_mask: PPUMASK::new(),
            ppu_scroll: PPUSCROLL::new(),
            status: PPUSTATUS::new(),
            oam_addr: 0,

            scanline: 0,
            cycles: 21,

            // Simplification of NMI_occurred and NMI_output
            nmi_interrupt: None,

            internal_data_buffer: 0,

            chr_ram: None,
        }
    }

    // Progresses PPU cycles and sets up NMI + VBLANK.
    pub fn tick(&mut self, ppu_cycles: usize) -> bool {
        self.cycles += ppu_cycles;

        if self.cycles >= 341 {
            self.cycles -= 341;
            self.scanline += 1;

            // VBLANK begins on 241
            if self.scanline == 241 {
                self.status.set(PPUSTATUS::VBLANK_STARTED, true);

                // println!("SCANLINE 241");

                if self.controller.contains(PPUCTRL::GENERATE_NMI)  {
                    self.nmi_interrupt = Some(1);
                    // println!("NMI_INTERRUPT SET");
                }
            };

            // VBLANK ends after 261 (cycle restarts)
            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.set(PPUSTATUS::SPRITE_ZERO_HIT, false);
                self.status.set(PPUSTATUS::VBLANK_STARTED, false);
                self.nmi_interrupt = None;
                return true;
            }
        };
        false
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.ppu_addr.update(value);
    }

    pub fn write_to_controller(&mut self, value: u8) {
        let before_nmi_status = self.controller.contains(PPUCTRL::GENERATE_NMI);
        self.controller = PPUCTRL::from_bits_truncate(value);
        // self.controller.set(PPUCTRL::GENERATE_NMI, true);
        if !before_nmi_status && self.controller.contains(PPUCTRL::GENERATE_NMI) && self.status.contains(PPUSTATUS::VBLANK_STARTED) {
            self.nmi_interrupt = Some(1);
        }
    }

    // Writes a value to PPUMASK ($2001).
    pub fn write_to_mask(&mut self, value: u8) {
        self.ppu_mask = PPUMASK::from_bits_truncate(value);
    }

    // Writes a value to PPUSCROLL ($2003).
    pub fn write_to_scroll(&mut self, value: u8) {
        self.ppu_scroll.write(value);
    }

    // Writing to OAMDATA ($2004).
    // This is notoriously finnicky. Check this later with PPU ROMs.
    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }
    
    // Replace OAM data.
    pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for x in data.iter() {
            self.oam_data[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }

    fn increment_vram_addr(&mut self) {
        if self.controller.contains(PPUCTRL::VRAM_ADD_INCREMENT) {
            self.ppu_addr.increment(32);
        } else {
            self.ppu_addr.increment(1);
        }
    }

    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.ppu_addr.get();

        self.increment_vram_addr();

        match addr {
            CHR_ROM_START..=CHR_ROM_END => {
                if let Some(chr_ram) = &mut self.chr_ram {
                    chr_ram[addr as usize] = value;
                } else {
                    println!("Ignoring write into PPU CHR-ROM space at addr {}", addr);
                }
            },

            VRAM_START..=VRAM_END => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
                // println!("writing {} to {}", value, self.mirror_vram_addr(addr))
            },
            UNUSED_START..=UNUSED_END => panic!("Attempting to write to unused space {}", addr),
            
            // $3f10, $3f14, $3f18, $3f1c are mirrors of $3f00, $3f04, $3f08, $3f0c respectively
            // Reference: https://www.nesdev.org/wiki/PPU_palettes
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                self.palette_table[(addr - 0x10 - PALETTE_TABLE_START) as usize] = value;
            }

            PALETTE_TABLE_START..=PALETTE_TABLE_END => {
                self.palette_table[(addr - PALETTE_TABLE_START) as usize] = value;
            }

            _ => panic!("Unexpected access to {}", addr),
        }
    }

    // Reference: https://www.nesdev.org/wiki/PPU_attribute_tables
    // In the diagram below, each byte-palette (big square) controls a 32x32 pixels (or 4x4 tiles).
    //        2xx0    2xx1    2xx2    2xx3    2xx4    2xx5    2xx6    2xx7
    //      ,-------+-------+-------+-------+-------+-------+-------+-------.
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    // 2xC0:| - 0 - | - 1 - | - 2 - | - 3 - | - 4 - | - 5 - | - 6 - | - 7 - |
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      +-------+-------+-------+-------+-------+-------+-------+-------+
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    // 2xC8:| - 8 - | - 9 - | - + - | - + - | - + - | - + - | - + - | - + - |
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      +-------+-------+-------+-------+-------+-------+-------+-------+
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    // 2xD0:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      +-------+-------+-------+-------+-------+-------+-------+-------+
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    // 2xD8:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      +-------+-------+-------+-------+-------+-------+-------+-------+
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    // 2xE0:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      +-------+-------+-------+-------+-------+-------+-------+-------+
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    // 2xE8:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      +-------+-------+-------+-------+-------+-------+-------+-------+
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    // 2xF0:| - + - | - + - | - + - | - + - | - + - | - + - | - + - | - + - |
    //      |   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      +-------+-------+-------+-------+-------+-------+-------+-------+
    // 2xF8:|   .   |   .   |   .   |   .   |   .   |   .   |   .   |   .   |
    //      `-------+-------+-------+-------+-------+-------+-------+-------'
    // More accurately, two bits in each byte determine each of the palette indices.
    //
    //       +---------------+
    //       | (0,0) | (1,0) |
    //       |       |       |
    //       +-------+-------+
    //       | (0,1) | (1,1) |
    //       |       |       |
    //       +-------+-------+
    pub fn bg_palette(&self, tile_x: usize, tile_y: usize) -> [u8; 4] {
        // / 4 because each byte controls 4x4 tiles. * 8 because 
        let attr_table_idx = (tile_y / 4) * 8 + (tile_x / 4);
        let attr_byte = self.vram[attr_table_idx + (ATTRIBUTE_TABLE_START - VRAM_START) as usize];  // note: still using hardcoded first nametable

        let palette_idx = match ((tile_x % 4) / 2, (tile_y % 4) / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            _ => unreachable!(),
        };

        // Multiply out 4 because each palette is 4 colors.
        let palette_start: usize = 1 + (palette_idx as usize) * 4;
        [
            self.palette_table[0],
            self.palette_table[palette_start],
            self.palette_table[palette_start + 1],
            self.palette_table[palette_start + 2],
        ]
    }

    pub fn sprite_palette(&self, palette_idx: u8) -> [u8; 4] {
        // 0x11 is where sprites start.
        let start = 0x11 + (palette_idx * 4) as usize;

        [
            // Dummy -- this should never be used.
            0,
            self.palette_table[start],
            self.palette_table[start + 1],
            self.palette_table[start + 2],
        ]
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.ppu_addr.get();

        self.increment_vram_addr();

        match addr {
            CHR_ROM_START..=CHR_ROM_END => {
                let result = self.internal_data_buffer;
                if let Some(chr_ram) = &mut self.chr_ram {
                    self.internal_data_buffer = chr_ram[addr as usize];
                } else {
                    self.internal_data_buffer = self.chr_rom[addr as usize];
                }
                result
            }
            VRAM_START..=VRAM_END => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            UNUSED_START..=UNUSED_END => panic!("addr space 0x3000 ~ 0x3eff should not be read from, requested = {}", addr),
            
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                self.palette_table[(addr - 0x10 - PALETTE_TABLE_START) as usize]
            }

            PALETTE_TABLE_START..=PALETTE_TABLE_END => {
                self.palette_table[(addr - PALETTE_TABLE_START) as usize]
            }

            _ => panic!("unexpected access to mirrored space {}", addr)
        }
    }

    pub fn read_oam_data(&mut self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn read_status(&mut self) -> u8 {
        let data = self.status.bits();
        self.status.set(PPUSTATUS::VBLANK_STARTED, false);
        self.ppu_addr.reset_write_latch();
        self.ppu_scroll.reset_latch();
        data
    }
    
    // Nametables:
    // [ 0 ] [ 1 ]
    // [ 2 ] [ 3 ]
    //
    // Horizontal: 
    // [ A ] [ a ]
    // [ B ] [ b ]
    //
    // Vertical: 
    // [ A ] [ B ]
    // [ a ] [ b ]
    //
    // Maps into VRAM.
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        // Maps into 0x2000 ~ 0x2fff, in case data is not there
        let mirrored_vram = addr & VRAM_END;
        let vram_index = mirrored_vram - VRAM_START;
        let name_table = vram_index / NAMETABLE_SIZE;
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - (2 * NAMETABLE_SIZE),
            (Mirroring::Horizontal, 2) => vram_index - NAMETABLE_SIZE,
            (Mirroring::Horizontal, 1) => vram_index - NAMETABLE_SIZE,
            (Mirroring::Horizontal, 3) => vram_index - (2 * NAMETABLE_SIZE),
            _ => vram_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ppu::{registers::status::PPUSTATUS, PPU};

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = PPU::default();
        ppu.status.set(PPUSTATUS::VBLANK_STARTED, true);

        let status = ppu.read_status();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status.bits() >> 7, 0);
    }

}