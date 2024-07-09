//! 6502 CPU implementation
//!
//! <http://wiki.nesdev.com/w/index.php/CPU>

use crate::cpu::operations::Operation;
use crate::bus::Bus;
use crate::cpu::opcodes::CPU_OPS_CODES;
use crate::cpu::addressing::AddressingMode;

pub mod trace;
mod operations;
mod opcodes;
mod addressing;


// Status flags -- https://www.nesdev.org/wiki/Status_flags
// 7654 3210
// NV0B DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// |||+------ (No CPU effect; see: the B flag)
// ||+------- (No CPU effect; always pushed as 1)
// |+-------- Overflow
// +--------- Negative
bitflags! {
    pub struct CPUFlags: u8 {
        const CARRY             = 1 << 0;
        const ZERO              = 1 << 1;
        const INTERRUPT_DISABLE = 1 << 2;
        const DECIMAL_MODE      = 1 << 3;
        const BREAK             = 1 << 4;
        const BREAK2            = 1 << 5; // not used, default = 1
        const OVERFLOW          = 1 << 6;
        const NEGATIVE          = 1 << 7;
    }
}

pub struct CPU {
    pub register_a: u8,
    pub status: CPUFlags,
    pub register_x: u8,
    pub register_y: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub bus: Bus,
}

// Stack occupied 0x0100 -> 0x01FF
const STACK: u16 = 0x0100;
// STACK + STACK_RESET is "top" of stack
const STACK_RESET: u8 = 0xfd;

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos.wrapping_add(1), hi);
    }
}

