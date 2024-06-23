use serde_json::{Result, Value};
use nes_rs::cpu::CPU;


fn process_instructions(input: &str) -> Vec<u8> {
    input
        .split_whitespace() // Split by whitespace
        .filter_map(|hex_str| u8::from_str_radix(hex_str, 16).ok()) // Parse each hex string to u8
        .collect() // Collect into Vec<u8>
}

#[test]
fn untyped_example() -> Result<()> {
    // Some JSON input data as a &str. Maybe this comes from the user.

    let data = r#"
    {
        "name": "b1 28 b5",
        "initial": {
            "pc": 59082,
            "s": 39,
            "a": 57,
            "x": 33,
            "y": 174,
            "p": 96,
            "ram": [
                [59082, 177],
                [59083, 40],
                [59084, 181],
                [40, 160],
                [41, 233],
                [59982, 119]
            ]
        },
        "final": {
            "pc": 59084,
            "s": 39,
            "a": 119,
            "x": 33,
            "y": 174,
            "p": 96,
            "ram": [
                [40, 160],
                [41, 233],
                [59082, 177],
                [59083, 40],
                [59084, 181],
                [59982, 119]
            ]
        },
        "cycles": [
            [59082, 177, "read"],
            [59083, 40, "read"],
            [40, 160, "read"],
            [41, 233, "read"],
            [59083, 40, "read"],
            [59982, 119, "read"]
        ]
    }"#;

    let v: Value = serde_json::from_str(data)?;

    let mut cpu = CPU::new();

    cpu.program_counter = v["initial"]["pc"].as_u64().expect("Unable to unwrap pc") as u16;
    cpu.stack_pointer = v["initial"]["s"].as_u64().expect("Unable to unwrap s") as u8;
    cpu.register_a = v["initial"]["a"].as_u64().expect("Unable to unwrap a") as u8;
    cpu.register_x = v["initial"]["x"].as_u64().expect("Unable to unwrap x") as u8;
    cpu.register_y = v["initial"]["y"].as_u64().expect("Unable to unwrap y") as u8;
    cpu.status.set_flags(v["initial"]["p"].as_u64().expect("Unable to unwrap p") as u8);

    let ram = v["initial"]["ram"].as_array().expect("Unable to unwrap ram");

    for pair in ram {
        let addr = pair[0].as_u64().unwrap() as u16;
        let data = pair[1].as_u64().unwrap() as u8;
        cpu.mem_write(addr, data);
    }

    let program = process_instructions(v["name"].as_str().unwrap());

    cpu.load(program);
    cpu.run();

    println!("a: {:?}", cpu.register_a);
    println!("x: {:?}", cpu.register_x);
    println!("y: {:?}", cpu.register_y);
    
    Ok(())
}