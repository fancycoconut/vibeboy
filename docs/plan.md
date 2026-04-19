# Vibeboy — Game Boy Emulator in Rust

## Context

The project is starting fresh in Rust on the `feature-initial-rust` branch. A previous Go prototype implemented ~13 CPU instructions before being wiped. The goal is a **DMG + GBC** emulator built with a **ship-fast** mentality using **SDL2** for windowing, rendering, and audio. We have `red.gb` (Pokemon Red — MBC1 cartridge) as the test ROM.

The approach: get pixels on screen fast, fix accuracy as we go. We'll design the architecture to accommodate GBC from day one (so we don't have to rip it apart later), but GBC features are a second pass.

---

## Architecture

```
vibeboy/
├── Cargo.toml
├── src/
│   ├── main.rs              # SDL2 init, event loop, render framebuffer
│   ├── gameboy.rs           # Top-level struct: owns all components, drives step()
│   ├── bus.rs               # Memory bus: routes reads/writes to correct component
│   ├── cpu/
│   │   ├── mod.rs           # CPU struct, registers (inc. F flags), step()
│   │   └── instructions.rs  # All opcode handlers (unprefixed + 0xCB prefix)
│   ├── ppu/
│   │   └── mod.rs           # PPU struct, mode state machine, pixel output
│   ├── apu/
│   │   └── mod.rs           # APU stub (wire up later)
│   ├── cartridge/
│   │   ├── mod.rs           # Cartridge trait + header parsing
│   │   ├── mbc0.rs          # ROM only (<=32KB, no banking)
│   │   └── mbc1.rs          # MBC1 (Pokemon Red/Blue, Tetris, etc.)
│   ├── timer.rs             # DIV / TIMA / TMA / TAC
│   ├── joypad.rs            # Joypad register (0xFF00)
│   └── interrupts.rs        # IF (0xFF0F) and IE (0xFFFF) registers
└── red.gb
```

---

## Implementation Plan

### Phase 1 — Project Setup
- `cargo init` with `sdl2 = { version = "0.37", features = ["bundled"] }` in Cargo.toml
- Stub out all modules with empty structs so the project compiles
- `GameBoy` struct owns: `Cpu`, `Bus`, `Ppu`, `Apu`, `Timer`, `Joypad`, `Interrupts`
- `Bus` owns the cartridge (via `Box<dyn Cartridge>`) and all RAM regions

### Phase 2 — Cartridge & Memory Bus

**Cartridge trait:**
```rust
pub trait Cartridge {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}
```
- Parse the ROM header at 0x0100–0x014F: title, cartridge type, ROM/RAM size
- Implement `Mbc0` (0x00) for simple ROMs
- Implement `Mbc1` (0x01–0x03) for Pokemon Red — ROM bank switching at 0x2000, RAM bank at 0x4000
- `factory(rom: Vec<u8>) -> Box<dyn Cartridge>` dispatches on header byte 0x0147

**Memory map in `Bus`:**

| Range | Component |
|---|---|
| 0x0000–0x7FFF | Cartridge ROM |
| 0x8000–0x9FFF | VRAM (8KB DMG / 16KB GBC banked) |
| 0xA000–0xBFFF | External RAM (cartridge) |
| 0xC000–0xCFFF | Work RAM bank 0 |
| 0xD000–0xDFFF | Work RAM bank 1–7 (GBC) |
| 0xE000–0xFDFF | Echo RAM (mirrors 0xC000) |
| 0xFE00–0xFE9F | OAM |
| 0xFF00–0xFF7F | I/O registers (dispatch to PPU/Timer/Joypad/APU) |
| 0xFF80–0xFFFE | High RAM (HRAM) |
| 0xFFFF | IE register |

### Phase 3 — CPU

**Registers struct:**
```rust
pub struct Registers {
    pub a: u8, pub f: u8,  // AF pair (F = flags: Z N H C _ _ _ _)
    pub b: u8, pub c: u8,  // BC pair
    pub d: u8, pub e: u8,  // DE pair
    pub h: u8, pub l: u8,  // HL pair
    pub sp: u16, pub pc: u16,
}
// Helper methods: af(), bc(), de(), hl(), set_af(), etc.
// Flag helpers: zero_flag(), set_zero_flag(), etc.
```

