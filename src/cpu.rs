// Reference: https://www.nesdev.org/obelisk-6502-guide/reference.html

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

pub enum Operation { 
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC, 
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, 
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI, 
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA 
}


bitflags! {
    // Status flags -- https://www.nesdev.org/wiki/Status_flags
    // 7654 3210
    // NV0B DIZC
    // |||| ||||
    // |||| |||+- Carry
    // |||| ||+-- Zero
    // |||| |+--- Interrupt Disable
    // |||| +---- Decimal
    // |||+------ (No CPU effect; see: the B flag)
    // ||+------- (No CPU effect; always pushed as 0)
    // |+-------- Overflow
    // +--------- Negative
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
    memory: [u8; 0xFFFF + 1] // + 1 for 2^16
}

// Stack occupied 0x0100 -> 0x01FF
const STACK: u16 = 0x0100;
// STACK + STACK_RESET is "top" of stack
const STACK_RESET: u8 = 0xfd;

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            program_counter: 0,
            stack_pointer: 0,
            // interrupt distable and negative initialized
            status: CPUFlags::from_bits_truncate(0b100100),
            memory: [0; 0xFFFF + 1],
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => self.mem_read(self.program_counter).wrapping_add(self.register_x) as u16,
            AddressingMode::ZeroPage_Y => self.mem_read(self.program_counter).wrapping_add(self.register_y) as u16,
            AddressingMode::Absolute_X => self.mem_read_u16(self.program_counter).wrapping_add(self.register_x as u16),
            AddressingMode::Absolute_Y => self.mem_read_u16(self.program_counter).wrapping_add(self.register_y as u16),
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
 
                let ptr: u8 = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
 
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                
                deref_base.wrapping_add(self.register_y as u16)
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
            AddressingMode::Indirect => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    // Reads 8 bits.
    pub fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    // Converts little-endian (used by NES) to big-endian
    pub fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }
 
    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos.wrapping_add(1), hi);
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
       self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
       self.mem_write_u16(0xFFFC, 0x8000);
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

        let carry = sum > 0xff;

        if carry {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }

        let result = sum as u8;

        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.status.insert(CPUFlags::OVERFLOW);
        } else {
            self.status.remove(CPUFlags::OVERFLOW)
        }

        self.set_register_a(result);
    }

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
            },

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

    fn lsr(&mut self, mode: &AddressingMode) {
        let mut data;
        let mut addr = 0; // Dummy
        // AddressingNone => Accumulator
        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = self.get_operand_address(mode);
                data = self.mem_read(addr);
            },

        }
        self.status.set(CPUFlags::CARRY, data & 1 == 1);
        data >>= 1;
        match mode {
            AddressingMode::NoneAddressing => self.register_a = data,
            _ => self.mem_write(addr, data),
        }
        self.update_zero_and_negative_flags(data);
    }

    fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.status.set(CPUFlags::CARRY,data <= compare_with);
        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.register_a ^= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a); // Unsure... documentation is too vague
    }

    fn dec(&mut self, mode: &AddressingMode){
        let addr = self.get_operand_address(mode);
        let val = self.mem_read(addr).wrapping_sub(1);

        self.mem_write(addr, val);
        self.update_zero_and_negative_flags(val);
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
            },
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

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x)
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y)
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
        self.status = CPUFlags::from_bits_retain((self.status.bits() & 0b0011_0000) | (data & 0b1100_1111));
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
            },
        }
    }

    fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(jump as u16);

            self.program_counter = jump_addr;
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
            },
        }
    }


    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.status.set(CPUFlags::ZERO, result == 0);
        self.status.set(CPUFlags::NEGATIVE, result & 0b1000_0000 != 0);
    }

    pub fn run(&mut self) {
        while self.run_once() {}
    }

    pub fn run_once(&mut self) -> bool {
        let code = self.mem_read(self.program_counter);
        self.program_counter = self.program_counter.wrapping_add(1);

        let opcode = CPU_OPS_CODES.iter().find(|opcode| opcode.code == code).unwrap_or_else(|| panic!("Invalid code {}", code));

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
            Operation::BRK => return false, // Assume BRK means program termination. We do not adjust the state of the CPU.
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
            },
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

        true
    }
}


#[cfg(test)]
mod test {
   use super::*;

   #[test]
   fn test_0xa9_lda_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
       assert_eq!(cpu.register_a, 0x05);
    //    assert!(cpu.status & 0b0000_0010 == 0b00);
    //    assert!(cpu.status & 0b1000_0000 == 0);
   }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        // assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
  
        assert_eq!(cpu.register_x, 0xc1)
    }
    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        // LDA (0xff)
        // TAX
        // INX
        // INX
        // BRK
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }    
    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }
    #[test]
    fn test_lda_sta_dec_and() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xA9, 0b1010_0010,      // LDA
            0x85, 0x87,             // STA, store 0x87 -> 0b1010_0010
            0xC6, 0x87,             // DEC
            0xC6, 0x87,             // DEC, register A now = 0b1010_0000
            0x25, 0x87              // AND
        ]);

        assert_eq!(cpu.register_a, 0b1010_0000)
    }
    #[test]
    fn test_lda_eor_and() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xA9, 0b0111_0110,      // LDA
            0x49, 0b1010_1100,      // EOR, A = 0b1101_1010
            0x29, 0b1010_1100,      // AND
        ]);

        assert_eq!(cpu.register_a, 0b1000_1000)
    }
    #[test]
    fn test_inc_ora() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xE6, 0x26,             // INC
            0x05, 0x26              // ORA
        ]);

        assert_eq!(cpu.register_a, 1)
    }
    #[test]
    fn test_stack_into_a() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xA2, 0x05,             // LDX #$05
            0xA9, 0x42,             // LDA #$42
            0x9D, 0x00, 0x02,       // STA $0200,X
            0xBD, 0x00, 0x02,       // LDA $0200,X
            0x48,                   // PHA
            0xA9, 0x00,             // LDA #$00
            0x68,                   // PLP
            0x00                    // BRK
        ]);

        assert_eq!(cpu.register_a, 0x42);
        assert_eq!(cpu.register_x, 0x05);
        assert_eq!(cpu.mem_read_u16(0x0205), 0x42);
    }
}