//! Struct for the PPU controller register ($2000)
//! Reference: https://www.nesdev.org/wiki/PPU_registers#PPUCTRL

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
    
    pub struct PPUCTRL: u8 {
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

impl PPUCTRL {
    pub fn new() -> Self {
        PPUCTRL::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(PPUCTRL::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }
}

