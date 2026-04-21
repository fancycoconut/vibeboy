use super::Cartridge;

/// MBC5 — up to 8MB ROM / 128KB RAM with 9-bit bank register.
/// Used by Pokemon Yellow, Gold/Silver, and many GBC games.
pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u16, // 9-bit (0–511)
    ram_bank: u8,  // 4-bit (0–15)
}

impl Mbc5 {
    pub fn new(rom: Vec<u8>, ram_size_code: u8) -> Self {
        let ram_bytes = match ram_size_code {
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            _ => 0,
        };
        Self {
            rom,
            ram: vec![0; ram_bytes],
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
        }
    }
}

impl Cartridge for Mbc5 {
    fn read(&self, addr: u16) -> u8 {
        let offset = match addr {
            0x0000..=0x3FFF => addr as usize,
            0x4000..=0x7FFF => self.rom_bank as usize * 0x4000 + (addr as usize - 0x4000),
            _ => return 0xFF,
        };
        self.rom.get(offset).copied().unwrap_or(0xFF)
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = val & 0x0F == 0x0A;
            }
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x100) | val as u16;
            }
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x0FF) | ((val as u16 & 0x01) << 8);
            }
            0x4000..=0x5FFF => {
                self.ram_bank = val & 0x0F;
            }
            _ => {}
        }
    }

    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            return 0xFF;
        }
        let offset = self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000);
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn ram_write(&mut self, addr: u16, val: u8) {
        if !self.ram_enabled || self.ram.is_empty() {
            return;
        }
        let offset = self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000);
        if let Some(b) = self.ram.get_mut(offset) {
            *b = val;
        }
    }
}
