use vibeboy::gameboy::GameBoy;

/// Mooneye-style test runner.
///
/// These ROMs signal completion by halting the CPU with specific registers:
///   Passed: B=3, C=5, D=8, E=13, H=21, L=34  (Fibonacci sequence)
///   Failed: any other register state when halted
///
/// Returns `true` if the test passed, `false` otherwise.
fn run_mooneye(rom_path: &str) -> bool {
    let rom = std::fs::read(rom_path)
        .unwrap_or_else(|e| panic!("failed to read {rom_path}: {e}"));

    let mut gb = GameBoy::new(rom);

    for _ in 0..100_000_000u64 {
        gb.step();
        if gb.cpu.is_halted() {
            let r = &gb.cpu.reg;
            return r.b == 3 && r.c == 5 && r.d == 8 && r.e == 13 && r.h == 21 && r.l == 34;
        }
    }

    false // timed out
}

macro_rules! oam_bug_test {
    ($name:ident, $path:expr) => {
        #[test]
        #[ignore = "OAM bug not yet emulated"]
        fn $name() {
            assert!(
                run_mooneye(concat!("tests/oam_bug/", $path)),
                "mooneye OAM bug test did not pass: {}",
                $path
            );
        }
    };
}

oam_bug_test!(oam_bug_1_lcd_sync,       "1-lcd_sync.gb");
oam_bug_test!(oam_bug_2_causes,         "2-causes.gb");
oam_bug_test!(oam_bug_3_non_causes,     "3-non_causes.gb");
oam_bug_test!(oam_bug_4_scanline_timing,"4-scanline_timing.gb");
oam_bug_test!(oam_bug_5_timing_bug,     "5-timing_bug.gb");
oam_bug_test!(oam_bug_6_timing_no_bug,  "6-timing_no_bug.gb");
oam_bug_test!(oam_bug_7_timing_effect,  "7-timing_effect.gb");
oam_bug_test!(oam_bug_8_instr_effect,   "8-instr_effect.gb");
