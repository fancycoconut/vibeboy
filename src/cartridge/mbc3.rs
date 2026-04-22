use std::time::{SystemTime, UNIX_EPOCH};

use super::Cartridge;

// ---------------------------------------------------------------------------
// Real-Time Clock
// ---------------------------------------------------------------------------

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// MBC3 Real-Time Clock.
///
/// Time is derived from a Unix-epoch `origin`: elapsed = now − origin when
/// running, or elapsed = halt_ts − origin when halted. Writing a register
/// adjusts `origin` so the new value takes effect immediately without
/// disturbing the other components.
///
/// Registers (selected via ram_bank 0x08–0x0C):
///   0x08 RTC S  — seconds   (0–59)
///   0x09 RTC M  — minutes   (0–59)
///   0x0A RTC H  — hours     (0–23)
///   0x0B RTC DL — day counter low byte
///   0x0C RTC DH — bit 0: day counter high bit | bit 6: halt | bit 7: carry
///
/// Games never read live registers; they read a latched snapshot captured by
/// writing 0x00 then 0x01 to 0x6000–0x7FFF.
struct Rtc {
    /// Unix timestamp such that elapsed_secs() == now − origin when running.
    origin: u64,
    /// Clock is halted when true; elapsed is frozen at halt_ts − origin.
    halted: bool,
    /// Unix timestamp at the moment HALT was asserted.
    halt_ts: u64,
    /// Day-counter carry: set when the 9-bit counter wraps at 512; cleared by
    /// writing 0 to DH bit 7.
    carry: bool,
    /// Latched snapshot — the values games actually read.
    latched: [u8; 5],
    /// True after 0x00 is written to 0x6000–0x7FFF (first step of latch sequence).
    latch_pending: bool,
}

impl Rtc {
    fn new() -> Self {
        Self {
            origin: unix_now(),
            halted: false,
            halt_ts: 0,
            carry: false,
            latched: [0; 5],
            latch_pending: false,
        }
    }

    /// Seconds elapsed since the RTC origin, frozen while halted.
    fn elapsed_secs(&self) -> u64 {
        if self.halted {
            self.halt_ts.saturating_sub(self.origin)
        } else {
            unix_now().saturating_sub(self.origin)
        }
    }

    /// Capture the current live values into the latch registers.
    fn latch(&mut self) {
        let e = self.elapsed_secs();
        let s = (e % 60) as u8;
        let m = ((e / 60) % 60) as u8;
        let h = ((e / 3600) % 24) as u8;
        let days = e / 86400;
        if days >= 512 {
            self.carry = true;
        }
        let days = days % 512;
        let dl = (days & 0xFF) as u8;
        let dh = ((days >> 8) as u8 & 0x01)
            | if self.carry  { 0x80 } else { 0 }
            | if self.halted { 0x40 } else { 0 };
        self.latched = [s, m, h, dl, dh];
    }

    /// Read latched register. `reg` is 0x08–0x0C.
    fn read(&self, reg: u8) -> u8 {
        self.latched[(reg - 0x08) as usize]
    }

    /// Write to a live RTC register. `reg` is 0x08–0x0C.
    fn write(&mut self, reg: u8, val: u8) {
        match reg {
            // Seconds: keep everything ≥ 1 minute, replace seconds.
            0x08 => self.set_component(val as u64 % 60, 1, 60),
            // Minutes: keep everything ≥ 1 hour, replace minutes, keep seconds.
            0x09 => self.set_component(val as u64 % 60, 60, 3600),
            // Hours: keep days, replace hours, keep minutes+seconds.
            0x0A => self.set_component(val as u64 % 24, 3600, 86400),
            // DL: replace the low 8 bits of the day counter.
            0x0B => self.set_day_lo(val),
            // DH: high day bit, halt flag, carry clear.
            0x0C => self.write_dh(val),
            _ => {}
        }
    }

    /// Update one time component without disturbing larger or smaller units.
    ///
    /// `new_val`       — the value to set (already range-clamped by the caller)
    /// `unit_secs`     — seconds per one unit (1 for seconds, 60 for minutes, …)
    /// `next_unit_secs`— seconds per one step of the next larger unit
    fn set_component(&mut self, new_val: u64, unit_secs: u64, next_unit_secs: u64) {
        let e = self.elapsed_secs();
        let new_e = (e / next_unit_secs) * next_unit_secs
            + new_val * unit_secs
            + (e % unit_secs);
        self.adjust_origin(new_e);
    }

