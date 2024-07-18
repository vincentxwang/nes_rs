use crate::ppu::{registers::controller::PPUCTRL, PPU};
use constants::*;
use frame::Frame;
use palette::SYSTEM_PALETTE;

pub mod palette;
pub mod frame;
pub mod constants;

impl Frame {
    // Reads PPU to mutate frame object.
    pub fn render(ppu: &PPU, frame: &mut Frame) {

        println!("RENDER RENDER RENDER");

        let bank: usize = ppu.controller.contains(PPUCTRL::BACKGROUND_PATTERN_ADDR) as usize * 0x1000;
    
        for i in 0..960 { // just for now, lets use the first nametable
            let tile_index = ppu.vram[i] as usize;
            // println!("tile: {}", tile);
            let tile_x = i % 32;
            let tile_y = i / 32;

            let bg_palette = ppu.bg_palette(tile_x, tile_y);

            // println!("bank: {}, tile: {}", bank, tile);
            // println!("{}", ppu.chr_rom.len());

            let tile: &[u8]; 
            
            if let Some(chr_ram) = &ppu.chr_ram {
                tile = &chr_ram[(bank + tile_index * 16) as usize..=(bank + tile_index * 16 + 15) as usize];
            } else {
                tile = &ppu.chr_rom[(bank + tile_index * 16) as usize..=(bank + tile_index * 16 + 15) as usize];
            }

     
            for y in 0..=7 {
                let mut lower = tile[y];
                let mut upper = tile[y + 8];
     
                for x in (0..=7).rev() {
                    let value = (1 & upper) << 1 | (1 & lower);
                    upper = upper >> 1;
                    lower = lower >> 1;
                    let rgb = match value {
                        0 => SYSTEM_PALETTE[bg_palette[0] as usize],
                        1 => SYSTEM_PALETTE[bg_palette[1] as usize],
                        2 => SYSTEM_PALETTE[bg_palette[2] as usize],
                        3 => SYSTEM_PALETTE[bg_palette[3] as usize],
                        _ => unreachable!(),
                    };
                    frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb)
                }
            }
        }  
    }

    // Displays a Frame on the screen.
    pub fn show(frame: &Frame) {
        let mut index = 0;
        for j in 0..NES_PIXEL_HEIGHT {
            for i in 0..NES_PIXEL_WIDTH {
                macroquad::prelude::draw_rectangle(
                    (i * PIXEL_RATIO) as f32, 
                    // Add one because draw_rectangle requires the top-left corner.
                    ((j + 1) * PIXEL_RATIO) as f32, 
                    PIXEL_RATIO as f32, 
                    PIXEL_RATIO as f32, 
                    frame.data[index]);
                    
                index += 1;
            }
        }
    }
}
