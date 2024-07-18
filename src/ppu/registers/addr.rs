//! Struct for the PPU address register ($2006)
//! Reference: https://www.nesdev.org/wiki/PPU_registers#PPUADDR
//! Note that the PPU data register ($2007) is implemented as `PPU::write_data()`

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
            self.set(self.get() & 0x4000);
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
            self.set(self.get() & 0x4000);
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