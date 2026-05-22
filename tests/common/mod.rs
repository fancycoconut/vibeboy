use vibeboy::gameboy::GameBoy;

pub fn run_sound_test(rom_path: &str) -> String {
    let rom = std::fs::read(rom_path)
        .unwrap_or_else(|e| panic!("failed to read {rom_path}: {e}"));

    let mut gb = GameBoy::new(rom);

    // Blargg sound ROMs write their output to external RAM, not the serial port:
    //   0xA000: result sentinel — 0x80 while running, 0x00 = passed, N = failed test N
    //   0xA001–0xA003: fixed magic bytes written during init (0xDE, 0xB0, 0x61)
    //   0xA004+: null-terminated text output (test name + pass/fail message)
    //
    // We detect completion by watching 0xA000 change from the in-progress value
    // (0x80) to a final result code.
    let mut started = false;
    for _ in 0..200_000_000u64 {
        gb.step();
        let sentinel = gb.bus.read(0xA000);
        if sentinel == 0x80 {
            started = true;
        } else if started && sentinel != 0xFF {
            // Sentinel changed from 0x80 — test has finished.
            break;
        }
    }

    // Read null-terminated text from 0xA004.
    let mut out = Vec::new();
    for addr in 0xA004..=0xA0FFu16 {
        let b = gb.bus.read(addr);
        if b == 0 {
            break;
        }
        out.push(b);
    }

    String::from_utf8_lossy(&out).into_owned()
}
