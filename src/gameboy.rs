use crate::bus::Bus;
use crate::cartridge::CartridgeHeader;
use crate::cpu::Cpu;

/// Top-level emulator struct. Owns the CPU and bus; drives the step loop.
pub struct GameBoy {
    pub cpu: Cpu,
    pub bus: Bus,
    /// Cycle count from the previous instruction, used to tick the APU
    /// *before* the next instruction executes. This ensures the APU's wave
    /// position and access timer advance before the CPU reads wave RAM,
    /// giving the wave_access_timer a non-zero value at read time.
    prev_cycles: u8,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        let is_gbc = CartridgeHeader::parse(&rom).is_gbc();
        let mut gb = Self {
            cpu: Cpu::new(is_gbc),
            bus: Bus::new(rom),
            prev_cycles: 0,
        };
        gb.bus.ppu.cgb_mode = is_gbc;
        gb.bus.apu.is_gbc = is_gbc;
        gb
    }

    /// Execute one CPU instruction and advance all other components.
    /// Returns true when a VBlank just completed (frame is ready).
    pub fn step(&mut self) -> bool {
        // Tick the APU for the previous instruction's cycles before the CPU
        // executes the current one. This makes the APU's wave position advance
        // ahead of the CPU's bus reads, so wave_access_timer is non-zero when
        // a read of wave RAM occurs in the same step as the wave clock.
        self.bus.tick_m_cycle(self.prev_cycles);

        let cycles = self.cpu.step(&mut self.bus);
        self.prev_cycles = cycles;

        self.bus.timer.step(cycles, &mut self.bus.interrupts);

        let oam = self.bus.oam_snapshot();
        self.bus.ppu.step(cycles, &oam);

        // Fire one HBlank DMA block each time the PPU enters HBlank.
        if self.bus.ppu.hblank_triggered {
            self.bus.hdma_hblank_step();
        }

        if self.bus.ppu.int_vblank {
            self.bus.interrupts.request(0);
        }
        if self.bus.ppu.int_stat {
            self.bus.interrupts.request(1);
        }

        if self.bus.ppu.frame_ready {
            self.bus.ppu.frame_ready = false;
            true
        } else {
            false
        }
    }

    /// Run until a complete frame is ready, then return the framebuffer.
    pub fn run_frame(&mut self) -> &[u8; 160 * 144 * 3] {
        while !self.step() {}
        &self.bus.ppu.framebuffer
    }
}
