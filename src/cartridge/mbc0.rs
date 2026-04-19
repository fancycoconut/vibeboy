use super::Cartridge;

/// MBC0 — ROM only, no banking. Up to 32KB.
pub struct Mbc0 {
    rom: Vec<u8>,
}

impl Mbc0 {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl Cartridge for Mbc0 {
    fn read(&self, addr: u16) -> u8 {
        self.rom.get(addr as usize).copied().unwrap_or(0xFF)
    }

    fn write(&mut self, _addr: u16, _val: u8) {
        // No writable registers on MBC0
    }

    fn ram_read(&self, _addr: u16) -> u8 {
        0xFF
    }

    fn ram_write(&mut self, _addr: u16, _val: u8) {}
}
