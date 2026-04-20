use crate::bus::Bus;
use crate::cartridge::CartridgeHeader;
use crate::cpu::Cpu;

/// Top-level emulator struct. Owns the CPU and bus; drives the step loop.
pub struct GameBoy {
    pub cpu: Cpu,
    pub bus: Bus,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        let is_gbc = CartridgeHeader::parse(&rom).is_gbc();
        let mut gb = Self {
            cpu: Cpu::new(is_gbc),
            bus: Bus::new(rom),
        };
        gb.bus.ppu.cgb_mode = is_gbc;
        gb
    }

    /// Execute one CPU instruction and advance all other components.
    /// Returns true when a VBlank just completed (frame is ready).
    pub fn step(&mut self) -> bool {
        let cycles = self.cpu.step(&mut self.bus);

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
