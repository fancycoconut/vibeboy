use super::Cartridge;

/// MBC3 — up to 2MB ROM / 32KB RAM, optional Real-Time Clock.
/// Used by Pokemon Red/Blue (English), Pokemon Gold/Silver, etc.
pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_rtc_enabled: bool,
    rom_bank: u8,
    /// 0x00–0x03 = RAM bank; 0x08–0x0C = RTC register select (stubbed)
    ram_bank: u8,
    latch_written: bool,
    // RTC registers (stubbed as zero — no actual clock)
    rtc: [u8; 5],
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>, ram_size_code: u8) -> Self {
        let ram_bytes = match ram_size_code {
            0x00 => 0,
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            _ => 8 * 1024,
        };
        Self {
            rom,
            ram: vec![0; ram_bytes],
            ram_rtc_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            latch_written: false,
            rtc: [0; 5],
        }
    }
}

impl Cartridge for Mbc3 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom.get(addr as usize).copied().unwrap_or(0xFF),
            0x4000..=0x7FFF => {
                let offset = self.rom_bank as usize * 0x4000 + (addr as usize - 0x4000);
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_rtc_enabled = val & 0x0F == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank = if val & 0x7F == 0 { 1 } else { val & 0x7F };
            }
            0x4000..=0x5FFF => {
                self.ram_bank = val;
            }
            0x6000..=0x7FFF => {
                // Latch clock data: first write 0x00 then 0x01
                if val == 0x00 {
                    self.latch_written = true;
                } else if val == 0x01 && self.latch_written {
                    // Latch RTC — stubbed, do nothing
                    self.latch_written = false;
                }
            }
            _ => {}
        }
    }

    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_rtc_enabled {
            return 0xFF;
        }
        match self.ram_bank {
            0x00..=0x03 => {
                if self.ram.is_empty() {
                    return 0xFF;
                }
                let offset = self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000);
                self.ram.get(offset).copied().unwrap_or(0xFF)
            }
            0x08..=0x0C => self.rtc[(self.ram_bank - 0x08) as usize],
            _ => 0xFF,
        }
    }

    fn ram_write(&mut self, addr: u16, val: u8) {
        if !self.ram_rtc_enabled {
            return;
        }
        match self.ram_bank {
            0x00..=0x03 => {
                if self.ram.is_empty() {
                    return;
                }
                let offset = self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000);
                if let Some(b) = self.ram.get_mut(offset) {
                    *b = val;
                }
            }
            0x08..=0x0C => {
                self.rtc[(self.ram_bank - 0x08) as usize] = val;
            }
            _ => {}
        }
    }
}
