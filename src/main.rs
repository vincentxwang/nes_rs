use macroquad::prelude::*;
use nes_rs::{bus::Bus, cartridge::Cartridge, cpu::CPU, ppu::PPU, render::{constants::*, frame::Frame}};

// Pixels are numbered from 0 to (256 * 200 - 256), from left to right, then up to down.
// Each is identified with an x and y coordinate.
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

    // let bytes: Vec<u8> = std::fs::read("tests/nestest/nestest.nes").unwrap();
    let bytes: Vec<u8> = std::fs::read("dk.nes").unwrap();
    let rom = Cartridge::new(&bytes).unwrap();

    let mut frame = Frame::new();
    
    let bus = Bus::new(rom, Box::from(move |ppu: &PPU| {

        Frame::render(&ppu, &mut frame);

        // let frame = Frame::show_tile_bank(&cpu.bus.ppu.chr_rom, 0);

        let mut index = 0;
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

    }));

    let mut cpu = CPU::new(bus);

    cpu.reset();

    let minimum_frame_time = 1. / 60.; // 60 FPS

    let frame_time = get_frame_time();

    println!("Frame time: {}ms", frame_time * 1000.);
    if frame_time < minimum_frame_time {
        let time_to_sleep = (minimum_frame_time - frame_time) * 1000.;
        println!("Sleep for {}ms", time_to_sleep);
        std::thread::sleep(std::time::Duration::from_millis(time_to_sleep as u64));
    }

    loop {
        cpu.run_once_with_callback(|x| {});

        next_frame().await;
    }
}