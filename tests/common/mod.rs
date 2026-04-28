use vibeboy::gameboy::GameBoy;

pub fn run_sound_test(rom_path: &str) -> String {
    let rom = std::fs::read(rom_path)
        .unwrap_or_else(|e| panic!("failed to read {rom_path}: {e}"));

    let mut gb = GameBoy::new(rom);

    for _ in 0..200_000_000u64 {
        let pc = gb.cpu.reg.pc;
        gb.step();

        // Detect `JR -2` (0x18 0xFE) spin loop — the blargg completion/fail halt.
        // HALT (0x76) also keeps PC fixed but is a normal wait; skip it.
        if gb.bus.read(pc) == 0x18 && gb.bus.read(pc.wrapping_add(1)) == 0xFE {
            break;
        }

        // Also support ROMs that output via serial (cpu_instrs style)
        let out = std::str::from_utf8(&gb.bus.serial_buf).unwrap_or("");
        if out.contains("Passed") || out.contains("Failed") {
            break;
        }
    }

    // Check serial output first (cpu_instrs style)
    if !gb.bus.serial_buf.is_empty() {
        return String::from_utf8_lossy(&gb.bus.serial_buf).into_owned();
    }

    // Blargg dmg_sound/cgb_sound ROMs store result in cartridge RAM at 0xA000.
    // 0x00 = passed, non-zero = failed sub-test number.
    let result = gb.bus.read(0xA000);
    if result == 0x00 {
        "Passed".to_string()
    } else {
        format!("Failed #{:02}", result)
    }
}
