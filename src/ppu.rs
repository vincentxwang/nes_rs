pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
 
    pub controller: ControlRegister,
    pub ppu_addr: PPUADDR,
    // pub mirroring: Mirroring,
}

bitflags! {
// 7654 3210
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels – see PPU OAM#Byte 1)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)

    pub struct ControlRegister: u8 {
        const NAMETABLE1                = 1 << 0;
        const NAMETABLE2                = 1 << 1;
        const VRAM_ADD_INCREMENT        = 1 << 2;
        const SPRITE_PATTERN_ADDR       = 1 << 3;
        const BACKGROUND_PATTERN_ADDR   = 1 << 4;
        const SPRITE_SIZE               = 1 << 5;
        const MASTER_SLAVE_SELECT       = 1 << 6;
        const GENERATE_NMI              = 1 << 7;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }
}

impl PPU {
    fn write_to_ppu_addr(&mut self, value: u8) {
        self.ppu_addr.update(value);
    }

    fn write_to_controller(&mut self, value: u8) {
        self.controller = ControlRegister::from_bits_truncate(value);
    }

    fn increment_vram_addr(&mut self) {
        if !self.controller.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            self.ppu_addr.increment(1);
        } else {
            self.ppu_addr.increment(32);
        }
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.ppu_addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1fff => todo!("read from chr_rom"),
            0x2000..=0x2fff => todo!("read from RAM"),
            0x3000..=0x3eff => panic!("addr space 0x3000 ~ 0x3eff should not be read from, requested = {}", addr),
            0x3f00..=0x3fff => 
            {
                self.palette_table[(addr - 0x3f00) as usize]
            }
            _ => panic!("unexpected access to mirrored space {}", addr)
        }
    }
}

pub struct PPUADDR {
    // high byte, then low byte
    value: (u8, u8),
    // tracks if we are writing to the high byte
    write_latch: bool,
}

// Address register corresponds to 0x2006.
impl PPUADDR {
    pub fn new() -> Self {
        PPUADDR {
            value: (0, 0),
            write_latch: true,
        }
    }

    // Sets value to be appropriate high/low bytes from u16.
    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xff) as u8;
    }

    // Writes only one byte to PPUADDR.
    pub fn update(&mut self, data: u8) {
        if self.write_latch {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        // Mirrors down in case result is greater than the valid address range.
        if self.get() > 0x3fff {
            self.set(self.get() & 0x3fff);
        }

        self.write_latch = !self.write_latch;
    }

    // Increments PPUADDR by inc
    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);

        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        // Mirrors down in case result is greater than the valid address range.
        if self.get() > 0x3fff {
            self.set(self.get() & 0x3fff);
        }
    }

    pub fn reset_write_latch(&mut self) {
        self.write_latch = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ppuaddr_get_set_inc() {
        let mut ppu_addr = PPUADDR::new();
        assert_eq!(ppu_addr.get(), 0);
        ppu_addr.set(100);
        assert_eq!(ppu_addr.get(), 100);
        ppu_addr.increment(50);
        assert_eq!(ppu_addr.get(), 150);
    }

    #[test]
    fn test_ppuaddr_update() {
        let mut ppu_addr = PPUADDR::new();
        // Write high bit
        ppu_addr.update(0b0011_0000);
        // Write low bit
        ppu_addr.update(0b1110_0000);
        assert_eq!(ppu_addr.get(), 0b0011_0000_1110_0000);
    }

    #[test]
    fn test_ppuaddr_reset_latch() {
        let mut ppu_addr = PPUADDR::new();
        // Write high bit
        ppu_addr.update(0b0011_0010);
        ppu_addr.reset_write_latch();
        // Write to high bit (again)
        ppu_addr.update(0b0011_1001);
        assert_eq!(ppu_addr.get(), 0b0011_1001_0000_0000)
    }

    #[test]
    fn test_ppuaddr_wraparaound() {
        let mut ppu_addr = PPUADDR::new();
        ppu_addr.update(0b1111_1111);
        ppu_addr.update(0b1111_1101);
        assert_eq!(ppu_addr.get(), 0b1111_1111_1111_1101 & 0x3fff)
    }
}