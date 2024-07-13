use lazy_static::lazy_static;
use macroquad::prelude::*;
use nes_rs::{bus::Bus, cartridge::Cartridge, cpu::CPU};

// Pixels are numbered from 0 to (256 * 200 - 256), from left to right, then up to down.
// Each is identified with an x and y coordinate.

const SCREEN_WIDTH: i32 = 1024;
const SCREEN_HEIGHT: i32 = 800;
const NES_PIXEL_WIDTH: i32 = 256;
const NES_PIXEL_HEIGHT: i32 = 200;
const NES_PIXEL_WIDTH_FLOAT: f32 = 256.0;
const NES_PIXEL_HEIGHT_FLOAT: f32 = 200.0;
const PIXEL_RATIO: i32 = SCREEN_WIDTH / NES_PIXEL_WIDTH;
const PIXEL_RATIO_FLOAT: f32 = 1024.0 / NES_PIXEL_WIDTH_FLOAT;

const PATTERN_TABLE_SIZE: usize = 0x1000;


pub struct Frame {
    pub data: Vec<Color>,
}

lazy_static! {
    pub static ref SYSTEM_PALETTE: [Color; 64] = [
    Color::from_rgba(0x80, 0x80, 0x80, 255), Color::from_rgba(0x00, 0x3D, 0xA6, 255), Color::from_rgba(0x00, 0x12, 0xB0, 255), Color::from_rgba(0x44, 0x00, 0x96, 255), Color::from_rgba(0xA1, 0x00, 0x5E, 255),
    Color::from_rgba(0xC7, 0x00, 0x28, 255), Color::from_rgba(0xBA, 0x06, 0x00, 255), Color::from_rgba(0x8C, 0x17, 0x00, 255), Color::from_rgba(0x5C, 0x2F, 0x00, 255), Color::from_rgba(0x10, 0x45, 0x00, 255),
    Color::from_rgba(0x05, 0x4A, 0x00, 255), Color::from_rgba(0x00, 0x47, 0x2E, 255), Color::from_rgba(0x00, 0x41, 0x66, 255), Color::from_rgba(0x00, 0x00, 0x00, 255), Color::from_rgba(0x05, 0x05, 0x05, 255),
    Color::from_rgba(0x05, 0x05, 0x05, 255), Color::from_rgba(0xC7, 0xC7, 0xC7, 255), Color::from_rgba(0x00, 0x77, 0xFF, 255), Color::from_rgba(0x21, 0x55, 0xFF, 255), Color::from_rgba(0x82, 0x37, 0xFA, 255),
    Color::from_rgba(0xEB, 0x2F, 0xB5, 255), Color::from_rgba(0xFF, 0x29, 0x50, 255), Color::from_rgba(0xFF, 0x22, 0x00, 255), Color::from_rgba(0xD6, 0x32, 0x00, 255), Color::from_rgba(0xC4, 0x62, 0x00, 255),
    Color::from_rgba(0x35, 0x80, 0x00, 255), Color::from_rgba(0x05, 0x8F, 0x00, 255), Color::from_rgba(0x00, 0x8A, 0x55, 255), Color::from_rgba(0x00, 0x99, 0xCC, 255), Color::from_rgba(0x21, 0x21, 0x21, 255),
    Color::from_rgba(0x09, 0x09, 0x09, 255), Color::from_rgba(0x09, 0x09, 0x09, 255), Color::from_rgba(0xFF, 0xFF, 0xFF, 255), Color::from_rgba(0x0F, 0xD7, 0xFF, 255), Color::from_rgba(0x69, 0xA2, 0xFF, 255),
    Color::from_rgba(0xD4, 0x80, 0xFF, 255), Color::from_rgba(0xFF, 0x45, 0xF3, 255), Color::from_rgba(0xFF, 0x61, 0x8B, 255), Color::from_rgba(0xFF, 0x88, 0x33, 255), Color::from_rgba(0xFF, 0x9C, 0x12, 255),
    Color::from_rgba(0xFA, 0xBC, 0x20, 255), Color::from_rgba(0x9F, 0xE3, 0x0E, 255), Color::from_rgba(0x2B, 0xF0, 0x35, 255), Color::from_rgba(0x0C, 0xF0, 0xA4, 255), Color::from_rgba(0x05, 0xFB, 0xFF, 255),
    Color::from_rgba(0x5E, 0x5E, 0x5E, 255), Color::from_rgba(0x0D, 0x0D, 0x0D, 255), Color::from_rgba(0x0D, 0x0D, 0x0D, 255), Color::from_rgba(0xFF, 0xFF, 0xFF, 255), Color::from_rgba(0xA6, 0xFC, 0xFF, 255),
    Color::from_rgba(0xB3, 0xEC, 0xFF, 255), Color::from_rgba(0xDA, 0xAB, 0xEB, 255), Color::from_rgba(0xFF, 0xA8, 0xF9, 255), Color::from_rgba(0xFF, 0xAB, 0xB3, 255), Color::from_rgba(0xFF, 0xD2, 0xB0, 255),
    Color::from_rgba(0xFF, 0xEF, 0xA6, 255), Color::from_rgba(0xFF, 0xF7, 0x9C, 255), Color::from_rgba(0xD7, 0xE8, 0x95, 255), Color::from_rgba(0xA6, 0xED, 0xAF, 255), Color::from_rgba(0xA2, 0xF2, 0xDA, 255),
    Color::from_rgba(0x99, 0xFF, 0xFC, 255), Color::from_rgba(0xDD, 0xDD, 0xDD, 255), Color::from_rgba(0x11, 0x11, 0x11, 255), Color::from_rgba(0x11, 0x11, 0x11, 255)
];
}

