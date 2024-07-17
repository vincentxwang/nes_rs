// The following test cases are taken from https://github.com/SingleStepTests/ProcessorTests/tree/main/nes6502 as of
// 6/22/2024. This setup assumes the following file naming: tests/harte_test_suite/nes6502/v1/[OPCODE].json.

#[cfg(test)]
mod harte {

    use nes_rs::cpu::trace::trace;
    use nes_rs::cpu::CPUFlags;
    use nes_rs::cpu::Mem;
    use nes_rs::cpu::CPU;
    use serde_json::{Result, Value};
    use nes_rs::cpu::opcodes::CPU_OPS_CODES;

    fn run_harte_test(v: &Value) -> Result<()> {
        // let v: Value = serde_json::from_str(data)?;
    
        let mut cpu = CPU::default();
    
        cpu.program_counter = v["initial"]["pc"].as_u64().expect("Unable to unwrap pc") as u16;
        cpu.stack_pointer = v["initial"]["s"].as_u64().expect("Unable to unwrap s") as u8;
        cpu.register_a = v["initial"]["a"].as_u64().expect("Unable to unwrap a") as u8;
        cpu.register_x = v["initial"]["x"].as_u64().expect("Unable to unwrap x") as u8;
        cpu.register_y = v["initial"]["y"].as_u64().expect("Unable to unwrap y") as u8;
        cpu.status =
            CPUFlags::from_bits_retain(v["initial"]["p"].as_u64().expect("Unable to unwrap p") as u8);
    
        let ram = v["initial"]["ram"]
            .as_array()
            .expect("Unable to unwrap ram");
    
        for pair in ram {
            let addr = pair[0].as_u64().unwrap() as u16;
            let data = pair[1].as_u64().unwrap() as u8;
            cpu.mem_write(addr, data);
        }
    
        // let program = process_instructions(v["name"].as_str().unwrap());
    
        // cpu.load(program);
        cpu.run_once_with_callback(move |x| {
            println!("{}", trace(x));
        });
    
        assert_eq!(
            cpu.register_a,
            v["final"]["a"].as_u64().expect("Unable to unwrap a") as u8
        );
        assert_eq!(
            cpu.register_x,
            v["final"]["x"].as_u64().expect("Unable to unwrap x") as u8
        );
        assert_eq!(
            cpu.register_y,
            v["final"]["y"].as_u64().expect("Unable to unwrap y") as u8
        );
        assert_eq!(
            cpu.program_counter,
            v["final"]["pc"].as_u64().expect("Unable to unwrap pc") as u16
        );
        assert_eq!(
            cpu.stack_pointer,
            v["final"]["s"].as_u64().expect("Unable to unwrap s") as u8
        );
        assert_eq!(
            cpu.status.bits(),
            v["final"]["p"].as_u64().expect("Unable to unwrap p") as u8
        );
    
        let ram_final = v["final"]["ram"]
            .as_array()
            .expect("Unable to unwrap final ram");
    
        for pair in ram_final {
            assert_eq!(
                cpu.mem_read(pair[0].as_u64().unwrap() as u16),
                pair[1].as_u64().unwrap() as u8
            )
        }
    
        Ok(())
    }
    
    fn run_single_opcode(opcode: &str) -> Result<()> {
        let filename = format!(
            "tests/harte/nes6502/v1/{}.json",
            opcode.to_string()
        );

        println!("{}", filename);
        let file: String = std::fs::read_to_string(filename).expect("File not found");
    
        let v: Value = serde_json::from_str(&file)?;
        let v_arr = v.as_array().unwrap();
    
        for i in 0..(v_arr.len() - 1) {
            println!("running test {:?}", i);
            run_harte_test(&v_arr[i]).expect(&format!("Failed on test {}", i));
        }
    
        Ok(())
    }
    
    #[test]
    fn run_all_opcodes() {
        for i in 0..CPU_OPS_CODES.len() {
            let result = format!("{:02x}", CPU_OPS_CODES[i].code);
            if &result != "00" {
                println!("running opcode: {}", result);
                run_single_opcode(&result);
            }
        }
    }

    #[test]
    fn test_opcode() {
        run_single_opcode("01");
    }
}
