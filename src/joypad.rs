use crate::interrupts::Interrupts;

/// Button indices into the `buttons` array.
pub mod btn {
    pub const RIGHT: usize = 0;
    pub const LEFT: usize = 1;
    pub const UP: usize = 2;
    pub const DOWN: usize = 3;
    pub const A: usize = 4;
    pub const B: usize = 5;
    pub const SELECT: usize = 6;
    pub const START: usize = 7;
}

/// Joypad register at 0xFF00.
///
/// Bits 5-4: select which group to read (0=selected)
///   bit5 = select action buttons (A, B, Select, Start)
///   bit4 = select d-pad (Right, Left, Up, Down)
/// Bits 3-0: button state (0=pressed)
pub struct Joypad {
    /// true = pressed for each button
    buttons: [bool; 8],
    /// The raw value written to 0xFF00 (upper nibble select bits)
    select: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            buttons: [false; 8],
            select: 0xFF,
        }
    }

    pub fn read(&self) -> u8 {
        let mut val = self.select | 0xCF; // bits 7-6 always 1, bits 3-0 start high
        if self.select & 0x10 == 0 {
            // D-pad selected
            for i in 0..4 {
                if self.buttons[i] {
                    val &= !(1 << i);
                }
            }
        }
        if self.select & 0x20 == 0 {
            // Action buttons selected
            for i in 0..4 {
                if self.buttons[i + 4] {
                    val &= !(1 << i);
                }
            }
        }
        val
    }

    pub fn write(&mut self, val: u8) {
        self.select = val & 0x30;
    }

    pub fn press(&mut self, button: usize, interrupts: &mut Interrupts) {
        self.buttons[button] = true;
        interrupts.request(4); // Joypad interrupt
    }

    pub fn release(&mut self, button: usize) {
        self.buttons[button] = false;
    }
}
