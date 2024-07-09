//! Struct for the PPU scroll register ($2005)
//! Reference: https://www.nesdev.org/wiki/PPU_registers#PPUMASK
pub struct PPUSCROLL {
    pub scroll_x: u8,
    pub scroll_y: u8,
    // Technically it shares the latch with the controller, but this doesn't really matter for implementation.
    // latch: false -> write to X; true -> write to Y
    pub latch: bool,
}

impl PPUSCROLL {
    pub fn new() -> Self {
        PPUSCROLL {
            scroll_x: 0,
            scroll_y: 0,
            latch: false,
        }
    }

    pub fn write(&mut self, data: u8) {
        if self.latch {
            self.scroll_y = data;
        } else {
            self.scroll_x = data;
        }
    }

    pub fn reset_latch(&mut self) {
        self.latch = false;
    }
}