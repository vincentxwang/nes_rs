use crate::ppu::{registers::controller::PPUCTRL, PPU};
use frame::Frame;
use palette::SYSTEM_PALETTE;

pub mod palette;
pub mod frame;
pub mod constants;

impl Frame {
    pub fn render(ppu: &PPU, frame: &mut Frame) {
        let bank: usize = ppu.controller.contains(PPUCTRL::BACKGROUND_PATTERN_ADDR) as usize * 0x1000;
    
        for i in 0..960 { // just for now, lets use the first nametable
            let tile = ppu.vram[i] as usize;
            let tile_x = i % 32;
            let tile_y = i / 32;

            let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];
     
            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];
     
                for x in (0..=7).rev() {
                    let value = (1 & upper) << 1 | (1 & lower);
                    upper = upper >> 1;
                    lower = lower >> 1;
                    let rgb = match value {
                        0 => SYSTEM_PALETTE[0x01],
                        1 => SYSTEM_PALETTE[0x23],
                        2 => SYSTEM_PALETTE[0x27],
                        3 => SYSTEM_PALETTE[0x30],
                        _ => panic!("can't be"),
                    };
                    frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb)
                }
            }
        }  
    }
}
