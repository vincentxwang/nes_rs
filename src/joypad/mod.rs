//! Implementation of controller input ($4016)
//! Reference: https://www.nesdev.org/wiki/Standard_controller

pub mod controller;
use macroquad::prelude::*;

bitflags! {
    // https://wiki.nesdev.com/w/index.php/Controller_reading_code
    #[derive(Clone, Copy)]
    pub struct JoypadButton: u8 {
        const RIGHT             = 1 << 7;
        const LEFT              = 1 << 6;
        const DOWN              = 1 << 5;
        const UP                = 1 << 4;
        const START             = 1 << 3;
        const SELECT            = 1 << 2;
        const BUTTON_B          = 1 << 1;
        const BUTTON_A          = 1 << 0;
    }
}

#[derive(Clone, Copy)]
pub struct Joypad {
    strobe: bool,
    button_index: u8,
    pub button_status: JoypadButton,
}
 
impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::from_bits_truncate(0),
        }
    }

    pub fn write(&mut self, data: u8) {
        // Set strobe to last bit of data.
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        // Extracts the button_index-th bit.
        let response = (self.button_status.bits() & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }
        response
    }
}


