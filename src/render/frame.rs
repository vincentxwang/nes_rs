use macroquad::color::Color;
use macroquad::color::colors::*;
use crate::render::constants::*;
use crate::render::palette::SYSTEM_PALETTE;

pub struct Frame {
    pub data: Vec<Color>,
}

impl Frame {
 
    pub fn new() -> Self {
        Frame {
            data: vec![PINK; (NES_PIXEL_WIDTH as usize) * (NES_PIXEL_HEIGHT as usize)],
        }
    }
    
    // Sets the golor of a single pixel defined by (x,y) to rgb values.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let base = y * (NES_PIXEL_WIDTH as usize) + x;
        self.data[base] = color;
    }
    
    // Reference: https://www.nesdev.org/wiki/PPU_memory_map
    fn show_tile(chr_rom: &Vec<u8>, bank: usize, tile_n: usize) -> Frame {
        assert!(bank <= 1);
     
        let mut frame = Frame::new();
        let bank = (bank * PATTERN_TABLE_SIZE) as usize;
     
        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];
     
        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];
     
            for x in (0..=7).rev() {
                // If neither bit is set to 1: The pixel is background/transparent.
                // If only the bit in the first plane is set to 1: The pixel's color index is 1.
                // If only the bit in the second plane is set to 1: The pixel's color index is 2.
                // If both bits are set to 1: The pixel's color index is 3.
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                frame.set_pixel(x, y, SYSTEM_PALETTE[value as usize])
            }
        }
     
        frame
    }
    
    pub fn show_tile_bank(chr_rom: &Vec<u8>, bank: usize) -> Frame {

        assert!(bank <= 1);
    
        let mut frame = Frame::new();
        let mut tile_y = 0;
        let mut tile_x = 0;
        let bank = (bank * 0x1000) as usize;
    
        for tile_n in 0..255 {
            if tile_n != 0 && tile_n % 20 == 0 {
                tile_y += 10;
                tile_x = 0;
            }
            let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];
    
            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];
    
                for x in (0..=7).rev() {
                    let value = (1 & upper) << 1 | (1 & lower);
                    upper = upper >> 1;
                    lower = lower >> 1;
                    frame.set_pixel(tile_x + x, tile_y + y, SYSTEM_PALETTE[value as usize])
                }
            }
    
            tile_x += 10;
        }
        frame
    }
}