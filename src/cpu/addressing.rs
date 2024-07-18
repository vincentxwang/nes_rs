use crate::cpu::CPU;
use crate::cpu::Mem;

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

impl CPU {
    // returns (address, page_crossed?)
    // wrapper for get_absolute_address
    pub fn get_operand_address(&mut self, mode: &AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Immediate => (self.program_counter, false),
            _ => self.get_absolute_address(mode, self.program_counter),
        }
    }

    // Returns whether or not a page was crossed when adding something to a that results in b.
    // Checks if the high byte is different.
    pub fn page_cross(a: u16, b: u16) -> bool {
       (a & 0xff00) != (b & 0xff00)
    }

    // returns (address, page_crossed)
    pub fn get_absolute_address(&mut self, mode: &AddressingMode, addr: u16) -> (u16, bool) {
        match mode {
            AddressingMode::ZeroPage => (self.mem_read(addr) as u16, false),
            AddressingMode::Absolute => (self.mem_read_u16(addr), false),
            AddressingMode::ZeroPage_X => (self.mem_read(addr).wrapping_add(self.register_x) as u16, false),
            AddressingMode::ZeroPage_Y => (self.mem_read(addr).wrapping_add(self.register_y) as u16, false),
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(addr);
                let res = base.wrapping_add(self.register_x as u16);
                (res, CPU::page_cross(base, res))
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(addr);
                let res = base.wrapping_add(self.register_y as u16);
                (res, CPU::page_cross(base, res))
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(addr);
    
                let ptr: u8 = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                ((hi as u16) << 8 | (lo as u16), false)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(addr);
    
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);

                (deref, CPU::page_cross(deref, deref_base))
            }
            _ => {
                // TODO: refactor the 0 as a None
                (0, false)
            }
        }
    }
}   
