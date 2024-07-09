//! Struct for the PPU mask register ($2001)
//! Reference: https://www.nesdev.org/wiki/PPU_registers#PPUMASK

bitflags! {
    // 7654 3210
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue
    pub struct PPUMASK: u8 {
        const GREYSCALE             = 1 << 0;
        const SHOW_BACKGROUND_LEFT  = 1 << 1;
        const SHOW_SPRITES_LEFT     = 1 << 2;
        const SHOW_BACKGROUND       = 1 << 3;
        const SHOW_SPRITES          = 1 << 4;
        const EMPHASIZE_RED         = 1 << 5;
        const EMPHASIZE_GREEN       = 1 << 6;
        const EMPHASIZE_BLUE        = 1 << 7;
    }
}

impl PPUMASK {
    pub fn new() -> Self {
        PPUMASK::from_bits_truncate(0b0000_0000)
    }
}