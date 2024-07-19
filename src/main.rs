use macroquad::prelude::*;
use nes_rs::{bus::Bus, cartridge::Cartridge, cpu::CPU, render::constants::*};

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

    let bytes: Vec<u8> = std::fs::read("balloon.nes").unwrap();
    let rom = Cartridge::new(&bytes).unwrap();

    // let mut frame = Frame::new();
    
    let bus = Bus::new(rom);

    let mut cpu = CPU::new(bus);

    cpu.reset();

    // let minimum_frame_time = 1. / 60.; // 60 FPS

    // let frame_time = get_frame_time();

    // println!("Frame time: {}ms", frame_time * 1000.);
    // if frame_time < minimum_frame_time {
    //     let time_to_sleep = (minimum_frame_time - frame_time) * 1000.;
    //     println!("Sleep for {}ms", time_to_sleep);
    //     std::thread::sleep(std::time::Duration::from_millis(time_to_sleep as u64));
    // }

    loop {
        cpu.run_once_with_callback(move |_| {
                // println!("{}", trace::trace(cpu));
        });

        next_frame().await;
    }
}

// fn main() {
//     let bytes: Vec<u8> = std::fs::read("tests/blarggcpu/rom_singles/03-immediate.nes").unwrap();
//     let rom = Cartridge::new(&bytes).unwrap();

//     let bus = Bus::default(rom);

//     let mut cpu = CPU::new(bus);
//     cpu.reset();
//     // cpu.program_counter = 0xE000;

//     // let master: String = fs::read_to_string("tests/nestest/nestestmaster.log").unwrap();

//     // let cursor = std::io::Cursor::new(master);
//     // let mut lines_iter = cursor.lines().map(|l| l.unwrap());

//     cpu.run_with_callback(move |cpu| {
//         // println!("{}", trace::trace(cpu));
//         // println!("{}", cpu.mem_read(0x6000));
//         let mut count: u16 = 0x6000;
//         while cpu.mem_read(count) != 0 {
//             print!("{} ", cpu.mem_read(count));
//             count += 1;
//         }
//         println!("{}", trace::trace(cpu));
//     });
// }
