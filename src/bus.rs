use crate::cartridge::{self, Cartridge};
use crate::interrupts::Interrupts;
use crate::joypad::Joypad;
use crate::ppu::Ppu;
use crate::timer::Timer;

/// The Game Boy memory bus — routes every read/write to the correct component.
///
/// Memory map (DMG/GBC):
///   0x0000–0x7FFF  Cartridge ROM
///   0x8000–0x9FFF  VRAM (8KB DMG, 16KB GBC banked)
///   0xA000–0xBFFF  Cartridge external RAM
///   0xC000–0xCFFF  Work RAM bank 0
///   0xD000–0xDFFF  Work RAM bank 1–7 (GBC switchable via SVBK)
///   0xE000–0xFDFF  Echo RAM (mirrors 0xC000–0xDDFF)
///   0xFE00–0xFE9F  OAM (sprite attribute table)
///   0xFEA0–0xFEFF  Not usable
///   0xFF00–0xFF7F  I/O registers
///   0xFF80–0xFFFE  High RAM (HRAM)
///   0xFFFF         Interrupt Enable register
pub struct Bus {
    cartridge: Box<dyn Cartridge>,
    wram: [u8; 0x8000], // 32KB — banks 0-7 (GBC), bank 0+1 only for DMG
    hram: [u8; 0x7F],
    pub oam: [u8; 0xA0],
    /// Serial data register (0xFF01) — printed to stdout for blargg tests
    serial_data: u8,
    /// Serial control (0xFF02)
    serial_ctrl: u8,
    pub ppu: Ppu,
    pub timer: Timer,
    pub joypad: Joypad,
    pub interrupts: Interrupts,
    /// GBC work RAM bank (SVBK 0xFF70), 1–7
    wram_bank: u8,
}

impl Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            cartridge: cartridge::load(rom),
            wram: [0; 0x8000],
            hram: [0; 0x7F],
            oam: [0; 0xA0],
            serial_data: 0,
            serial_ctrl: 0,
            ppu: Ppu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            interrupts: Interrupts::new(),
            wram_bank: 1,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cartridge.read(addr),
            0x8000..=0x9FFF => self.ppu.vram_read(addr),
            0xA000..=0xBFFF => self.cartridge.ram_read(addr),
            0xC000..=0xCFFF => self.wram[(addr - 0xC000) as usize],
            0xD000..=0xDFFF => {
                let bank = self.wram_bank.max(1) as usize;
                self.wram[bank * 0x1000 + (addr - 0xD000) as usize]
            }
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFEA0..=0xFEFF => 0xFF, // not usable
            0xFF00 => self.joypad.read(),
            0xFF01 => self.serial_data,
            0xFF02 => self.serial_ctrl,
            0xFF03..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.interrupts.flags,
            0xFF10..=0xFF3F => 0xFF, // APU — stub
            0xFF40..=0xFF4B => self.ppu.reg_read(addr),
            0xFF4F => self.ppu.vram_bank_read(),
            0xFF51..=0xFF55 => 0xFF, // GBC DMA — stub
            0xFF68..=0xFF6B => self.ppu.cgb_palette_read(addr),
            0xFF70 => self.wram_bank,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.interrupts.enable,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.cartridge.write(addr, val),
            0x8000..=0x9FFF => self.ppu.vram_write(addr, val),
            0xA000..=0xBFFF => self.cartridge.ram_write(addr, val),
            0xC000..=0xCFFF => self.wram[(addr - 0xC000) as usize] = val,
            0xD000..=0xDFFF => {
                let bank = self.wram_bank.max(1) as usize;
                self.wram[bank * 0x1000 + (addr - 0xD000) as usize] = val;
            }
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = val,
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = val,
            0xFEA0..=0xFEFF => {} // not usable
            0xFF00 => self.joypad.write(val),
            0xFF01 => self.serial_data = val,
            0xFF02 => {
                self.serial_ctrl = val;
                // Blargg test ROMs signal transfer start by writing 0x81
                if val == 0x81 {
                    print!("{}", self.serial_data as char);
                }
            }
            0xFF03..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupts.flags = val,
            0xFF10..=0xFF3F => {} // APU — stub
            0xFF46 => self.oam_dma(val),
            0xFF40..=0xFF4B => self.ppu.reg_write(addr, val),
            0xFF4F => self.ppu.vram_bank_write(val),
            0xFF51..=0xFF55 => {} // GBC DMA — stub
            0xFF68..=0xFF6B => self.ppu.cgb_palette_write(addr, val),
            0xFF70 => self.wram_bank = if val & 0x07 == 0 { 1 } else { val & 0x07 },
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,
            0xFFFF => self.interrupts.enable = val,
            _ => {}
        }
    }

    /// OAM DMA transfer: copy 160 bytes from `base * 0x100` into OAM.
    fn oam_dma(&mut self, base: u8) {
        let src = (base as u16) << 8;
        for i in 0..0xA0u16 {
            self.oam[i as usize] = self.read(src + i);
        }
    }

    /// Return a snapshot copy of OAM (needed to work around borrow checker
    /// when passing OAM to the PPU while Bus is also borrowed for reads).
    pub fn oam_snapshot(&self) -> [u8; 0xA0] {
        self.oam
    }

    /// Convenience 16-bit read (little-endian).
    pub fn read16(&self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Convenience 16-bit write (little-endian).
    pub fn write16(&mut self, addr: u16, val: u16) {
        self.write(addr, val as u8);
        self.write(addr.wrapping_add(1), (val >> 8) as u8);
    }
}
