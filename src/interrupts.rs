/// Interrupt flags (IF) at 0xFF0F and Interrupt enable (IE) at 0xFFFF.
///
/// Bit positions: 0=VBlank 1=LCD 2=Timer 3=Serial 4=Joypad
pub struct Interrupts {
    /// 0xFF0F — Interrupt Flag: bits set when interrupt is requested
    pub flags: u8,
    /// 0xFFFF — Interrupt Enable: bits set to allow interrupt to fire
    pub enable: u8,
}

impl Interrupts {
    pub fn new() -> Self {
        Self { flags: 0, enable: 0 }
    }

    pub fn request(&mut self, bit: u8) {
        self.flags |= 1 << bit;
    }

    /// Returns the bit index of the highest-priority pending+enabled interrupt,
    /// or None if nothing is pending.
    pub fn pending(&self) -> Option<u8> {
        let active = self.flags & self.enable & 0x1F;
        if active == 0 {
            None
        } else {
            Some(active.trailing_zeros() as u8)
        }
    }

    /// Acknowledge an interrupt by clearing its flag bit.
    pub fn ack(&mut self, bit: u8) {
        self.flags &= !(1 << bit);
    }
}
