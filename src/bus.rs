use crate::apu::Apu;
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
    pub cartridge: Box<dyn Cartridge>,
    wram: [u8; 0x8000], // 32KB — banks 0-7 (GBC), bank 0+1 only for DMG
    hram: [u8; 0x7F],
    pub oam: [u8; 0xA0],
    /// Serial data register (0xFF01) — printed to stdout for blargg tests
    serial_data: u8,
    /// Serial control (0xFF02)
    serial_ctrl: u8,
    /// Accumulates every byte transferred via serial (0xFF02 write 0x81).
    /// Used by integration tests to capture blargg output without stdout.
    pub serial_buf: Vec<u8>,
    pub apu: Apu,
    pub ppu: Ppu,
    pub timer: Timer,
    pub joypad: Joypad,
    pub interrupts: Interrupts,
    /// GBC work RAM bank (SVBK 0xFF70), 1–7
    wram_bank: u8,

    // -----------------------------------------------------------------------
    // GBC HDMA state (0xFF51–0xFF55)
    // -----------------------------------------------------------------------
    /// HDMA1–HDMA4: source/destination address registers as written by the game.
    hdma1: u8, // source high byte
    hdma2: u8, // source low byte  (bits 3-0 ignored → aligned to 16)
    hdma3: u8, // dest high byte   (bits 7-5 ignored → VRAM 0x8000-0x9FFF)
    hdma4: u8, // dest low byte    (bits 3-0 ignored → aligned to 16)
    /// Running source pointer (updated as blocks are transferred).
    hdma_src: u16,
    /// Running destination pointer (updated as blocks are transferred).
    hdma_dst: u16,
    /// Remaining 16-byte blocks – 1.  0x7F means "idle / complete" so that
    /// reading HDMA5 returns 0xFF when nothing is active.
    hdma_remaining: u8,
    /// True while an HBlank DMA is in progress (as opposed to one-shot GDMA).
    hdma_active: bool,
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
            serial_buf: Vec::new(),
            apu: Apu::new(),
            ppu: Ppu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            interrupts: Interrupts::new(),
            wram_bank: 1,
            hdma1: 0xFF,
            hdma2: 0xFF,
            hdma3: 0xFF,
            hdma4: 0xFF,
            hdma_src: 0,
            hdma_dst: 0x8000,
            hdma_remaining: 0x7F,
            hdma_active: false,
        }
    }

    /// Advance all cycle-sensitive components by N machine cycles (N×4 T-cycles).
    /// Single place to add timer/PPU T-cycle accuracy in the future.
    pub fn tick_m_cycle(&mut self, cycles: u8) {
        self.apu.step(cycles);
        // future: self.timer.tick_t(cycles as u32 * 4);
        // future: self.ppu.tick_t(cycles as u32 * 4);
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
            0xFF04..=0xFF07 => self.timer.read(addr),
            0xFF0F => self.interrupts.flags,
            0xFF10..=0xFF3F => self.apu.read(addr),
            0xFF40..=0xFF4B => self.ppu.reg_read(addr),
            0xFF4F => self.ppu.vram_bank_read(),
            0xFF51 => self.hdma1,
            0xFF52 => self.hdma2,
            0xFF53 => self.hdma3,
            0xFF54 => self.hdma4,
            // HDMA5: bit 7 = 0 while HBlank DMA is active, 1 when idle/done.
            // Bits 6-0 = remaining blocks – 1 (0x7F when idle → reads 0xFF).
            0xFF55 => {
                if self.hdma_active {
                    self.hdma_remaining & 0x7F
                } else {
                    0x80 | self.hdma_remaining
                }
            }
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
                    self.serial_buf.push(self.serial_data);
                    #[cfg(not(test))]
                    print!("{}", self.serial_data as char);
                }
            }
            0xFF04..=0xFF07 => self.timer.write(addr, val),
            0xFF0F => self.interrupts.flags = val,
            0xFF10..=0xFF3F => self.apu.write(addr, val),
            0xFF46 => self.oam_dma(val),
            0xFF40..=0xFF4B => self.ppu.reg_write(addr, val),
            0xFF4F => self.ppu.vram_bank_write(val),
            0xFF51 => self.hdma1 = val,
            0xFF52 => self.hdma2 = val & 0xF0, // lower nibble ignored (16-byte alignment)
            0xFF53 => self.hdma3 = val & 0x1F, // destination must be in VRAM 0x8000-0x9FFF
            0xFF54 => self.hdma4 = val & 0xF0, // lower nibble ignored (16-byte alignment)
            0xFF55 => self.write_hdma5(val),
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

    /// Handle a write to HDMA5 (0xFF55): start GDMA, start HBlank DMA, or
    /// cancel an in-progress HBlank DMA.
    fn write_hdma5(&mut self, val: u8) {
        if self.hdma_active && val & 0x80 == 0 {
            // Cancel active HBlank DMA.  HDMA5 bit 7 becomes 1 (inactive) on
            // next read; remaining block count is preserved.
            self.hdma_active = false;
            return;
        }

        // Compute source and destination from the four address registers.
        // Source: HDMA1:HDMA2  (HDMA2 lower nibble already zeroed on write)
        // Dest:   0x8000 | (HDMA3 & 0x1F) << 8 | HDMA4  (both aligned to 16)
        self.hdma_src = (self.hdma1 as u16) << 8 | self.hdma2 as u16;
        self.hdma_dst = 0x8000 | (self.hdma3 as u16) << 8 | self.hdma4 as u16;
        self.hdma_remaining = val & 0x7F;

        if val & 0x80 == 0 {
            // General Purpose DMA — transfer all blocks immediately, then idle.
            self.gdma_transfer();
        } else {
            // HBlank DMA — transfer one 16-byte block per HBlank.
            self.hdma_active = true;
        }
    }

    /// General Purpose DMA: copy all blocks in one shot (CPU is frozen on HW).
    fn gdma_transfer(&mut self) {
        loop {
            self.hdma_copy_block();
            if self.hdma_remaining == 0 {
                break;
            }
            self.hdma_remaining -= 1;
        }
        self.hdma_remaining = 0x7F; // signal idle/complete
    }

    /// Called by `GameBoy::step` each time the PPU enters HBlank while an
    /// HBlank DMA is active.  Transfers exactly one 16-byte block.
    pub fn hdma_hblank_step(&mut self) {
        if !self.hdma_active {
            return;
        }
        self.hdma_copy_block();
        if self.hdma_remaining == 0 {
            self.hdma_active = false;
            self.hdma_remaining = 0x7F; // signal complete
        } else {
            self.hdma_remaining -= 1;
        }
    }

    /// Copy one 16-byte block from `hdma_src` → VRAM at `hdma_dst`, advancing
    /// both pointers.
    #[inline(always)]
    fn hdma_copy_block(&mut self) {
        for _ in 0..16u8 {
            let src = self.hdma_src;
            let dst = self.hdma_dst;
            let val = self.read(src);
            self.ppu.vram_write_dma(dst, val);
            self.hdma_src = self.hdma_src.wrapping_add(1);
            self.hdma_dst = self.hdma_dst.wrapping_add(1);
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
