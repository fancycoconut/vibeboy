# Vibeboy

Game Boy (DMG + GBC) emulator written in Rust.

## Requirements

- [Rust](https://rustup.rs/) (stable)
- A Game Boy ROM file (`.gb` / `.gbc`)

SDL2 is compiled from source automatically via the `bundled` feature — no system SDL2 installation needed.

## Building

```bash
cargo build --release
```

The first build takes a few minutes while SDL2 compiles. Subsequent builds are fast.

## Running

```bash
cargo run --release -- path/to/rom.gb
```

Or run the binary directly after building:

```bash
./target/release/vibeboy path/to/rom.gb
```

If no ROM path is given it defaults to `red.gb` in the current directory.

## Controls

| Key | Game Boy button |
|---|---|
| Arrow keys | D-pad |
| X | A |
| Z | B |
| Enter | Start |
| Shift | Select |
| Escape | Quit |

## Cartridge support

| MBC type | Cartridges |
|---|---|
| MBC0 (ROM only) | Tetris, Dr. Mario |
| MBC1 | Early titles |
| MBC3 | Pokemon Red/Blue, Gold/Silver |

## Project structure

```
src/
  main.rs           SDL2 window, event loop, framebuffer blit
  gameboy.rs        Top-level emulator — drives the step loop
  bus.rs            Memory bus — routes reads/writes across the full 64KB map
  cpu/
    mod.rs          LR35902 CPU, registers, interrupt dispatch
    instructions.rs All 512 opcodes (unprefixed + 0xCB prefix)
  ppu/
    mod.rs          Scanline PPU, 4-mode state machine, DMG palette
  cartridge/
    mod.rs          Cartridge trait, header parsing, factory
    mbc0.rs         ROM-only
    mbc1.rs         MBC1 bank switching
    mbc3.rs         MBC3 bank switching + RTC stub
  timer.rs          DIV / TIMA / TMA / TAC
  joypad.rs         Joypad register
  interrupts.rs     IF / IE registers
  apu/mod.rs        Audio stub (not yet implemented)
docs/
  plan.md           Full implementation plan and milestones
```

## Status

| Milestone | Status |
|---|---|
| Project compiles | Done |
| ROM header parsed, title logged | Done |
| CPU steps through instructions | Done |
| PPU scanline renderer | Done |
| SDL2 window + joypad input | Done |
| Timer + interrupts | Done |
| Pokemon Red title screen | In progress |
| blargg cpu_instrs tests pass | Upcoming |
| GBC color support | Upcoming |