impl Mem for CPU {
    // This is a mut self because we need to increment VRAM address in PPU
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            bus,
            program_counter: 0,
            stack_pointer: STACK_RESET,
            // Interrupt disable (bit 2) and the unused (bit 5) initialized by default
            status: CPUFlags::from_bits_truncate(0b100100),
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = STACK_RESET;
        self.status = CPUFlags::from_bits_truncate(0b100100);

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // 0x8000 to 0xFFFF stores program ROM
        for i in 0..(program.len() as u16) {
            self.mem_write(0x0600 + i, program[i as usize]);
        }
        // self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(STACK + self.stack_pointer as u16)
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write(STACK + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    /// note: NES ignores decimal mode, unlike most 6502 processors
    /// http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    fn add_to_register_a(&mut self, data: u8) {
        let sum = self.register_a as u16
            + data as u16
            + (if self.status.contains(CPUFlags::CARRY) {
                1
            } else {
                0
            }) as u16;

        self.status.set(CPUFlags::CARRY, sum > 0xff);

        let result = sum as u8;

        self.status.set(
            CPUFlags::OVERFLOW,
            (data ^ result) & (result ^ self.register_a) & 0x80 != 0,
        );

        self.set_register_a(result);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.status.set(CPUFlags::ZERO, result == 0);
        self.status
            .set(CPUFlags::NEGATIVE, result & 0b1000_0000 != 0);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        // let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

        loop {
            callback(self);

            let code = self.mem_read(self.program_counter);
            self.program_counter = self.program_counter.wrapping_add(1);

            // TODO: implement a hashmap instead of this lookup
            let opcode = CPU_OPS_CODES
                .iter()
                .find(|opcode| opcode.code == code)
                .unwrap_or_else(|| panic!("Invalid code {}", code));

            match opcode.op {
                Operation::ADC => self.adc(&opcode.addressing_mode),
                Operation::AND => self.and(&opcode.addressing_mode),
                Operation::ASL => self.asl(&opcode.addressing_mode),
                Operation::BCC => self.branch(!self.status.contains(CPUFlags::CARRY)),
                Operation::BCS => self.branch(self.status.contains(CPUFlags::CARRY)),
                Operation::BEQ => self.branch(self.status.contains(CPUFlags::ZERO)),
                Operation::BIT => self.bit(&opcode.addressing_mode),
                Operation::BMI => self.branch(self.status.contains(CPUFlags::NEGATIVE)),
                Operation::BNE => self.branch(!self.status.contains(CPUFlags::ZERO)),
                Operation::BPL => self.branch(!self.status.contains(CPUFlags::NEGATIVE)),
                Operation::BRK => return, // Assume BRK means program termination. We do not adjust the state of the CPU.
                Operation::BVC => self.branch(!self.status.contains(CPUFlags::OVERFLOW)),
                Operation::BVS => self.branch(self.status.contains(CPUFlags::OVERFLOW)),
                Operation::CLC => self.status.remove(CPUFlags::CARRY),
                Operation::CLD => self.status.remove(CPUFlags::DECIMAL_MODE),
                Operation::CLI => self.status.remove(CPUFlags::INTERRUPT_DISABLE),
                Operation::CLV => self.status.remove(CPUFlags::OVERFLOW),
                Operation::CMP => self.compare(&opcode.addressing_mode, self.register_a),
                Operation::CPX => self.compare(&opcode.addressing_mode, self.register_x),
                Operation::CPY => self.compare(&opcode.addressing_mode, self.register_y),
                Operation::DCP => {
                    self.dec(&opcode.addressing_mode);
                    self.compare(&opcode.addressing_mode, self.register_a);
                }
                Operation::DEC => self.dec(&opcode.addressing_mode),
                Operation::DEX => self.dex(),
                Operation::DEY => self.dey(),
                Operation::EOR => self.eor(&opcode.addressing_mode),
                Operation::INC => self.inc(&opcode.addressing_mode),
                Operation::INX => self.inx(),
                Operation::INY => self.iny(),
                Operation::ISB => {
                    self.inc(&opcode.addressing_mode);
                    self.sbc(&opcode.addressing_mode);
                }
                Operation::JMP => self.jmp(&opcode.addressing_mode),
                Operation::JSR => self.jsr(),
                Operation::LAX => {
                    self.lda(&opcode.addressing_mode);
                    self.tax();
                },
                Operation::LDA => self.lda(&opcode.addressing_mode),
                Operation::LDX => self.ldx(&opcode.addressing_mode),
                Operation::LDY => self.ldy(&opcode.addressing_mode),
                Operation::LSR => self.lsr(&opcode.addressing_mode),
                Operation::NOP => (),
                Operation::ORA => self.ora(&opcode.addressing_mode),
                Operation::PHA => self.stack_push(self.register_a),
                Operation::PHP => self.stack_push(self.status.bits() | 0b0011_0000), // set break flag and bit 5 to be 1
                Operation::PLA => self.pla(),
                Operation::PLP => self.plp(),
                Operation::ROL => self.rol(&opcode.addressing_mode),
                Operation::ROR => self.ror(&opcode.addressing_mode),
                Operation::RLA => {
                    self.rol(&opcode.addressing_mode);
                    self.and(&opcode.addressing_mode);
                }
                Operation::RRA => {
                    self.ror(&opcode.addressing_mode);
                    self.adc(&opcode.addressing_mode);
                }
                Operation::RTI => {
                    self.plp();
                    self.program_counter = self.stack_pop_u16();
                }
                Operation::RTS => self.program_counter = self.stack_pop_u16().wrapping_add(1),
                Operation::SAX => self.sax(&opcode.addressing_mode),
                Operation::SBC => self.sbc(&opcode.addressing_mode),
                Operation::SEC => self.status.insert(CPUFlags::CARRY),
                Operation::SED => self.status.insert(CPUFlags::DECIMAL_MODE),
                Operation::SEI => self.status.insert(CPUFlags::INTERRUPT_DISABLE),
                Operation::SLO => {
                    self.asl(&opcode.addressing_mode);
                    self.ora(&opcode.addressing_mode);
                }
                Operation::SRE => {
                    self.lsr(&opcode.addressing_mode);
                    self.eor(&opcode.addressing_mode);
                }
                Operation::STA => self.sta(&opcode.addressing_mode),
                Operation::STX => self.stx(&opcode.addressing_mode),
                Operation::STY => self.sty(&opcode.addressing_mode),
                Operation::TAX => self.tax(),
                Operation::TAY => self.tay(),
                Operation::TSX => self.tsx(),
                Operation::TXA => self.txa(),
                Operation::TXS => self.stack_pointer = self.register_x,
                Operation::TYA => self.tya(),
            }

            // -1 because we already incremented program_counter to account for the instruction
            self.program_counter = self.program_counter.wrapping_add((opcode.bytes - 1) as u16);
        }
    }
}