impl Frame {
 
    pub fn new() -> Self {
        Frame {
            data: vec![BLACK; (NES_PIXEL_WIDTH as usize) * (NES_PIXEL_HEIGHT as usize)],
        }
    }
    
    // Sets the golor of a single pixel defined by (x,y) to rgb values.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let base = y * (NES_PIXEL_WIDTH as usize) + x;
        self.data[base] = color;
    }

    pub fn test_pixel(&mut self, x: usize, y: usize, color: Color) -> usize {
        let base = y * (NES_PIXEL_WIDTH as usize) + x;
        return base
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

    fn show_tile_bank(chr_rom: &Vec<u8>, bank: usize) ->Frame {
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
 

fn nes_rs() -> Conf {
    Conf {
        window_title: "nes_rs".to_owned(),
        window_width: SCREEN_WIDTH,
        window_height: SCREEN_HEIGHT,
        ..Default::default()
    }
}

#[macroquad::main(nes_rs)]
async fn main() {

    let bytes: Vec<u8> = std::fs::read("pacman.nes").unwrap();
    let rom = Cartridge::new(&bytes).unwrap();

    // let bus = Bus::new(rom);
    // let mut cpu = CPU::new(bus);

    // cpu.reset();

    let mut frame = Frame::show_tile_bank(&rom.chr_rom, 1);
    let minimum_frame_time = 1. / 60.; // 60 FPS

    let frame_time = get_frame_time();

    println!("{}", frame.test_pixel(0, 1, PINK));

    println!("Frame time: {}ms", frame_time * 1000.);
    if frame_time < minimum_frame_time {
        let time_to_sleep = (minimum_frame_time - frame_time) * 1000.;
        println!("Sleep for {}ms", time_to_sleep);
        std::thread::sleep(std::time::Duration::from_millis(time_to_sleep as u64));
    }

    let mut temp = true;
    loop {
        let mut index = 0;

        if temp {
            println!("{:?}", frame.data[256]);
            temp = false;
        }
 

        for j in 0..NES_PIXEL_HEIGHT {
            for i in 0..NES_PIXEL_WIDTH {

                draw_rectangle(
                    (i * PIXEL_RATIO) as f32, 
                    // Add one because draw_rectangle requires the top-left corner.
                    ((j + 1) * PIXEL_RATIO) as f32, 
                    PIXEL_RATIO as f32, 
                    PIXEL_RATIO as f32, 
                    frame.data[index]);
                    
                index += 1;
            }
        }
        next_frame().await;
    }
}