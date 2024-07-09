//! Operation logic
//! Reference (official): https://www.nesdev.org/obelisk-6502-guide/reference.html
//! Reference (unofficial): https://www.oxyron.de/html/opcodes02.html

use core::fmt;
use crate::cpu::CPU;
use crate::cpu::addressing::AddressingMode;
use crate::cpu::Mem;
use crate::cpu::CPUFlags;

#[derive(Debug, PartialEq)]
pub enum Operation {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
    // Unofficial opcodes
    LAX, SAX, DCP, ISB, SLO, RLA, SRE, RRA
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl CPU {
    // Add with carry
    pub fn adc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.add_to_register_a(value);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Logical AND
    pub fn and(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        self.register_a &= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Arithmetic shift left
    pub fn asl(&mut self, mode: &AddressingMode) {
        let mut data;
        let mut addr: Option<u16> = None;
        // AddressingNone corresponds to shifting the accumulator left, and addr = None in this case.

        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = Some(self.get_operand_address(mode).0);
                data = self.mem_read(addr.unwrap());
            }
        }
        self.status.set(CPUFlags::CARRY, data >> 7 == 1);
        data <<= 1;
        match mode {
            AddressingMode::NoneAddressing => self.register_a = data,
            _ => self.mem_write(addr.unwrap(), data),
        }
        self.update_zero_and_negative_flags(data);
    }

    // Bit test
    pub fn bit(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let res = self.register_a & data;

        self.status.set(CPUFlags::ZERO, res == 0);
        self.status.set(CPUFlags::NEGATIVE, data & 0b10000000 > 0);
        self.status.set(CPUFlags::OVERFLOW, data & 0b01000000 > 0);
    }

    // Branches if condition = true
    pub fn branch(&mut self, condition: bool) {
        if condition {
            self.bus.tick(1);

            let base = self.program_counter;
            // NES converts this address into a signed 8-bit integer
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = base.wrapping_add(jump as u16);

            self.program_counter = jump_addr;

            if CPU::page_cross(base, jump_addr) {
                self.bus.tick(1);
            }
        }
    }

    pub fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.status.set(CPUFlags::CARRY, data <= compare_with);
        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Exclusive OR
    pub fn eor(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        self.register_a ^= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Decrement memory
    pub fn dec(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let val = self.mem_read(addr).wrapping_sub(1);

        self.mem_write(addr, val);
        self.update_zero_and_negative_flags(val);
    }

    // Decrement X register
    pub fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x)
    }

    // Decrement Y register
    pub fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y)
    }

    // Jump
    pub fn jmp(&mut self, mode: &AddressingMode) {
        let mem_address = self.mem_read_u16(self.program_counter);

        // We -2 because of there are extra bytes added on later that account for the length of the JMP opcode and address
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

    // Jump to subroutine
    pub fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter + 2 - 1);
        let target_address = self.mem_read_u16(self.program_counter);
        // We -2 because of there are extra bytes added on later that account for the length of the JMP opcode and address
        // that we don't want.
        self.program_counter = target_address.wrapping_sub(2);
    }

    // (Unofficial) Store bitwise AND of accumulator and X
    pub fn sax(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x & self.register_a);
    }

    // Store accumulator
    pub fn sta(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    // Store X register
    pub fn stx(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    // Store Y register
    pub fn sty(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    // Load into accumulator
    pub fn lda(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_a = val;
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Load into X register
    pub fn ldx(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_x = val;
        self.update_zero_and_negative_flags(self.register_x);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Load into Y register
    pub fn ldy(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_y = val;
        self.update_zero_and_negative_flags(self.register_y);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Logical shift right
    pub fn lsr(&mut self, mode: &AddressingMode) {
        let mut data;
        let mut addr : Option<u16> = None; 
        // AddressingNone corresponds to shifting the accumulator left, and addr = None in this case.

        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = Some(self.get_operand_address(mode).0);
                data = self.mem_read(addr.unwrap());
            }
        }
        self.status.set(CPUFlags::CARRY, data & 1 == 1);
        data >>= 1;
        match mode {
            AddressingMode::NoneAddressing => self.register_a = data,
            _ => self.mem_write(addr.unwrap(), data),
        }
        self.update_zero_and_negative_flags(data);
    }

    // Logical inclusive OR
    pub fn ora(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.register_a |= val;
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.bus.tick(1);
        }
    }

    // Pull from stack and into accumulator
    pub fn pla(&mut self) {
        let data = self.stack_pop();
        self.set_register_a(data);
    }

    // Pull from stack and into processor flags
    pub fn plp(&mut self) {
        let data = self.stack_pop();
        // ignore break flag and bit 5
        self.status =
            CPUFlags::from_bits_retain((self.status.bits() & 0b0011_0000) | (data & 0b1100_1111));
    }

    pub fn sbc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
        if page_cross {
            self.bus.tick(1);
        }
    }

    pub fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn inc(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let val = self.mem_read(addr);

        self.mem_write(addr, val.wrapping_add(1));
        self.update_zero_and_negative_flags(val.wrapping_add(1));
    }

    pub fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

        // Rotate left
    pub fn rol(&mut self, mode: &AddressingMode) {0;
        let mut data;
        let mut addr: Option<u16> = None;
        // AddressingNone corresponds to shifting the accumulator left, and addr = None in this case.

        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = Some(self.get_operand_address(mode).0);
                data = self.mem_read(addr.unwrap());
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
                self.mem_write(addr.unwrap(), data);
                self.status.set(CPUFlags::NEGATIVE, data >> 7 == 1);
                self.status.set(CPUFlags::ZERO, data == 0);
            }
        }
    }

    // Rotate right
    pub fn ror(&mut self, mode: &AddressingMode) {
        let mut data;
        let mut addr: Option<u16> = None;
        // AddressingNone corresponds to shifting the accumulator left, and addr = None in this case.

        match mode {
            AddressingMode::NoneAddressing => data = self.register_a,
            _ => {
                addr = Some(self.get_operand_address(mode).0);
                data = self.mem_read(addr.unwrap());
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
                self.mem_write(addr.unwrap(), data);
                self.status.set(CPUFlags::NEGATIVE, data >> 7 == 1);
                self.status.set(CPUFlags::ZERO, data == 0);
            }
        }
    }
}