    fn set_day_lo(&mut self, dl: u8) {
        let e = self.elapsed_secs();
        let days_hi = (e / 86400) & 0x100; // preserve the high bit
        let new_days = days_hi | dl as u64;
        let new_e = new_days * 86400 + (e % 86400);
        self.adjust_origin(new_e);
    }

    fn write_dh(&mut self, val: u8) {
        // --- Update day-hi bit ---
        let e = self.elapsed_secs();
        let new_days = (e / 86400 & 0xFF) | ((val as u64 & 0x01) << 8);
        let new_e = new_days * 86400 + (e % 86400);
        // adjust_origin reads self.halted, which is still the old value here — intentional.
        self.adjust_origin(new_e);

        // --- Carry: writing 0 to bit 7 clears it ---
        if val & 0x80 == 0 {
            self.carry = false;
        }

        // --- Halt transitions ---
        let was_halted = self.halted;
        let now_halted = val & 0x40 != 0;
        if !was_halted && now_halted {
            // Halting: record the freeze point.
            self.halt_ts = unix_now();
        } else if was_halted && !now_halted {
            // Resuming: shift origin forward so elapsed continues from the frozen point.
            // At this moment self.halted is still true, so:
            //   frozen = halt_ts - origin = new_e  (from adjust_origin above)
            let frozen = self.halt_ts.saturating_sub(self.origin);
            self.origin = unix_now().saturating_sub(frozen);
        }
        self.halted = now_halted;
    }

    /// Set origin so that elapsed_secs() == new_elapsed.
    /// Uses halt_ts as "now" when halted to keep the freeze consistent.
    fn adjust_origin(&mut self, new_elapsed: u64) {
        if self.halted {
            self.origin = self.halt_ts.saturating_sub(new_elapsed);
        } else {
            self.origin = unix_now().saturating_sub(new_elapsed);
        }
    }
}

// ---------------------------------------------------------------------------
// MBC3
// ---------------------------------------------------------------------------

/// MBC3 — up to 2MB ROM / 32KB RAM, optional Real-Time Clock.
/// Used by Pokemon Red/Blue (English), Pokemon Gold/Silver, etc.
pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_rtc_enabled: bool,
    rom_bank: u8,
    /// 0x00–0x03 = RAM bank; 0x08–0x0C = RTC register select
    ram_bank: u8,
    rtc: Rtc,
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>, ram_size_code: u8) -> Self {
        let ram_bytes = match ram_size_code {
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            _ => 0,
        };
        Self {
            rom,
            ram: vec![0; ram_bytes],
            ram_rtc_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc: Rtc::new(),
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
                if val == 0x00 {
                    self.rtc.latch_pending = true;
                } else if val == 0x01 && self.rtc.latch_pending {
                    self.rtc.latch();
                    self.rtc.latch_pending = false;
                } else {
                    self.rtc.latch_pending = false;
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
                let offset = self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000);
                self.ram.get(offset).copied().unwrap_or(0xFF)
            }
            0x08..=0x0C => self.rtc.read(self.ram_bank),
            _ => 0xFF,
        }
    }

    fn ram_write(&mut self, addr: u16, val: u8) {
        if !self.ram_rtc_enabled {
            return;
        }
        match self.ram_bank {
            0x00..=0x03 => {
                let offset = self.ram_bank as usize * 0x2000 + (addr as usize - 0xA000);
                if let Some(b) = self.ram.get_mut(offset) {
                    *b = val;
                }
            }
            0x08..=0x0C => self.rtc.write(self.ram_bank, val),
            _ => {}
        }
    }

    fn save_ram(&self) -> Vec<u8> {
        let mut data = self.ram.clone();
        // Append RTC state: origin(8) + halted(1) + halt_ts(8) + carry(1) = 18 bytes
        data.extend_from_slice(&self.rtc.origin.to_le_bytes());
        data.push(self.rtc.halted as u8);
        data.extend_from_slice(&self.rtc.halt_ts.to_le_bytes());
        data.push(self.rtc.carry as u8);
        data
    }

    fn load_ram(&mut self, data: &[u8]) {
        let ram_len = self.ram.len();
        let len = ram_len.min(data.len());
        self.ram[..len].copy_from_slice(&data[..len]);
        // Parse RTC state if present after RAM bytes
        let rtc_data = &data[ram_len.min(data.len())..];
        if rtc_data.len() >= 18 {
            self.rtc.origin   = u64::from_le_bytes(rtc_data[0..8].try_into().unwrap());
            self.rtc.halted   = rtc_data[8] != 0;
            self.rtc.halt_ts  = u64::from_le_bytes(rtc_data[9..17].try_into().unwrap());
            self.rtc.carry    = rtc_data[17] != 0;
        }
    }
}