**Instruction dispatch in `instructions.rs`:**
- `step(bus: &mut Bus) -> u8` returns machine cycles consumed
- Match on opcode byte, handle all 256 unprefixed + 256 CB-prefixed = 512 opcodes
- Group by family: LD, ALU (ADD/SUB/AND/OR/XOR/CP/INC/DEC), JR/JP/CALL/RET, PUSH/POP, BIT ops, etc.
- Interrupts: check IE & IF at start of each step; if IME set and interrupt pending, push PC and jump to vector

Ship-fast order: implement the opcodes the boot ROM and early game code actually hit. Use `unimplemented!()` with the opcode hex for anything not yet handled — this surfaces what's needed as tests run.

### Phase 4 — PPU (Scanline Renderer)

PPU has 4 modes, cycling per scanline (154 scanlines total, 456 dots each):

| Mode | Dots | Action |
|---|---|---|
| 2 (OAM Scan) | 80 | Collect up to 10 sprites for this line |
| 3 (Draw) | 172–289 | Output 160 pixels to line buffer |
| 0 (HBlank) | remaining | Idle, fire HBlank interrupt if enabled |
| 1 (VBlank) | lines 144–153 | Fire VBlank interrupt, present frame |

**Framebuffer:** `[u8; 160 * 144 * 3]` (RGB bytes). PPU writes to it; `main.rs` blits to SDL2 texture each VBlank.

**Rendering pipeline (scanline approach):**
1. Read LCDC (0xFF40): check BG/sprite/window enable bits
2. Compute BG tile map address from SCY/SCX, read tile IDs from 0x9800 or 0x9C00
3. Fetch tile data from 0x8000 or 0x8800 addressing modes
4. Mix sprites (OAM) with priority rules
5. Apply DMG palette (BGP 0xFF47, OBP0/OBP1 0xFF48–49) → 4 grey shades
6. Write pixel to framebuffer

### Phase 5 — SDL2 Integration & Main Loop

```
main loop:
  while running:
    poll SDL2 events → update joypad state, handle quit
    step GameBoy until 70224 cycles elapsed (one frame worth)
    blit framebuffer to SDL2 texture → present
    cap to ~60 FPS
```
- SDL2 window: 320×288 (2× scale) or 480×432 (3×)
- Keyboard map: arrow keys → D-pad, Z → B, X → A, Enter → Start, Shift → Select

### Phase 6 — Timer & Interrupts
- `DIV` (0xFF03): increments at 16384 Hz (every 256 CPU cycles), write resets to 0
- `TIMA` (0xFF05): increments at rate set by TAC, overflows → load TMA, request Timer interrupt
- Interrupt vectors: VBlank=0x40, LCD=0x48, Timer=0x50, Serial=0x58, Joypad=0x60
- `IME` (interrupt master enable): set by `EI`, cleared by `DI` and interrupt dispatch; `RETI` sets it

### Phase 7 — Joypad
- 0xFF00 register: upper nibble selects which button group to read (D-pad or action buttons)
- On keypress: clear bit in register, optionally fire Joypad interrupt

### Phase 8 — Validation

Run **blargg's test ROMs** to find accuracy gaps:
- Wire serial output (0xFF01/0xFF02) to stdout for test result printing
- Target: `cpu_instrs` all 11 subtests pass
- Integration milestone: Pokemon Red reaches the title screen and accepts joypad input

### Phase 9 — GBC Extensions (Second Pass)
- Double speed mode (KEY1 register 0xFF4D)
- VRAM bank switching (VBK register 0xFF4F) — second 8KB VRAM bank
- WRAM bank switching (SVBK register 0xFF70) — banks 1–7
- Color palettes: BCPS/BCPD (0xFF68/69) and OCPS/OCPD (0xFF6A/6B) — 8 palettes × 4 colors
- OAM extended attributes: VRAM bank bit, palette number, flip flags

---

## Milestones

| # | Milestone | Signal |
|---|---|---|
| 1 | Project compiles | `cargo build` succeeds |
| 2 | ROM info prints | Cartridge header parsed, title logged |
| 3 | CPU steps | Opcodes execute without panic on early boot |
| 4 | First pixels | PPU outputs scanlines (even if garbled) |
| 5 | Title screen | Pokemon Red renders title, joypad works |
| 6 | CPU tests pass | blargg cpu_instrs all 11 subtests green |
| 7 | GBC color | Pokemon Crystal renders in color |
