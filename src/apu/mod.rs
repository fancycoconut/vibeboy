/// Audio Processing Unit — stub. Full implementation is a future phase.
pub struct Apu;

impl Apu {
    pub fn new() -> Self {
        Self
    }

    pub fn step(&mut self, _cycles: u8) {}

    pub fn read(&self, _addr: u16) -> u8 {
        0xFF
    }

    pub fn write(&mut self, _addr: u16, _val: u8) {}
}
