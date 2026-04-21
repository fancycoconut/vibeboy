mod common;

macro_rules! sound_test {
    ($name:ident, $dir:expr, $path:expr) => {
        #[test]
        fn $name() {
            let out = common::run_sound_test(concat!("tests/", $dir, "/", $path));
            assert!(
                out.contains("Passed"),
                "sound test did not pass.\nSerial output:\n{out}"
            );
        }
    };
}

sound_test!(gbc_01_registers,            "gameboy_color_sound", "01-registers.gb");
sound_test!(gbc_02_len_ctr,              "gameboy_color_sound", "02-len ctr.gb");
sound_test!(gbc_03_trigger,              "gameboy_color_sound", "03-trigger.gb");
sound_test!(gbc_04_sweep,                "gameboy_color_sound", "04-sweep.gb");
sound_test!(gbc_05_sweep_details,        "gameboy_color_sound", "05-sweep details.gb");
sound_test!(gbc_06_overflow_on_trigger,  "gameboy_color_sound", "06-overflow on trigger.gb");
sound_test!(gbc_07_len_sweep_period,     "gameboy_color_sound", "07-len sweep period sync.gb");
sound_test!(gbc_08_len_ctr_power,        "gameboy_color_sound", "08-len ctr during power.gb");
sound_test!(gbc_09_wave_read,            "gameboy_color_sound", "09-wave read while on.gb");
sound_test!(gbc_10_wave_trigger,         "gameboy_color_sound", "10-wave trigger while on.gb");
sound_test!(gbc_11_regs_after_power,     "gameboy_color_sound", "11-regs after power.gb");
sound_test!(gbc_12_wave,                 "gameboy_color_sound", "12-wave.gb");

sound_test!(dmg_01_registers,            "dmg_sound", "01-registers.gb");
sound_test!(dmg_02_len_ctr,              "dmg_sound", "02-len ctr.gb");
sound_test!(dmg_03_trigger,              "dmg_sound", "03-trigger.gb");
sound_test!(dmg_04_sweep,                "dmg_sound", "04-sweep.gb");
sound_test!(dmg_05_sweep_details,        "dmg_sound", "05-sweep details.gb");
sound_test!(dmg_06_overflow_on_trigger,  "dmg_sound", "06-overflow on trigger.gb");
sound_test!(dmg_07_len_sweep_period,     "dmg_sound", "07-len sweep period sync.gb");
sound_test!(dmg_08_len_ctr_power,        "dmg_sound", "08-len ctr during power.gb");
sound_test!(dmg_09_wave_read,            "dmg_sound", "09-wave read while on.gb");
sound_test!(dmg_10_wave_trigger,         "dmg_sound", "10-wave trigger while on.gb");
sound_test!(dmg_11_regs_after_power,     "dmg_sound", "11-regs after power.gb");
sound_test!(dmg_12_wave,                 "dmg_sound", "12-wave write while on.gb");
