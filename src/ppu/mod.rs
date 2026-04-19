/// Pixel Processing Unit — DMG scanline renderer with GBC color palette stubs.
///
/// Modes per scanline (456 dots total):
///   Mode 2 (OAM Scan)  — dots 0–79
///   Mode 3 (Draw)      — dots 80–251 (variable, simplified to 172)
///   Mode 0 (HBlank)    — dots 252–455
/// Lines 144–153: Mode 1 (VBlank)
pub struct Ppu {
    /// Two 8KB VRAM banks (bank 1 GBC only; DMG uses bank 0 only)
    vram: [[u8; 0x2000]; 2],
    vram_bank: usize,

    // LCD control / status registers
    pub lcdc: u8,     // 0xFF40
    pub stat: u8,     // 0xFF41
    pub scy: u8,      // 0xFF42
    pub scx: u8,      // 0xFF43
    pub ly: u8,       // 0xFF44 — current scanline
    pub lyc: u8,      // 0xFF45
    pub bgp: u8,      // 0xFF47 — DMG BG palette
    pub obp0: u8,     // 0xFF48
    pub obp1: u8,     // 0xFF49
    pub wy: u8,       // 0xFF4A
    pub wx: u8,       // 0xFF4B

    // GBC color palettes
    bcps: u8,                   // 0xFF68 BG palette spec
    bcpd: [u8; 64],             // 0xFF69 BG palette data (8 palettes × 4 colors × 2 bytes)
    ocps: u8,                   // 0xFF6A OBJ palette spec
    ocpd: [u8; 64],             // 0xFF6B OBJ palette data

    // Internal state
    dot: u16,                  // dot counter within current scanline (0–455)
    pub mode: u8,              // current PPU mode (0–3)

    /// Completed frame framebuffer: 160×144 pixels, 3 bytes (RGB) each.
    pub framebuffer: Box<[u8; 160 * 144 * 3]>,
    /// Set to true on VBlank start; cleared by the main loop after blitting.
    pub frame_ready: bool,

    /// Interrupt flags to request (caller checks after each step)
    pub int_vblank: bool,
    pub int_stat: bool,
}

