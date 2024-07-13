use crate::ppu::PPU;
use frame::Frame;
use palette::SYSTEM_PALETTE;

pub mod palette;
pub mod frame;
pub mod constants;

pub fn render(ppu: &PPU, frame: &mut Frame) {
    let bank: usize = 1;

    let mut frame = Frame::new();
    let mut tile_y = 0;
    let mut tile_x = 0;
    let bank = (bank * 0x1000) as usize;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 {
            tile_y += 10;
            tile_x = 0;
        }
        let tile = &ppu.chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

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
}  