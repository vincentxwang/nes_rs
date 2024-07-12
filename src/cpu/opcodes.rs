//! Hardcoded opcodes
//! Reference (official): https://www.nesdev.org/obelisk-6502-guide/reference.html
//! Reference (unofficial): https://www.oxyron.de/html/opcodes02.html

use crate::cpu::AddressingMode;
use crate::cpu::Operation;
use std::collections::HashMap;
pub struct OpCode {
    pub code: u8,
    pub op: Operation,
    pub bytes: u8,
    pub cycles: usize,
    pub addressing_mode: AddressingMode,
}

impl OpCode {
    pub fn new(
        code: u8,
        op: Operation,
        bytes: u8,
        cycles: usize,
        addressing_mode: AddressingMode,
    ) -> Self {
        OpCode {
            code,
            op,
            bytes,
            cycles,
            addressing_mode,
        }
    }
}

lazy_static! {
    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        OpCode::new(0x69, Operation::ADC, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, Operation::ADC, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, Operation::ADC, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6d, Operation::ADC, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7d, Operation::ADC, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x79, Operation::ADC, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x61, Operation::ADC, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, Operation::ADC, 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0x29, Operation::AND, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, Operation::AND, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, Operation::AND, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x2d, Operation::AND, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3d, Operation::AND, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x39, Operation::AND, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x21, Operation::AND, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x31, Operation::AND, 2, 5/*+1 if page crossed */, AddressingMode::Indirect_Y),

        OpCode::new(0x0a, Operation::ASL, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x06, Operation::ASL, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, Operation::ASL, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0e, Operation::ASL, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1e, Operation::ASL, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0x90, Operation::BCC, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0xb0, Operation::BCS, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0xf0, Operation::BEQ, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0x24, Operation::BIT, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2c, Operation::BIT, 3, 4, AddressingMode::Absolute),

        OpCode::new(0x30, Operation::BMI, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0xd0, Operation::BNE, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0x10, Operation::BPL, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0x00, Operation::BRK, 1, 7, AddressingMode::NoneAddressing),

        OpCode::new(0x50, Operation::BVC, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0x70, Operation::BVS, 2, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::NoneAddressing),

        OpCode::new(0x18, Operation::CLC, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xD8, Operation::CLD, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x58, Operation::CLI, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xb8, Operation::CLV, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xc9, Operation::CMP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc5, Operation::CMP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xd5, Operation::CMP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xcd, Operation::CMP, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xdd, Operation::CMP, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xd9, Operation::CMP, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xc1, Operation::CMP, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xd1, Operation::CMP, 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0xe0, Operation::CPX, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe4, Operation::CPX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xec, Operation::CPX, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xc0, Operation::CPY, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc4, Operation::CPY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xcc, Operation::CPY, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xc6, Operation::DEC, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xd6, Operation::DEC, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xce, Operation::DEC, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xde, Operation::DEC, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0xca, Operation::DEX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x88, Operation::DEY, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x49, Operation::EOR, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, Operation::EOR, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, Operation::EOR, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x4d, Operation::EOR, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5d, Operation::EOR, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0x59, Operation::EOR, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0x41, Operation::EOR, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x51, Operation::EOR, 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0xe6, Operation::INC, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xf6, Operation::INC, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xee, Operation::INC, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xfe, Operation::INC, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0xe8, Operation::INX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xc8, Operation::INY, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x4c, Operation::JMP, 3, 3, AddressingMode::Absolute),
        OpCode::new(0x6c, Operation::JMP, 3, 5, AddressingMode::Indirect), // there is a bug here that is NOT implemented

        OpCode::new(0x20, Operation::JSR, 3, 6, AddressingMode::NoneAddressing),

        OpCode::new(0xa9, Operation::LDA, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, Operation::LDA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, Operation::LDA, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xad, Operation::LDA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbd, Operation::LDA, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xb9, Operation::LDA, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xa1, Operation::LDA, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xb1, Operation::LDA, 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0xa2, Operation::LDX, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa6, Operation::LDX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb6, Operation::LDX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xae, Operation::LDX, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbe, Operation::LDX, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),

        OpCode::new(0xa0, Operation::LDY, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa4, Operation::LDY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb4, Operation::LDY, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xac, Operation::LDY, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbc, Operation::LDY, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),

        OpCode::new(0x4a, Operation::LSR, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x46, Operation::LSR, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, Operation::LSR, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4e, Operation::LSR, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5e, Operation::LSR, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0xea, Operation::NOP, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x09, Operation::ORA, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, Operation::ORA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, Operation::ORA, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0d, Operation::ORA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1d, Operation::ORA, 3, 4/*+1 if page crossed */, AddressingMode::Absolute_X),
        OpCode::new(0x19, Operation::ORA, 3, 4/*+1 if page crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0x01, Operation::ORA, 2, 6/*+1 if page crossed */, AddressingMode::Indirect_X),
        OpCode::new(0x11, Operation::ORA, 2, 5/*+1 if page crossed */, AddressingMode::Indirect_Y),

        OpCode::new(0x48, Operation::PHA, 1, 3, AddressingMode::NoneAddressing),

        OpCode::new(0x08, Operation::PHP, 1, 3, AddressingMode::NoneAddressing),

        OpCode::new(0x68, Operation::PLA, 1, 4, AddressingMode::NoneAddressing),

        OpCode::new(0x28, Operation::PLP, 1, 4, AddressingMode::NoneAddressing),

        OpCode::new(0x2a, Operation::ROL, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x26, Operation::ROL, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, Operation::ROL, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2e, Operation::ROL, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3e, Operation::ROL, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0x6a, Operation::ROR, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x66, Operation::ROR, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, Operation::ROR, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6e, Operation::ROR, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7e, Operation::ROR, 3, 7, AddressingMode::Absolute_X),

        OpCode::new(0x40, Operation::RTI, 1, 6, AddressingMode::NoneAddressing),

        OpCode::new(0x60, Operation::RTS, 1, 6, AddressingMode::NoneAddressing),

        OpCode::new(0xe9, Operation::SBC, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe5, Operation::SBC, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xf5, Operation::SBC, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xed, Operation::SBC, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xfd, Operation::SBC, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xf9, Operation::SBC, 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xe1, Operation::SBC, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xf1, Operation::SBC, 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0x38, Operation::SEC, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x78, Operation::SEI, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xf8, Operation::SED, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x85, Operation::STA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, Operation::STA, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8d, Operation::STA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, Operation::STA, 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, Operation::STA, 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, Operation::STA, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, Operation::STA, 2, 6, AddressingMode::Indirect_Y),

        OpCode::new(0x86, Operation::STX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, Operation::STX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8e, Operation::STX, 3, 4, AddressingMode::Absolute),

        OpCode::new(0x84, Operation::STY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, Operation::STY, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8c, Operation::STY, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xaa, Operation::TAX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xa8, Operation::TAY, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xba, Operation::TSX, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x9a, Operation::TXS, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x8a, Operation::TXA, 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0x98, Operation::TYA, 1, 2, AddressingMode::NoneAddressing),

        // Unofficial opcodes

        OpCode::new(0x1a, Operation::NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x3a, Operation::NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x5a, Operation::NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x7a, Operation::NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xda, Operation::NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xfa, Operation::NOP, 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0x80, Operation::NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x82, Operation::NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x89, Operation::NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc2, Operation::NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe2, Operation::NOP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x04, Operation::NOP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x44, Operation::NOP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x64, Operation::NOP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x14, Operation::NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x34, Operation::NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x54, Operation::NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x74, Operation::NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xd4, Operation::NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xf4, Operation::NOP, 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x0c, Operation::NOP, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1c, Operation::NOP, 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0x3c, Operation::NOP, 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0x5c, Operation::NOP, 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0x7c, Operation::NOP, 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0xdc, Operation::NOP, 3, 4 /*or 5*/, AddressingMode::Absolute_X),
        OpCode::new(0xfc, Operation::NOP, 3, 4 /*or 5*/, AddressingMode::Absolute_X),

        OpCode::new(0xa3, Operation::LAX, 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xab, Operation::LAX, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa7, Operation::LAX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb7, Operation::LAX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0xb3, Operation::LAX, 2, 5 /* or 6 boundary cross*/, AddressingMode::Indirect_Y),
        OpCode::new(0xaf, Operation::LAX, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbf, Operation::LAX, 3, 4 /* or 5 */, AddressingMode::Absolute_Y),

        OpCode::new(0x87, Operation::SAX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x97, Operation::SAX, 2, 4, AddressingMode::ZeroPage_Y),
        OpCode::new(0x8f, Operation::SAX, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x83, Operation::SAX, 2, 6, AddressingMode::Indirect_X),

        OpCode::new(0xeb, Operation::SBC, 2,2, AddressingMode::Immediate),

        OpCode::new(0xc7, Operation::DCP, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xd7, Operation::DCP, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0xcf, Operation::DCP, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xdf, Operation::DCP, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0xdb, Operation::DCP, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0xd3, Operation::DCP, 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0xc3, Operation::DCP, 2, 8, AddressingMode::Indirect_X),

        OpCode::new(0xe7, Operation::ISB, 2,5, AddressingMode::ZeroPage),
        OpCode::new(0xf7, Operation::ISB, 2,6, AddressingMode::ZeroPage_X),
        OpCode::new(0xef, Operation::ISB, 3,6, AddressingMode::Absolute),
        OpCode::new(0xff, Operation::ISB, 3,7, AddressingMode::Absolute_X),
        OpCode::new(0xfb, Operation::ISB, 3,7, AddressingMode::Absolute_Y),
        OpCode::new(0xe3, Operation::ISB, 2,8, AddressingMode::Indirect_X),
        OpCode::new(0xf3, Operation::ISB, 2,8, AddressingMode::Indirect_Y),


        OpCode::new(0x07, Operation::SLO, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x17, Operation::SLO, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x0f, Operation::SLO, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1f, Operation::SLO, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x1b, Operation::SLO, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x03, Operation::SLO, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x13, Operation::SLO, 2, 8, AddressingMode::Indirect_Y),


        OpCode::new(0x27, Operation::RLA, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x37, Operation::RLA, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x2f, Operation::RLA, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3f, Operation::RLA, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x3b, Operation::RLA, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x33, Operation::RLA, 2, 8, AddressingMode::Indirect_Y),
        OpCode::new(0x23, Operation::RLA, 2, 8, AddressingMode::Indirect_X),


        OpCode::new(0x47, Operation::SRE, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x57, Operation::SRE, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x4f, Operation::SRE, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5f, Operation::SRE, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x5b, Operation::SRE, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x43, Operation::SRE, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x53, Operation::SRE, 2, 8, AddressingMode::Indirect_Y),

        OpCode::new(0x67, Operation::RRA, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x77, Operation::RRA, 2, 6, AddressingMode::ZeroPage_X),
        OpCode::new(0x6f, Operation::RRA, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7f, Operation::RRA, 3, 7, AddressingMode::Absolute_X),
        OpCode::new(0x7b, Operation::RRA, 3, 7, AddressingMode::Absolute_Y),
        OpCode::new(0x63, Operation::RRA, 2, 8, AddressingMode::Indirect_X),
        OpCode::new(0x73, Operation::RRA, 2, 8, AddressingMode::Indirect_Y),
];


    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for cpuop in &*CPU_OPS_CODES {
            map.insert(cpuop.code, cpuop);
        }
        map
    };

    // For tracing purposes
    pub static ref UNOFFICIAL_OPCODES: Vec<u8> = vec![
        // NOP
        0x1a, 0x3a, 0x5a, 0x7a, 0xda, 0xfa,
        0x04, 0x44, 0x64,
        0x14, 0x34, 0x54, 0x74, 0xd4, 0xf4,
        0x0c,
        0x1c, 0x3c, 0x5c, 0x7c, 0xdc, 0xfc,
        0x80,
        // LAX
        0xa3, 0xab, 0xa7, 0xb7, 0xb3, 0xaf, 0xbf,
        // SBC
        0xeb,
        // DCP
        0xc7, 0xd7, 0xcf, 0xdf, 0xdb, 0xd3, 0xc3,
        // ISB
        0xe7, 0xf7, 0xef, 0xff, 0xfb, 0xe3, 0xf3,
        // SLO
        0x07, 0x17, 0x0f, 0x1f, 0x1b, 0x03, 0x13,
        // RLA
        0x27, 0x37, 0x2f, 0x3f, 0x3b, 0x33, 0x23,
        // SRE
        0x47, 0x57, 0x4f, 0x5f, 0x5b, 0x43, 0x53,
        // RRA
        0x67, 0x77, 0x6f, 0x7f, 0x7b, 0x63, 0x73,
        // SAX
        0x83, 0x87, 0x8f, 0x97
    ];
}


