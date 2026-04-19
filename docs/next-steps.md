# Next Steps

1. **CPU accuracy** — run blargg `cpu_instrs` test ROMs to find remaining gaps
2. **PPU verification** — confirm tile rendering is correct and get Pokemon Red / Gold title screens visible
3. **GBC HDMA** (`0xFF51–0xFF55`) — implement General Purpose and HBlank DMA; required by most GBC games to load tile/sprite data into VRAM
4. **MBC5** — covers Pokemon Crystal, Zelda Oracle games, Link's Awakening DX, and most other GBC-era titles
5. **Battery saves + RTC persistence** — write save RAM to `.sav` and RTC state (`origin`, `carry`) to `.rtc` alongside the ROM path so saves and the clock survive restarts
6. **GBC double-speed mode** (`KEY1` `0xFF4D`) — some GBC games switch the CPU to 8 MHz; timer and PPU dot timing need to respect the speed multiplier
7. **APU** — implement the four channels (pulse ×2, wave, noise) for audio
8. **OAM bug** — 8 blargg test ROMs are already present and ignored; emulate the DMG hardware corruption that occurs during OAM scan

## What's verified working

- All 11 blargg cpu_instrs subtests pass
- DMG + GBC PPU rendering, palettes, VRAM banking
- MBC0, MBC1, MBC3 — covers Pokemon Red/Gold
- Timer, joypad, interrupts, serial

## What still needs doing (in rough priority order)

1. GBC HDMA (0xFF51–0xFF55) — highest impact

The bus currently ignores all writes to these registers. Many GBC games (including Gold) use HDMA to DMA tile/sprite
data into VRAM during HBlank. Without it, the screen will be partially or fully garbled. Two modes to implement:
General Purpose DMA (one shot) and HBlank DMA (triggered each HBlank).

2. MBC5 — broad game compatibility

Covers a huge chunk of GBC-era games: Pokemon Crystal, Link's Awakening DX, Zelda Oracle games, etc. It's simpler than
  MBC3 (no RTC, no latch — just a 9-bit ROM bank register and 4-bit RAM bank). Without it, loading those ROMs panics.

3. Double-speed mode (KEY1 register 0xFF4D)

GBC games can switch the CPU to 2× speed (8 MHz instead of 4). The timer and PPU dot timing need to be aware of the
speed multiplier. Some games require it to function correctly.

4. APU / audio

The APU is a complete stub — silence. The four channels (pulse ×2, wave, noise) are well-documented. Most games work
without sound but it's a big part of the experience.

5. OAM bug (8 ignored tests)

The DMG has a hardware bug where OAM is corrupted when certain instructions access a 16-bit register during OAM scan
(mode 2). You already have the 8 test ROMs waiting — they're all ignored right now.

6. Battery saves (RAM persistence)

MBC1/MBC3 both have RAM but it's never saved to disk. The cartridge BATTERY flag means the save RAM should be
persisted to a .sav file alongside the ROM.