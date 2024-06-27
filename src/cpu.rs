//! 6502 CPU implementation
//!
//! <http://wiki.nesdev.com/w/index.php/CPU>

use core::fmt;
use std::collections::HashMap;

use crate::bus::Bus;
use crate::opcodes;
use crate::opcodes::CPU_OPS_CODES;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

// Only official opcodes are implemented
// Reference: https://www.nesdev.org/obelisk-6502-guide/reference.html
#[derive(Debug)]
pub enum Operation {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
        const CARRY             = 1;
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
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
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
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

// CPU instruction functions

impl CPU {
    // Add with carry.
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.add_to_register_a(value);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.register_a &= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let mut data;
        let mut addr = 0; // Dummy
                          // AddressingNone => Accumulator
        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = self.get_operand_address(mode);
                data = self.mem_read(addr);
            }
        }
        self.status.set(CPUFlags::CARRY, data >> 7 == 1);
        data <<= 1;
        match mode {
            AddressingMode::NoneAddressing => self.register_a = data,
            _ => self.mem_write(addr, data),
        }
        self.update_zero_and_negative_flags(data);
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let res = self.register_a & data;

        self.status.set(CPUFlags::ZERO, res == 0);
        self.status.set(CPUFlags::NEGATIVE, data & 0b10000000 > 0);
        self.status.set(CPUFlags::OVERFLOW, data & 0b01000000 > 0);
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self.program_counter.wrapping_add(jump as u16);

            self.program_counter = jump_addr;
        }
    }

    fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.status.set(CPUFlags::CARRY, data <= compare_with);
        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.register_a ^= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a); // Unsure... documentation is too vague
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr).wrapping_sub(1);

        self.mem_write(addr, val);
        self.update_zero_and_negative_flags(val);
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x)
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y)
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let mem_address = self.mem_read_u16(self.program_counter);

        // We -2 because of the extra bytes added on to account for the length of the program
        // that we don't want.
        match mode {
            AddressingMode::Absolute => self.program_counter = mem_address.wrapping_sub(2),
            AddressingMode::Indirect => {
                let indirect_ref = if mem_address & 0x00FF == 0x00FF {
                    let lo = self.mem_read(mem_address);
                    let hi = self.mem_read(mem_address & 0xFF00);
                    (hi as u16) << 8 | (lo as u16)
                } else {
                    self.mem_read_u16(mem_address)
                };

                self.program_counter = indirect_ref.wrapping_sub(2);
            }
            _ => {
                panic!("Invalid mode {:?} in JMP", mode);
            }
        }
    }

    fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter + 2 - 1);
        let target_address = self.mem_read_u16(self.program_counter);
        // We -2 because of the extra bytes added on to account for the length of the program
        // that we don't want.
        self.program_counter = target_address.wrapping_sub(2);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_a = val;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_x = val;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_y = val;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let mut data;
        let mut addr = 0; // Dummy
                          // AddressingNone => Accumulator
        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = self.get_operand_address(mode);
                data = self.mem_read(addr);
            }
        }
        self.status.set(CPUFlags::CARRY, data & 1 == 1);
        data >>= 1;
        match mode {
            AddressingMode::NoneAddressing => self.register_a = data,
            _ => self.mem_write(addr, data),
        }
        self.update_zero_and_negative_flags(data);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_a |= val;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn pla(&mut self) {
        let data = self.stack_pop();
        self.set_register_a(data);
    }

    fn plp(&mut self) {
        let data = self.stack_pop();
        // ignore break flag and bit 5
        self.status =
            CPUFlags::from_bits_retain((self.status.bits() & 0b0011_0000) | (data & 0b1100_1111));
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.mem_write(addr, val.wrapping_add(1));
        self.update_zero_and_negative_flags(val.wrapping_add(1));
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let mut addr = 0;
        let mut data;
        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = self.get_operand_address(mode);
                data = self.mem_read(addr);
            }
        }

        let old_carry = self.status.contains(CPUFlags::CARRY);
        self.status.set(CPUFlags::CARRY, data & 1 == 1);
        data >>= 1;

        if old_carry {
            data |= 0b10000000;
        }

        match mode {
            AddressingMode::NoneAddressing => self.set_register_a(data),
            _ => {
                self.mem_write(addr, data);
                self.status.set(CPUFlags::NEGATIVE, data >> 7 == 1);
                self.status.set(CPUFlags::ZERO, data == 0);
            }
        }
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let mut addr = 0;
        let mut data;
        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = self.get_operand_address(mode);
                data = self.mem_read(addr);
            }
        }

        let old_carry = self.status.contains(CPUFlags::CARRY);
        self.status.set(CPUFlags::CARRY, data >> 7 == 1);
        data <<= 1;

        if old_carry {
            data |= 1;
        }

        match mode {
            AddressingMode::NoneAddressing => self.set_register_a(data),
            _ => {
                self.mem_write(addr, data);
                self.status.set(CPUFlags::NEGATIVE, data >> 7 == 1);
                self.status.set(CPUFlags::ZERO, data == 0);
            }
        }
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
            // interrupt distable and negative initialized
            status: CPUFlags::from_bits_truncate(0b100100),
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            _ => self.get_absolute_address(mode, self.program_counter),
        }
    }

    fn get_absolute_address(&self, mode: &AddressingMode, addr: u16) -> u16 {
        match mode {
            AddressingMode::ZeroPage => self.mem_read(addr) as u16,
            AddressingMode::Absolute => self.mem_read_u16(addr),
            AddressingMode::ZeroPage_X => self.mem_read(addr).wrapping_add(self.register_x) as u16,
            AddressingMode::ZeroPage_Y => self.mem_read(addr).wrapping_add(self.register_y) as u16,
            AddressingMode::Absolute_X => {
                self.mem_read_u16(addr).wrapping_add(self.register_x as u16)
            }
            AddressingMode::Absolute_Y => {
                self.mem_read_u16(addr).wrapping_add(self.register_y as u16)
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(addr);

                let ptr: u8 = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(addr);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);

                deref_base.wrapping_add(self.register_y as u16)
            }
            _ => {
                panic!("mode {:?} is not supported", mode);
            }
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

    /// note: ignoring decimal mode
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
                Operation::DEC => self.dec(&opcode.addressing_mode),
                Operation::DEX => self.dex(),
                Operation::DEY => self.dey(),
                Operation::EOR => self.eor(&opcode.addressing_mode),
                Operation::INC => self.inc(&opcode.addressing_mode),
                Operation::INX => self.inx(),
                Operation::INY => self.iny(),
                Operation::JMP => self.jmp(&opcode.addressing_mode),
                Operation::JSR => self.jsr(),
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
                Operation::RTI => {
                    self.plp();
                    self.program_counter = self.stack_pop_u16();
                }
                Operation::RTS => self.program_counter = self.stack_pop_u16().wrapping_add(1),
                Operation::SBC => self.sbc(&opcode.addressing_mode),
                Operation::SEC => self.status.insert(CPUFlags::CARRY),
                Operation::SED => self.status.insert(CPUFlags::DECIMAL_MODE),
                Operation::SEI => self.status.insert(CPUFlags::INTERRUPT_DISABLE),
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

pub fn trace(cpu: &CPU) -> String {
    let opscodes: &HashMap<u8, &'static opcodes::OpCode> = &opcodes::OPCODES_MAP;

    let code = cpu.mem_read(cpu.program_counter);
    let ops = opscodes.get(&code).unwrap();

    let begin = cpu.program_counter;
    let mut hex_dump = vec![];
    hex_dump.push(code);

    let (mem_addr, stored_value) = match ops.addressing_mode {
        AddressingMode::Immediate | AddressingMode::NoneAddressing | AddressingMode::Indirect => (0, 0),
        _ => {
            let addr = cpu.get_absolute_address(&ops.addressing_mode, begin.wrapping_add(1));
            (addr, cpu.mem_read(addr))
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
                AddressingMode::Absolute => format!("${:04x} = {:02x}", mem_addr, stored_value),
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
    let asm_str = format!(
        "{:04x}  {:8} {: >4} {}",
        begin,
        hex_str,
        ops.op.to_string(),
        tmp
    )
    .trim()
    .to_string();

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
        asm_str, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.stack_pointer,
    )
    .to_ascii_uppercase()
}

#[cfg(test)]
mod test {
    
    

    // #[test]
    // fn test_0xa9_lda_immediate_load_data() {
    //     let cart = test::create_test_cartridge(&mut vec![0xa9, 0x05, 0x00]);
    //     let mut cpu = CPU::new(Bus::new(cart));
    //     cpu.reset();
    //     cpu.run();
    //     assert_eq!(cpu.register_a, 0x05);
    //     //    assert!(cpu.status & 0b0000_0010 == 0b00);
    //     //    assert!(cpu.status & 0b1000_0000 == 0);
    // }

    // #[test]
    // fn test_0xa9_lda_zero_flag() {
    //     let mut cpu = CPU::new();
    //     cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
    //     // assert!(cpu.status & 0b0000_0010 == 0b10);
    // }

    // #[test]
    // fn test_5_ops_working_together() {
    //     let mut cpu = CPU::new();

    //     cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

    //     assert_eq!(cpu.register_x, 0xc1)
    // }
    // #[test]
    // fn test_inx_overflow() {
    //     let mut cpu = CPU::new();
    //     // LDA (0xff)
    //     // TAX
    //     // INX
    //     // INX
    //     // BRK
    //     cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

    //     assert_eq!(cpu.register_x, 1)
    // }
    // #[test]
    // fn test_lda_from_memory() {
    //     let mut cpu = CPU::new();
    //     cpu.mem_write(0x10, 0x55);

    //     cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

    //     assert_eq!(cpu.register_a, 0x55);
    // }
    // #[test]
    // fn test_lda_sta_dec_and() {
    //     let mut cpu = CPU::new();
    //     cpu.load_and_run(vec![
    //         0xA9,
    //         0b1010_0010, // LDA
    //         0x85,
    //         0x87, // STA, store 0x87 -> 0b1010_0010
    //         0xC6,
    //         0x87, // DEC
    //         0xC6,
    //         0x87, // DEC, register A now = 0b1010_0000
    //         0x25,
    //         0x87, // AND
    //     ]);

    //     assert_eq!(cpu.register_a, 0b1010_0000)
    // }
    // #[test]
    // fn test_lda_eor_and() {
    //     let mut cpu = CPU::new();
    //     cpu.load_and_run(vec![
    //         0xA9,
    //         0b0111_0110, // LDA
    //         0x49,
    //         0b1010_1100, // EOR, A = 0b1101_1010
    //         0x29,
    //         0b1010_1100, // AND
    //     ]);

    //     assert_eq!(cpu.register_a, 0b1000_1000)
    // }
    // #[test]
    // fn test_inc_ora() {
    //     let mut cpu = CPU::new();
    //     cpu.load_and_run(vec![
    //         0xE6, 0x26, // INC
    //         0x05, 0x26, // ORA
    //     ]);

    //     assert_eq!(cpu.register_a, 1)
    // }
}

#[cfg(test)]
mod trace_test {
    use super::*;
    use crate::bus::Bus;
    use crate::cartridge::test::create_test_cartridge;
    use crate::cpu::CPU;

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new(create_test_cartridge());
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
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new(create_test_cartridge());
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
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }
}
