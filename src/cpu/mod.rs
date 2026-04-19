mod instructions;
#[cfg(test)]
mod tests;

use crate::bus::Bus;

/// Sharp LR35902 CPU registers.
///
/// Flag register F layout: Z N H C 0 0 0 0
///   Z (bit7): Zero flag
///   N (bit6): Subtract flag
///   H (bit5): Half Carry flag
///   C (bit4): Carry flag
#[derive(Default)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    // 16-bit register pair getters
    pub fn af(&self) -> u16 { ((self.a as u16) << 8) | (self.f as u16) }
    pub fn bc(&self) -> u16 { ((self.b as u16) << 8) | (self.c as u16) }
    pub fn de(&self) -> u16 { ((self.d as u16) << 8) | (self.e as u16) }
    pub fn hl(&self) -> u16 { ((self.h as u16) << 8) | (self.l as u16) }

    // 16-bit register pair setters
    pub fn set_af(&mut self, v: u16) { self.a = (v >> 8) as u8; self.f = (v & 0xF0) as u8; }
    pub fn set_bc(&mut self, v: u16) { self.b = (v >> 8) as u8; self.c = v as u8; }
    pub fn set_de(&mut self, v: u16) { self.d = (v >> 8) as u8; self.e = v as u8; }
    pub fn set_hl(&mut self, v: u16) { self.h = (v >> 8) as u8; self.l = v as u8; }

    // Flag helpers
    pub fn flag_z(&self) -> bool { self.f & 0x80 != 0 }
    pub fn flag_n(&self) -> bool { self.f & 0x40 != 0 }
    pub fn flag_h(&self) -> bool { self.f & 0x20 != 0 }
    pub fn flag_c(&self) -> bool { self.f & 0x10 != 0 }

    pub fn set_flag_z(&mut self, v: bool) { if v { self.f |= 0x80 } else { self.f &= !0x80 } }
    pub fn set_flag_n(&mut self, v: bool) { if v { self.f |= 0x40 } else { self.f &= !0x40 } }
    pub fn set_flag_h(&mut self, v: bool) { if v { self.f |= 0x20 } else { self.f &= !0x20 } }
    pub fn set_flag_c(&mut self, v: bool) { if v { self.f |= 0x10 } else { self.f &= !0x10 } }

    pub fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.set_flag_z(z);
        self.set_flag_n(n);
        self.set_flag_h(h);
        self.set_flag_c(c);
    }
}

pub struct Cpu {
    pub reg: Registers,
    /// Interrupt Master Enable
    pub ime: bool,
    /// EI schedules IME=true after the next instruction
    ime_scheduled: bool,
    /// HALT state
    halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        // Post-boot ROM register state (DMG)
        let mut reg = Registers::default();
        reg.a = 0x01;
        reg.f = 0xB0;
        reg.b = 0x00;
        reg.c = 0x13;
        reg.d = 0x00;
        reg.e = 0xD8;
        reg.h = 0x01;
        reg.l = 0x4D;
        reg.sp = 0xFFFE;
        reg.pc = 0x0100;
        Self {
            reg,
            ime: false,
            ime_scheduled: false,
            halted: false,
        }
    }

    /// Execute one instruction (or service an interrupt).
    /// Returns the number of machine cycles consumed.
    pub fn step(&mut self, bus: &mut Bus) -> u8 {
        // Apply deferred EI
        if self.ime_scheduled {
            self.ime = true;
            self.ime_scheduled = false;
        }

        // Handle interrupts
        if self.ime {
            if let Some(bit) = bus.interrupts.pending() {
                bus.interrupts.ack(bit);
                self.ime = false;
                self.halted = false;
                self.push16(bus, self.reg.pc);
                self.reg.pc = 0x0040 + (bit as u16) * 8;
                return 5; // ISR takes 5 machine cycles
            }
        }

        if self.halted {
            // Check if any interrupt became pending (even with IME=false, HALT exits)
            if bus.interrupts.pending().is_some() {
                self.halted = false;
            } else {
                return 1;
            }
        }

        instructions::execute(self, bus)
    }

    // -----------------------------------------------------------------------
    // Stack helpers
    // -----------------------------------------------------------------------

    pub fn push8(&mut self, bus: &mut Bus, val: u8) {
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        bus.write(self.reg.sp, val);
    }

    pub fn pop8(&mut self, bus: &mut Bus) -> u8 {
        let val = bus.read(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);
        val
    }

    pub fn push16(&mut self, bus: &mut Bus, val: u16) {
        self.push8(bus, (val >> 8) as u8);
        self.push8(bus, val as u8);
    }

    pub fn pop16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.pop8(bus) as u16;
        let hi = self.pop8(bus) as u16;
        (hi << 8) | lo
    }

    // -----------------------------------------------------------------------
    // Fetch helpers
    // -----------------------------------------------------------------------

    pub fn fetch8(&mut self, bus: &Bus) -> u8 {
        let v = bus.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        v
    }

    pub fn fetch16(&mut self, bus: &Bus) -> u16 {
        let lo = self.fetch8(bus) as u16;
        let hi = self.fetch8(bus) as u16;
        (hi << 8) | lo
    }

    pub fn schedule_ei(&mut self) {
        self.ime_scheduled = true;
    }

    pub fn halt(&mut self) {
        self.halted = true;
    }
}
