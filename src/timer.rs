use crate::interrupts::Interrupts;

/// Timer registers: DIV (0xFF03), TIMA (0xFF05), TMA (0xFF06), TAC (0xFF07).
pub struct Timer {
    /// Internal 16-bit counter. Upper byte is exposed as DIV.
    div_counter: u16,
    /// TIMA — timer counter
    pub tima: u8,
    /// TMA — timer modulo (reload value on overflow)
    pub tma: u8,
    /// TAC — timer control: bit2=enable, bits1-0=clock select
    pub tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => (self.div_counter >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.div_counter = 0,
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => self.tac = val & 0x07,
            _ => {}
        }
    }

    /// Advance timer by `cycles` (machine cycles = T-cycles / 4).
    pub fn step(&mut self, cycles: u8, interrupts: &mut Interrupts) {
        let t_cycles = cycles as u32 * 4;
        let prev = self.div_counter as u32;
        self.div_counter = self.div_counter.wrapping_add(t_cycles as u16);

        if self.tac & 0x04 != 0 {
            // Count falling edges on the selected DIV bit. A falling edge on bit B
            // occurs at every multiple of 2^(B+1), so we count how many such
            // multiples fall in the half-open interval (prev, prev + t_cycles].
            let bit = Self::tac_to_div_bit(self.tac & 0x03) as u32;
            let period = 1u32 << (bit + 1);
            let next = prev + t_cycles; // safe in u32; t_cycles is at most 24
            let falling_edges = next / period - prev / period;
            for _ in 0..falling_edges {
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0 {
                    self.tima = self.tma;
                    interrupts.request(2);
                }
            }
        }
    }

    fn tac_to_div_bit(clock_select: u8) -> u8 {
        match clock_select {
            0 => 9,  // 4096 Hz
            1 => 3,  // 262144 Hz
            2 => 5,  // 65536 Hz
            _ => 7,  // 16384 Hz
        }
    }
}
