use vibeboy::gameboy::GameBoy;

/// Step the emulator until serial output contains "Passed" or "Failed",
/// or until `max_steps` instructions have executed. Returns the full
/// serial output collected during the run.
fn run_blargg(rom_path: &str) -> String {
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

macro_rules! blargg_test {
    ($name:ident, $path:expr) => {
        #[test]
        fn $name() {
            let out = run_blargg(concat!("tests/cpu/", $path));
            assert!(
                out.contains("Passed"),
                "blargg test did not pass.\nSerial output:\n{out}"
            );
        }
    };
}

blargg_test!(cpu_01_special,          "01-special.gb");
blargg_test!(cpu_02_interrupts,       "02-interrupts.gb");
blargg_test!(cpu_03_op_sp_hl,         "03-op sp,hl.gb");
blargg_test!(cpu_04_op_r_imm,         "04-op r,imm.gb");
blargg_test!(cpu_05_op_rp,            "05-op rp.gb");
blargg_test!(cpu_06_ld_r_r,           "06-ld r,r.gb");
blargg_test!(cpu_07_jr_jp_call_ret,   "07-jr,jp,call,ret,rst.gb");
blargg_test!(cpu_08_misc_instrs,      "08-misc instrs.gb");
blargg_test!(cpu_09_op_r_r,           "09-op r,r.gb");
blargg_test!(cpu_10_bit_ops,          "10-bit ops.gb");
blargg_test!(cpu_11_op_a_hl,          "11-op a,(hl).gb");
