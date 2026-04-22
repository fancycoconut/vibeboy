use super::Cartridge;

/// MBC1 — up to 2MB ROM / 32KB RAM with bank switching.
/// Used by Pokemon Red/Blue, Tetris, and many others.
pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    banking_mode: u8, // 0=ROM banking, 1=RAM banking
}

impl Mbc1 {
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
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: 0,
        }
    }

    fn rom_offset(&self, addr: u16) -> usize {
        let bank = if addr < 0x4000 {
            if self.banking_mode == 1 {
                (self.ram_bank as usize) << 5
            } else {
                0
            }
        } else {
            let base = ((self.ram_bank as usize) << 5) | (self.rom_bank as usize & 0x1F);
            base
        };
        let bank_offset = if addr < 0x4000 { addr as usize } else { (addr as usize) - 0x4000 };
        bank * 0x4000 + bank_offset
    }
}

impl Cartridge for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        let offset = self.rom_offset(addr);
        self.rom.get(offset).copied().unwrap_or(0xFF)
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = val & 0x0F == 0x0A;
            }
            0x2000..=0x3FFF => {
                let bank = val & 0x1F;
                // Bank 0 translates to bank 1 (hardware quirk)
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            0x4000..=0x5FFF => {
                self.ram_bank = val & 0x03;
            }
            0x6000..=0x7FFF => {
                self.banking_mode = val & 0x01;
            }
            _ => {}
        }
    }

    fn ram_read(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            return 0xFF;
        }
        let offset = if self.banking_mode == 1 {
            self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000)
        } else {
            addr as usize - 0xA000
        };
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn ram_write(&mut self, addr: u16, val: u8) {
        if !self.ram_enabled || self.ram.is_empty() {
            return;
        }
        let offset = if self.banking_mode == 1 {
            self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000)
        } else {
            addr as usize - 0xA000
        };
        if let Some(b) = self.ram.get_mut(offset) {
            *b = val;
        }
    }

    fn save_ram(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn load_ram(&mut self, data: &[u8]) {
        let len = self.ram.len().min(data.len());
        self.ram[..len].copy_from_slice(&data[..len]);
    }
}
