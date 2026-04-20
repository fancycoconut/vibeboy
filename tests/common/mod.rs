use vibeboy::gameboy::GameBoy;

pub fn run_sound_test(rom_path: &str) -> String {
    let rom = std::fs::read(rom_path)
        .unwrap_or_else(|e| panic!("failed to read {rom_path}: {e}"));

    let mut gb = GameBoy::new(rom);

    for _ in 0..200_000_000u64 {
        gb.step();
        let out = std::str::from_utf8(&gb.bus.serial_buf).unwrap_or("");
        if out.contains("Passed") || out.contains("Failed") {
            break;
        }
    }

    String::from_utf8_lossy(&gb.bus.serial_buf).into_owned()
}