/// DMG 4-shade palette (greenish).
const DMG_COLORS: [[u8; 3]; 4] = [
    [0xE0, 0xF0, 0xE0], // 0 — lightest
    [0x88, 0xB0, 0x88], // 1
    [0x30, 0x60, 0x30], // 2
    [0x08, 0x18, 0x08], // 3 — darkest
];

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [[0; 0x2000]; 2],
            vram_bank: 0,
            lcdc: 0x91,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0xFC,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            bcps: 0,
            bcpd: [0xFF; 64],
            ocps: 0,
            ocpd: [0xFF; 64],
            dot: 0,
            mode: 2,
            framebuffer: Box::new([0; 160 * 144 * 3]),
            frame_ready: false,
            int_vblank: false,
            int_stat: false,
        }
    }

    // -----------------------------------------------------------------------
    // VRAM access
    // -----------------------------------------------------------------------

    pub fn vram_read(&self, addr: u16) -> u8 {
        // VRAM inaccessible during mode 3
        if self.mode == 3 {
            return 0xFF;
        }
        self.vram[self.vram_bank][(addr - 0x8000) as usize]
    }

    pub fn vram_write(&mut self, addr: u16, val: u8) {
        if self.mode == 3 {
            return;
        }
        self.vram[self.vram_bank][(addr - 0x8000) as usize] = val;
    }

    pub fn vram_bank_read(&self) -> u8 {
        0xFE | self.vram_bank as u8
    }

    pub fn vram_bank_write(&mut self, val: u8) {
        self.vram_bank = (val & 0x01) as usize;
    }

    // -----------------------------------------------------------------------
    // I/O register access
    // -----------------------------------------------------------------------

    pub fn reg_read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat | 0x80, // bit 7 always set
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    pub fn reg_write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF40 => {
                let was_on = self.lcdc & 0x80 != 0;
                self.lcdc = val;
                if was_on && val & 0x80 == 0 {
                    // LCD turned off: real hardware resets to mode 0, LY=0, dot=0
                    // so VRAM is accessible and next enable starts from a clean state.
                    self.ly = 0;
                    self.dot = 0;
                    self.set_mode(0);
                }
            }
            0xFF41 => self.stat = (self.stat & 0x07) | (val & 0x78),
            0xFF42 => self.scy = val,
            0xFF43 => self.scx = val,
            0xFF44 => {} // LY is read-only
            0xFF45 => self.lyc = val,
            0xFF46 => {} // OAM DMA handled by Bus
            0xFF47 => self.bgp = val,
            0xFF48 => self.obp0 = val,
            0xFF49 => self.obp1 = val,
            0xFF4A => self.wy = val,
            0xFF4B => self.wx = val,
            _ => {}
        }
    }

    pub fn cgb_palette_read(&self, addr: u16) -> u8 {
        match addr {
            0xFF68 => self.bcps,
            0xFF69 => self.bcpd[(self.bcps & 0x3F) as usize],
            0xFF6A => self.ocps,
            0xFF6B => self.ocpd[(self.ocps & 0x3F) as usize],
            _ => 0xFF,
        }
    }

    pub fn cgb_palette_write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF68 => self.bcps = val,
            0xFF69 => {
                let idx = (self.bcps & 0x3F) as usize;
                self.bcpd[idx] = val;
                if self.bcps & 0x80 != 0 {
                    self.bcps = (self.bcps & 0x80) | ((idx as u8 + 1) & 0x3F);
                }
            }
            0xFF6A => self.ocps = val,
            0xFF6B => {
                let idx = (self.ocps & 0x3F) as usize;
                self.ocpd[idx] = val;
                if self.ocps & 0x80 != 0 {
                    self.ocps = (self.ocps & 0x80) | ((idx as u8 + 1) & 0x3F);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Step / rendering
    // -----------------------------------------------------------------------

    /// Advance the PPU by `cycles` machine cycles (4 dots each).
    /// Returns interrupt flags: (vblank_requested, stat_requested).
    pub fn step(&mut self, cycles: u8, oam: &[u8; 0xA0]) {
        self.int_vblank = false;
        self.int_stat = false;

        if self.lcdc & 0x80 == 0 {
            // LCD off
            return;
        }

        let dots = cycles as u16 * 4;
        self.dot += dots;

        match self.mode {
            2 => {
                // OAM scan
                if self.dot >= 80 {
                    self.dot -= 80;
                    self.set_mode(3);
                }
            }
            3 => {
                // Drawing
                if self.dot >= 172 {
                    self.dot -= 172;
                    self.render_scanline(oam);
                    self.set_mode(0);
                    if self.stat & 0x08 != 0 {
                        self.int_stat = true;
                    }
                }
            }
            0 => {
                // HBlank
                if self.dot >= 204 {
                    self.dot -= 204;
                    self.ly += 1;
                    self.check_lyc();
                    if self.ly == 144 {
                        self.set_mode(1);
                        self.int_vblank = true;
                        self.frame_ready = true;
                        if self.stat & 0x10 != 0 {
                            self.int_stat = true;
                        }
                    } else {
                        self.set_mode(2);
                        if self.stat & 0x20 != 0 {
                            self.int_stat = true;
                        }
                    }
                }
            }
            1 => {
                // VBlank — 10 lines × 456 dots
                if self.dot >= 456 {
                    self.dot -= 456;
                    self.ly += 1;
                    self.check_lyc();
                    if self.ly > 153 {
                        self.ly = 0;
                        self.set_mode(2);
                        if self.stat & 0x20 != 0 {
                            self.int_stat = true;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn set_mode(&mut self, mode: u8) {
        self.mode = mode;
        self.stat = (self.stat & 0xF8) | (mode & 0x03);
    }

    fn check_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat |= 0x04;
            if self.stat & 0x40 != 0 {
                self.int_stat = true;
            }
        } else {
            self.stat &= !0x04;
        }
    }

    // -----------------------------------------------------------------------
    // Scanline rendering (simplified scanline rasterizer)
    // -----------------------------------------------------------------------

    fn render_scanline(&mut self, oam: &[u8; 0xA0]) {
        let line = self.ly as usize;
        if line >= 144 {
            return;
        }

        let mut line_buf = [0u8; 160]; // colour index (0–3) per pixel
        let mut bg_priority = [false; 160]; // BG-over-sprite flag

        // --- Background ---
        if self.lcdc & 0x01 != 0 {
            let tile_map_base: u16 = if self.lcdc & 0x08 != 0 { 0x9C00 } else { 0x9800 };
            // Bit 4 = 1: unsigned, tiles 0-255 at 0x8000. Bit 4 = 0: signed, tile 0 at 0x9000.
            let tile_data_base: u16 = if self.lcdc & 0x10 != 0 { 0x8000 } else { 0x9000 };
            let signed_addressing = self.lcdc & 0x10 == 0;

            let y = line.wrapping_add(self.scy as usize) & 0xFF;
            let tile_row = y / 8;
            let tile_y = y % 8;

            for px in 0..160usize {
                let x = (px + self.scx as usize) & 0xFF;
                let tile_col = x / 8;
                let tile_x = x % 8;

                let map_addr = tile_map_base + (tile_row * 32 + tile_col) as u16;
                let tile_id = self.vram[0][(map_addr - 0x8000) as usize];

                let tile_addr = if signed_addressing {
                    let id = tile_id as i8 as i16;
                    ((tile_data_base as i32) + (id as i32) * 16) as u16
                } else {
                    tile_data_base + tile_id as u16 * 16
                };

                let row_addr = (tile_addr - 0x8000) as usize + tile_y * 2;
                let lo = self.vram[0][row_addr];
                let hi = self.vram[0][row_addr + 1];

                let bit = 7 - tile_x;
                let color_id = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
                let palette_color = (self.bgp >> (color_id * 2)) & 0x03;
                line_buf[px] = palette_color;
                bg_priority[px] = color_id != 0;
            }
        }

        // --- Window ---
        if self.lcdc & 0x20 != 0 && line >= self.wy as usize {
            let wx = self.wx.saturating_sub(7) as usize;
            let win_map_base: u16 = if self.lcdc & 0x40 != 0 { 0x9C00 } else { 0x9800 };
            let tile_data_base: u16 = if self.lcdc & 0x10 != 0 { 0x8000 } else { 0x9000 };
            let signed_addressing = self.lcdc & 0x10 == 0;

            let y = line - self.wy as usize;
            let tile_row = y / 8;
            let tile_y = y % 8;

            for px in wx..160usize {
                let x = px - wx;
                let tile_col = x / 8;
                let tile_x = x % 8;

                let map_addr = win_map_base + (tile_row * 32 + tile_col) as u16;
                let tile_id = self.vram[0][(map_addr - 0x8000) as usize];

                let tile_addr = if signed_addressing {
                    let id = tile_id as i8 as i16;
                    ((tile_data_base as i32) + (id as i32) * 16) as u16
                } else {
                    tile_data_base + tile_id as u16 * 16
                };

                let row_addr = (tile_addr - 0x8000) as usize + tile_y * 2;
                let lo = self.vram[0][row_addr];
                let hi = self.vram[0][row_addr + 1];

                let bit = 7 - tile_x;
                let color_id = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
                let palette_color = (self.bgp >> (color_id * 2)) & 0x03;
                line_buf[px] = palette_color;
                bg_priority[px] = color_id != 0;
            }
        }

        // --- Sprites ---
        if self.lcdc & 0x02 != 0 {
            let sprite_height: i32 = if self.lcdc & 0x04 != 0 { 16 } else { 8 };
            let mut visible: Vec<(i32, usize)> = Vec::new(); // (x, oam_index)

            for i in 0..40usize {
                let base = i * 4;
                let sy = oam[base] as i32 - 16;
                let sx = oam[base + 1] as i32 - 8;
                if (line as i32) >= sy && (line as i32) < sy + sprite_height {
                    visible.push((sx, i));
                    if visible.len() == 10 {
                        break;
                    }
                }
            }

            // Lower OAM index has priority (draw last so it lands on top)
            visible.reverse();

            for (sx, i) in visible {
                let base = i * 4;
                let sy = oam[base] as i32 - 16;
                let flags = oam[base + 3];
                let tile_id = oam[base + 2] & if sprite_height == 16 { 0xFE } else { 0xFF };
                let palette = if flags & 0x10 != 0 { self.obp1 } else { self.obp0 };
                let flip_x = flags & 0x20 != 0;
                let flip_y = flags & 0x40 != 0;
                let bg_over = flags & 0x80 != 0;

                let mut tile_y = (line as i32 - sy) as usize;
                if flip_y {
                    tile_y = sprite_height as usize - 1 - tile_y;
                }

                let row_addr = tile_id as usize * 16 + tile_y * 2;
                let lo = self.vram[0][row_addr];
                let hi = self.vram[0][row_addr + 1];

                for tile_x in 0..8usize {
                    let px = sx + tile_x as i32;
                    if px < 0 || px >= 160 {
                        continue;
                    }
                    let px = px as usize;

                    if bg_over && bg_priority[px] {
                        continue;
                    }

                    let bit = if flip_x { tile_x } else { 7 - tile_x };
                    let color_id = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
                    if color_id == 0 {
                        continue; // transparent
                    }
                    let palette_color = (palette >> (color_id * 2)) & 0x03;
                    line_buf[px] = palette_color;
                }
            }
        }

        // --- Write to framebuffer ---
        let fb_base = line * 160 * 3;
        for (px, &color_idx) in line_buf.iter().enumerate() {
            let rgb = DMG_COLORS[color_idx as usize];
            self.framebuffer[fb_base + px * 3] = rgb[0];
            self.framebuffer[fb_base + px * 3 + 1] = rgb[1];
            self.framebuffer[fb_base + px * 3 + 2] = rgb[2];
        }
    }
}
