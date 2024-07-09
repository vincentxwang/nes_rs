//! Struct for the PPU mask register ($2002)
//! Reference: https://www.nesdev.org/wiki/PPU_registers#Status_($2002)_%3C_read

bitflags! {
    // 7654 3210
    // ---- ----
    // VSO. ....
    // |||| ||||
    // |||+-++++- PPU open bus. Returns stale PPU bus contents.
    // ||+------- Sprite overflow. The intent was for this flag to be set
    // ||         whenever more than eight sprites appear on a scanline, but a
    // ||         hardware bug causes the actual behavior to be more complicated
    // ||         and generate false positives as well as false negatives; see
    // ||         PPU sprite evaluation. This flag is set during sprite
    // ||         evaluation and cleared at dot 1 (the second dot) of the
    // ||         pre-render line.
    // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    // |          a nonzero background pixel; cleared at dot 1 of the pre-render
    // |          line.  Used for raster timing.
    // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
    //            Set at dot 1 of line 241 (the line *after* the post-render
    //            line); cleared after reading $2002 and at dot 1 of the
    //            pre-render line.
    pub struct PPUSTATUS: u8 {
        const UNUSED1           = 1 << 0;
        const UNUSED2           = 1 << 1;
        const UNUSED3           = 1 << 2;
        const UNUSED4           = 1 << 3;
        const UNUSED5           = 1 << 4;
        const SPRITE_OVERFLOW   = 1 << 5;
        const SPRITE_ZERO_HIT   = 1 << 6;
        const VBLANK_STARTED    = 1 << 7;
    }
}

impl PPUSTATUS {
    pub fn new() -> Self {
        PPUSTATUS::from_bits_truncate(0b0000_0000)
    }
}