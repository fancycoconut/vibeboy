use vibeboy::ppu::Ppu;

const WHITE: [u8; 3] = [0xE0, 0xF0, 0xE0];
const LGREY: [u8; 3] = [0x88, 0xB0, 0x88];
const DGREY: [u8; 3] = [0x30, 0x60, 0x30];
const BLACK: [u8; 3] = [0x08, 0x18, 0x08];

fn oam_empty() -> [u8; 0xA0] {
    [0u8; 0xA0]
}

/// Advance the PPU through one complete scanline.
/// mode 2 (OAM scan, 80 dots) → mode 3 (draw, 172 dots) → mode 0 (HBlank, 204 dots)
fn step_scanline(ppu: &mut Ppu, oam: &[u8; 0xA0]) {
    ppu.step(20, oam);
    ppu.step(43, oam);
    ppu.step(51, oam);
}

fn px(fb: &[u8], x: usize, y: usize) -> [u8; 3] {
    let base = (y * 160 + x) * 3;
    [fb[base], fb[base + 1], fb[base + 2]]
}

/// BG tile data at 0x8000 (unsigned, LCDC bit 4 = 1).
/// Tile row with lo=0xAA, hi=0x00 produces alternating color_ids 1,0,1,0...
#[test]
fn bg_unsigned_tile_addressing() {
    let mut ppu = Ppu::new();
    ppu.reg_write(0xFF40, 0x91); // LCD on, tile data=0x8000, BG on
    ppu.reg_write(0xFF47, 0xE4); // BGP: 0→white, 1→lgrey, 2→dgrey, 3→black

    ppu.vram_write(0x8000, 0xAA); // lo: 10101010
    ppu.vram_write(0x8001, 0x00); // hi: 00000000

    let oam = oam_empty();
    step_scanline(&mut ppu, &oam);

    let fb = ppu.framebuffer.as_ref();
    assert_eq!(px(fb, 0, 0), LGREY, "pixel 0: color_id 1");
    assert_eq!(px(fb, 1, 0), WHITE, "pixel 1: color_id 0");
    assert_eq!(px(fb, 2, 0), LGREY, "pixel 2: color_id 1");
    assert_eq!(px(fb, 3, 0), WHITE, "pixel 3: color_id 0");
}

/// BG tile data at 0x9000 (signed, LCDC bit 4 = 0).
/// Tile ID 0 lives at 0x9000 (VRAM offset 0x1000).
/// lo=0x00, hi=0xFF → color_id 2 for every pixel → DGREY.
#[test]
fn bg_signed_tile_addressing() {
    let mut ppu = Ppu::new();
    ppu.reg_write(0xFF40, 0x81); // LCD on, tile data=0x9000, BG on
    ppu.reg_write(0xFF47, 0xE4);

    ppu.vram_write(0x9000, 0x00); // lo: 00000000
    ppu.vram_write(0x9001, 0xFF); // hi: 11111111 → color_id 2 everywhere

    let oam = oam_empty();
    step_scanline(&mut ppu, &oam);

    let fb = ppu.framebuffer.as_ref();
    for x in 0..8 {
        assert_eq!(px(fb, x, 0), DGREY, "pixel {x}");
    }
}

/// A sprite at (0,0) with all-opaque pixels renders over a disabled BG.
#[test]
fn sprite_basic_rendering() {
    let mut ppu = Ppu::new();
    ppu.reg_write(0xFF40, 0x82); // LCD on, sprites on, BG off
    ppu.reg_write(0xFF48, 0xE4); // OBP0: color_id 1 → LGREY

    // Tile 0: lo=0xFF, hi=0x00 → color_id 1 for every pixel
    ppu.vram_write(0x8000, 0xFF);
    ppu.vram_write(0x8001, 0x00);

    let mut oam = oam_empty();
    oam[0] = 16; // Y: sprite top at scanline 0
    oam[1] = 8;  // X: sprite left at pixel 0
    oam[2] = 0;  // tile 0
    oam[3] = 0;  // no flags

    step_scanline(&mut ppu, &oam);

    let fb = ppu.framebuffer.as_ref();
    for x in 0..8 {
        assert_eq!(px(fb, x, 0), LGREY, "sprite pixel {x}");
    }
    assert_eq!(px(fb, 8, 0), [0, 0, 0], "no sprite at pixel 8");
}

/// OAM flag bit 7 (bg_over): sprite is hidden behind non-transparent BG pixels
/// but still appears over transparent (color_id 0) BG pixels.
#[test]
fn sprite_bg_priority() {
    let mut ppu = Ppu::new();
    ppu.reg_write(0xFF40, 0x93); // LCD on, sprites on, tile data=0x8000 (bit4), BG on
    ppu.reg_write(0xFF47, 0xE4); // BGP: color_id 1 → LGREY
    ppu.reg_write(0xFF48, 0xFC); // OBP0: color_id 1 → BLACK (distinguishable from BG)

    // BG tile 0: lo=0xAA, hi=0x00 → even pixels nonzero (color_id 1), odd pixels zero
    ppu.vram_write(0x8000, 0xAA);
    ppu.vram_write(0x8001, 0x00);

    // Sprite tile 1: lo=0xFF, hi=0x00 → color_id 1 everywhere → BLACK via OBP0
    ppu.vram_write(0x8010, 0xFF);
    ppu.vram_write(0x8011, 0x00);

    let mut oam = oam_empty();
    oam[0] = 16;   // Y
    oam[1] = 8;    // X
    oam[2] = 1;    // tile 1
    oam[3] = 0x80; // bg_over: sprite behind BG

    step_scanline(&mut ppu, &oam);

    let fb = ppu.framebuffer.as_ref();
    // Pixel 0: BG color_id=1 (nonzero) + bg_over → BG wins → LGREY
    assert_eq!(px(fb, 0, 0), LGREY, "bg_over: BG nonzero should win");
    // Pixel 1: BG color_id=0 (transparent) + bg_over → sprite wins → BLACK
    assert_eq!(px(fb, 1, 0), BLACK, "bg_over: sprite wins on transparent BG pixel");
}
