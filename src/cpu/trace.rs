//! An nes_test-compatible tracer (https://www.qmtpro.com/~nes/misc/nestest.txt)

use std::collections::HashMap;
use crate::bus::Bus;
use crate::cpu::CPU;
use crate::cpu::AddressingMode;
use crate::cpu::operations::Operation;
use crate::cpu::opcodes::{self, UNOFFICIAL_OPCODES};
use crate::bus::*;

use super::Mem;

impl Bus {
    // Reads without doing any effects, or panicing.
    pub fn mem_read_debug(&self, addr: u16) -> u16 {
        match addr {
            // WRAP start (0x0000 -> 0x1fff)
            WRAM_START..=WRAM_END => {
                // Take the last 11 bits.
                let mirror_down_addr = addr & 0b111_1111_1111;
                self.cpu_wram[mirror_down_addr as usize] as u16
            }

            // PPU start (0x2000 -> 0x3fff)
            0x2000 => self.ppu.controller.bits() as u16,
            
            0x2001 => self.ppu.ppu_mask.bits() as u16,

            0x2002 => self.ppu.status.bits() as u16,

            0x2003 => self.ppu.oam_addr as u16,

            0x2004 => self.ppu.oam_data[self.ppu.oam_addr as usize] as u16,

            0x2005 => {
                // TODO: implement scroll get
                println!("dummy");
                42
            },

            0x2006 => self.ppu.ppu_addr.get(),

            // TODO: implement PPUDATA debug
            0x2007 => { 
                println!("dummy");
                42
            }

            // TODO: implement OAMDATA debug
            0x4014 => {
                println!("dummy");
                42
            }

            0x4016 => self.joypad.button_status.bits() as u16,

            PPU_MIRRORS_START..=PPU_MIRRORS_END => {
                // Mirrors $2008 - $4000 into $2000 - $2008
                // let mirror_down_addr = addr & 0b00100000_00000111;
                // self.mem_read(mirror_down_addr)
                // TODO: fix this lol
                println!("dummy");
                42
      
            },

            PRG_RAM_START..=PRG_RAM_END => self.read_prg_ram(addr) as u16,

            PRG_ROM_START..=PRG_ROM_END => self.read_prg_rom(addr) as u16,

            _ => {
                println!("Ignoring mem_read at BUS address {}", addr);
                0
            }
        }
    }
}

// TODO: add in PPU
pub fn trace(cpu: &mut CPU) -> String {
    let opscodes: &HashMap<u8, &'static opcodes::OpCode> = &opcodes::OPCODES_MAP;

    let code = cpu.mem_read(cpu.program_counter);
    let ops = opscodes.get(&code).expect(&format!("no opcode found for {}", code));

    let begin = cpu.program_counter;
    let mut hex_dump = vec![];
    hex_dump.push(code);

    let (mem_addr, stored_value) = match ops.addressing_mode {
        AddressingMode::Immediate | AddressingMode::NoneAddressing | AddressingMode::Indirect => (0, 0),
        _ => {
            let (addr, _) = cpu.get_absolute_address(&ops.addressing_mode, begin.wrapping_add(1));
            (addr, cpu.bus.mem_read_debug(addr))
            // (addr, 69)
        }
    };

    let tmp = match ops.bytes {
        1 => match ops.code {
            0x0a | 0x4a | 0x2a | 0x6a => "A ".to_string(),
            _ => String::from(""),
        },
        2 => {
            let address: u8 = cpu.mem_read(begin.wrapping_add(1));
            // let value = cpu.mem_read(address));
            hex_dump.push(address);

            match ops.addressing_mode {
                AddressingMode::Immediate => format!("#${:02x}", address),
                AddressingMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, stored_value),
                AddressingMode::ZeroPage_X => format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::ZeroPage_Y => format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::Indirect_X => format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    address,
                    (address.wrapping_add(cpu.register_x)),
                    mem_addr,
                    stored_value
                ),
                AddressingMode::Indirect_Y => format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    address,
                    (mem_addr.wrapping_sub(cpu.register_y as u16)),
                    mem_addr,
                    stored_value
                ),
                AddressingMode::NoneAddressing => {
                    // assuming local jumps: BNE, BVS, etc....
                    let address: usize =
                        (begin as usize + 2).wrapping_add((address as i8) as usize);
                    format!("${:04x}", address)
                }

                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 2. code {:02x}",
                    ops.addressing_mode, ops.code
                ),
            }
        }
        3 => {
            let address_lo = cpu.mem_read(begin + 1);
            let address_hi = cpu.mem_read(begin + 2);
            hex_dump.push(address_lo);
            hex_dump.push(address_hi);

            let address = cpu.mem_read_u16(begin + 1);

            match ops.addressing_mode {
                AddressingMode::NoneAddressing => {
                    format!("${:04x}", address)
                }
                AddressingMode::Absolute => {
                    if ops.op == Operation::JMP {
                        format!("${:04x}", mem_addr)
                    } else {
                        format!("${:04x} = {:02x}", mem_addr, stored_value)
                    }

                },
                AddressingMode::Absolute_X => format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::Absolute_Y => format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    address, mem_addr, stored_value
                ),
                AddressingMode::Indirect => {
                    let jmp_addr = if address & 0x00FF == 0x00FF {
                        let lo = cpu.mem_read(address);
                        let hi = cpu.mem_read(address & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        cpu.mem_read_u16(address)
                    };

                    // let jmp_addr = cpu.mem_read_u16(address);
                    format!("(${:04x}) = {:04x}", address, jmp_addr)
                },
                _ => panic!(
                    "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                    ops.addressing_mode, ops.code
                ),
            }
        }
        _ => String::from(""),
    };

    let hex_str = hex_dump
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let operation_str = if UNOFFICIAL_OPCODES.contains(&ops.code) {
        format!("*{}", ops.op)
    } else {
        ops.op.to_string()
    };
    let asm_str = format!(
        "{:04x}  {:8} {: >4} {}",
        begin,
        hex_str,
        operation_str,
        tmp
    )
    .trim()
    .to_string();

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x} PPU:{:>3},{:>3} CYC:{}",
        asm_str, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.stack_pointer, cpu.bus.ppu.scanline, cpu.bus.ppu.cycles, cpu.bus.cycles
    )
    .to_ascii_uppercase()
}


#[cfg(test)]
mod trace_test {
    use super::*;
    use crate::bus::Bus;
    use crate::cartridge::test::create_test_cartridge;
    use crate::cpu::{Mem, CPU};

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::default(create_test_cartridge());
        bus.mem_write(100, 0xa2);
        bus.mem_write(101, 0x01);
        bus.mem_write(102, 0xca);
        bus.mem_write(103, 0x88);
        bus.mem_write(104, 0x00);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_a = 1;
        cpu.register_x = 2;
        cpu.register_y = 3;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD PPU:  0, 21 CYC:7",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD PPU:  0, 27 CYC:9",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD PPU:  0, 33 CYC:11",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::default(create_test_cartridge());
        // ORA ($33), Y
        bus.mem_write(100, 0x11);
        bus.mem_write(101, 0x33);

        //data
        bus.mem_write(0x33, 0x00);
        bus.mem_write(0x34, 0x04);

        //target cell
        bus.mem_write(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_y = 0;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD PPU:  0, 21 CYC:7",
            result[0]
        );
    }
}
