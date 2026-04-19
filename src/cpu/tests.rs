use super::{Cpu, Registers};
use crate::bus::Bus;

// ---------------------------------------------------------------------------
// Test harness
// ---------------------------------------------------------------------------

/// Build a minimal MBC0 ROM with `instructions` placed at 0x0100 (where PC
/// starts), then return a freshly-reset Cpu and a Bus backed by that ROM.
///
/// The CPU registers are zeroed (except SP=0xFFFE, PC=0x0100) so each test
/// starts from a predictable blank slate rather than the post-boot-ROM state.
fn setup(instructions: &[u8]) -> (Cpu, Bus) {
    let mut rom = vec![0u8; 0x8000]; // 32 KB — smallest valid MBC0 size
    rom[0x0147] = 0x00; // cartridge type: ROM only
    rom[0x0148] = 0x00; // ROM size: 32 KB
    rom[0x0149] = 0x00; // RAM size: none

    for (i, &b) in instructions.iter().enumerate() {
        rom[0x0100 + i] = b;
    }

    let mut cpu = Cpu::new(false);
    cpu.reg = Registers::default();
    cpu.reg.sp = 0xFFFE;
    cpu.reg.pc = 0x0100;

    let bus = Bus::new(rom);
    (cpu, bus)
}

/// Run exactly `n` instructions and return the cycle count of the last one.
fn run(cpu: &mut Cpu, bus: &mut Bus, n: usize) -> u8 {
    let mut cycles = 0;
    for _ in 0..n {
        cycles = cpu.step(bus);
    }
    cycles
}

// ---------------------------------------------------------------------------
// NOP
// ---------------------------------------------------------------------------

#[test]
fn nop_advances_pc_by_one() {
    let (mut cpu, mut bus) = setup(&[0x00]);
    let cycles = run(&mut cpu, &mut bus, 1);
    assert_eq!(cpu.reg.pc, 0x0101);
    assert_eq!(cycles, 1);
}

// ---------------------------------------------------------------------------
// 8-bit immediate loads
// ---------------------------------------------------------------------------

#[test]
fn ld_b_imm() {
    let (mut cpu, mut bus) = setup(&[0x06, 0x42]); // LD B, 0x42
    let cycles = run(&mut cpu, &mut bus, 1);
    assert_eq!(cpu.reg.b, 0x42);
    assert_eq!(cycles, 2);
}

#[test]
fn ld_all_immediates() {
    // LD B,1  LD C,2  LD D,3  LD E,4  LD H,5  LD L,6  LD A,7
    let (mut cpu, mut bus) = setup(&[
        0x06, 0x01,
        0x0E, 0x02,
        0x16, 0x03,
        0x1E, 0x04,
        0x26, 0x05,
        0x2E, 0x06,
        0x3E, 0x07,
    ]);
    run(&mut cpu, &mut bus, 7);
    assert_eq!(cpu.reg.b, 0x01);
    assert_eq!(cpu.reg.c, 0x02);
    assert_eq!(cpu.reg.d, 0x03);
    assert_eq!(cpu.reg.e, 0x04);
    assert_eq!(cpu.reg.h, 0x05);
    assert_eq!(cpu.reg.l, 0x06);
    assert_eq!(cpu.reg.a, 0x07);
}

// ---------------------------------------------------------------------------
// Register-to-register loads
// ---------------------------------------------------------------------------

#[test]
fn ld_r_r() {
    // LD A, 0x55 ; LD B, A
    let (mut cpu, mut bus) = setup(&[0x3E, 0x55, 0x47]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.b, 0x55);
}

// ---------------------------------------------------------------------------
// LD (HL) indirect
// ---------------------------------------------------------------------------

#[test]
fn ld_hl_indirect_write_and_read() {
    // LD HL, 0xC000 ; LD A, 0xAB ; LD (HL), A ; LD B, (HL)
    let (mut cpu, mut bus) = setup(&[
        0x21, 0x00, 0xC0, // LD HL, 0xC000
        0x3E, 0xAB,       // LD A, 0xAB
        0x77,             // LD (HL), A
        0x46,             // LD B, (HL)
    ]);
    run(&mut cpu, &mut bus, 4);
    assert_eq!(bus.read(0xC000), 0xAB);
    assert_eq!(cpu.reg.b, 0xAB);
}

#[test]
fn ld_hl_inc_dec() {
    // LD HL, 0xC000 ; LD A, 0x11 ; LD (HL+), A ; check HL became 0xC001
    let (mut cpu, mut bus) = setup(&[
        0x21, 0x00, 0xC0, // LD HL, 0xC000
        0x3E, 0x11,       // LD A, 0x11
        0x22,             // LD (HL+), A
    ]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(bus.read(0xC000), 0x11);
    assert_eq!(cpu.reg.hl(), 0xC001);

    // LD (HL-), A — HL should go back to 0xC000
    let (mut cpu, mut bus) = setup(&[
        0x21, 0x01, 0xC0, // LD HL, 0xC001
        0x3E, 0x22,       // LD A, 0x22
        0x32,             // LD (HL-), A
    ]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(bus.read(0xC001), 0x22);
    assert_eq!(cpu.reg.hl(), 0xC000);
}

// ---------------------------------------------------------------------------
// 16-bit loads
// ---------------------------------------------------------------------------

#[test]
fn ld_sp_and_push_pop() {
    // LD BC, 0x1234 ; PUSH BC ; POP DE
    let (mut cpu, mut bus) = setup(&[
        0x01, 0x34, 0x12, // LD BC, 0x1234
        0xC5,             // PUSH BC
        0xD1,             // POP DE
    ]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.reg.de(), 0x1234);
    assert_eq!(cpu.reg.sp, 0xFFFE); // SP back to original
}

#[test]
fn ld_hl_sp_plus_e() {
    // LD SP, 0xFF00 ; LD HL, SP+4
    let (mut cpu, mut bus) = setup(&[
        0x31, 0x00, 0xFF, // LD SP, 0xFF00
        0xF8, 0x04,       // LD HL, SP+4
    ]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.hl(), 0xFF04);
    assert!(!cpu.reg.flag_z());
    assert!(!cpu.reg.flag_n());
}

// ---------------------------------------------------------------------------
// 8-bit ALU — ADD
// ---------------------------------------------------------------------------

#[test]
fn add_basic() {
    // LD A, 0x10 ; ADD A, 0x20
    let (mut cpu, mut bus) = setup(&[0x3E, 0x10, 0xC6, 0x20]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x30);
    assert!(!cpu.reg.flag_z());
    assert!(!cpu.reg.flag_n());
    assert!(!cpu.reg.flag_h());
    assert!(!cpu.reg.flag_c());
}

#[test]
fn add_sets_zero_flag() {
    // LD A, 0x00 ; ADD A, 0x00
    let (mut cpu, mut bus) = setup(&[0x3E, 0x00, 0xC6, 0x00]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x00);
    assert!(cpu.reg.flag_z());
}

#[test]
fn add_sets_carry_flag() {
    // LD A, 0xFF ; ADD A, 0x01  → wraps to 0x00, carry set
    let (mut cpu, mut bus) = setup(&[0x3E, 0xFF, 0xC6, 0x01]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x00);
    assert!(cpu.reg.flag_z());
    assert!(cpu.reg.flag_c());
}

#[test]
fn add_sets_half_carry_flag() {
    // LD A, 0x0F ; ADD A, 0x01  → half-carry from nibble overflow
    let (mut cpu, mut bus) = setup(&[0x3E, 0x0F, 0xC6, 0x01]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x10);
    assert!(cpu.reg.flag_h());
    assert!(!cpu.reg.flag_c());
}

// ---------------------------------------------------------------------------
// ADC
// ---------------------------------------------------------------------------

#[test]
fn adc_includes_carry() {
    // LD A, 0x10 ; SCF ; ADC A, 0x01  → 0x10 + 0x01 + 1 = 0x12
    let (mut cpu, mut bus) = setup(&[0x3E, 0x10, 0x37, 0xCE, 0x01]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.reg.a, 0x12);
}

// ---------------------------------------------------------------------------
// SUB
// ---------------------------------------------------------------------------

#[test]
fn sub_basic() {
    // LD A, 0x30 ; SUB 0x10
    let (mut cpu, mut bus) = setup(&[0x3E, 0x30, 0xD6, 0x10]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x20);
    assert!(!cpu.reg.flag_z());
    assert!(cpu.reg.flag_n());
    assert!(!cpu.reg.flag_c());
}

#[test]
fn sub_sets_zero_flag() {
    // LD A, 0x42 ; SUB 0x42
    let (mut cpu, mut bus) = setup(&[0x3E, 0x42, 0xD6, 0x42]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x00);
    assert!(cpu.reg.flag_z());
    assert!(cpu.reg.flag_n());
}

#[test]
fn sub_sets_borrow_carry() {
    // LD A, 0x00 ; SUB 0x01  → underflow, carry (borrow) set
    let (mut cpu, mut bus) = setup(&[0x3E, 0x00, 0xD6, 0x01]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0xFF);
    assert!(cpu.reg.flag_c());
    assert!(cpu.reg.flag_n());
}

#[test]
fn sub_sets_half_borrow() {
    // LD A, 0x10 ; SUB 0x01  → half-carry (borrow from bit 4)
    let (mut cpu, mut bus) = setup(&[0x3E, 0x10, 0xD6, 0x01]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x0F);
    assert!(cpu.reg.flag_h());
}

// ---------------------------------------------------------------------------
// AND / OR / XOR
// ---------------------------------------------------------------------------

#[test]
fn and_result_and_flags() {
    // LD A, 0xFF ; AND 0x0F
    let (mut cpu, mut bus) = setup(&[0x3E, 0xFF, 0xE6, 0x0F]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x0F);
    assert!(!cpu.reg.flag_z());
    assert!(cpu.reg.flag_h()); // AND always sets H
    assert!(!cpu.reg.flag_c());
}

#[test]
fn and_zero_sets_z() {
    // LD A, 0xF0 ; AND 0x0F  → 0x00
    let (mut cpu, mut bus) = setup(&[0x3E, 0xF0, 0xE6, 0x0F]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x00);
    assert!(cpu.reg.flag_z());
}

#[test]
fn or_result_and_flags() {
    // LD A, 0xF0 ; OR 0x0F  → 0xFF
    let (mut cpu, mut bus) = setup(&[0x3E, 0xF0, 0xF6, 0x0F]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0xFF);
    assert!(!cpu.reg.flag_z());
    assert!(!cpu.reg.flag_h());
    assert!(!cpu.reg.flag_c());
}

#[test]
fn xor_self_is_zero() {
    // LD A, 0xAB ; XOR A  → 0x00, Z set
    let (mut cpu, mut bus) = setup(&[0x3E, 0xAB, 0xAF]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x00);
    assert!(cpu.reg.flag_z());
}

// ---------------------------------------------------------------------------
// CP (compare)
// ---------------------------------------------------------------------------

#[test]
fn cp_equal_sets_z() {
    // LD A, 0x10 ; CP 0x10
    let (mut cpu, mut bus) = setup(&[0x3E, 0x10, 0xFE, 0x10]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x10); // A unchanged
    assert!(cpu.reg.flag_z());
    assert!(cpu.reg.flag_n());
}

#[test]
fn cp_less_sets_carry() {
    // LD A, 0x05 ; CP 0x10  → A < operand, carry (borrow) set
    let (mut cpu, mut bus) = setup(&[0x3E, 0x05, 0xFE, 0x10]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x05);
    assert!(!cpu.reg.flag_z());
    assert!(cpu.reg.flag_c());
}

// ---------------------------------------------------------------------------
// INC / DEC (8-bit)
// ---------------------------------------------------------------------------

#[test]
fn inc_basic_and_half_carry() {
    // LD B, 0x0F ; INC B
    let (mut cpu, mut bus) = setup(&[0x06, 0x0F, 0x04]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.b, 0x10);
    assert!(cpu.reg.flag_h());
    assert!(!cpu.reg.flag_z());
    assert!(!cpu.reg.flag_n());
}

#[test]
fn inc_wraps_to_zero() {
    // LD B, 0xFF ; INC B  → 0x00, Z set
    let (mut cpu, mut bus) = setup(&[0x06, 0xFF, 0x04]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.b, 0x00);
    assert!(cpu.reg.flag_z());
}

#[test]
fn dec_basic_and_half_carry() {
    // LD C, 0x10 ; DEC C
    let (mut cpu, mut bus) = setup(&[0x0E, 0x10, 0x0D]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.c, 0x0F);
    assert!(cpu.reg.flag_h()); // borrow from bit 4
    assert!(cpu.reg.flag_n());
}

#[test]
fn dec_to_zero_sets_z() {
    // LD A, 0x01 ; DEC A
    let (mut cpu, mut bus) = setup(&[0x3E, 0x01, 0x3D]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x00);
    assert!(cpu.reg.flag_z());
    assert!(cpu.reg.flag_n());
}

// ---------------------------------------------------------------------------
// INC / DEC (16-bit) — no flag effects
// ---------------------------------------------------------------------------

#[test]
fn inc_dec_16bit() {
    // LD BC, 0x00FF ; INC BC  → 0x0100
    let (mut cpu, mut bus) = setup(&[0x01, 0xFF, 0x00, 0x03]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.bc(), 0x0100);

    // LD DE, 0x0001 ; DEC DE  → 0x0000
    let (mut cpu, mut bus) = setup(&[0x11, 0x01, 0x00, 0x1B]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.de(), 0x0000);
}

// ---------------------------------------------------------------------------
// ADD HL, rr
// ---------------------------------------------------------------------------

#[test]
fn add_hl_rr_carry_and_half_carry() {
    // LD HL, 0x0FFF ; ADD HL, HL  → 0x1FFE, H set
    let (mut cpu, mut bus) = setup(&[0x21, 0xFF, 0x0F, 0x29]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.hl(), 0x1FFE);
    assert!(cpu.reg.flag_h());
    assert!(!cpu.reg.flag_c());
    assert!(!cpu.reg.flag_n());
}

#[test]
fn add_hl_rr_overflow() {
    // LD HL, 0x8000 ; ADD HL, HL  → 0x0000, C set
    let (mut cpu, mut bus) = setup(&[0x21, 0x00, 0x80, 0x29]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.hl(), 0x0000);
    assert!(cpu.reg.flag_c());
}

// ---------------------------------------------------------------------------
// CPL / SCF / CCF
// ---------------------------------------------------------------------------

#[test]
fn cpl_complements_a_and_sets_flags() {
    // LD A, 0xAA ; CPL  → 0x55
    let (mut cpu, mut bus) = setup(&[0x3E, 0xAA, 0x2F]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x55);
    assert!(cpu.reg.flag_n());
    assert!(cpu.reg.flag_h());
}

#[test]
fn scf_sets_carry_clears_n_h() {
    let (mut cpu, mut bus) = setup(&[0x37]);
    // Pre-set N and H to verify they're cleared
    cpu.reg.set_flag_n(true);
    cpu.reg.set_flag_h(true);
    run(&mut cpu, &mut bus, 1);
    assert!(cpu.reg.flag_c());
    assert!(!cpu.reg.flag_n());
    assert!(!cpu.reg.flag_h());
}

#[test]
fn ccf_flips_carry() {
    // With carry clear → set
    let (mut cpu, mut bus) = setup(&[0x3F]);
    cpu.reg.set_flag_c(false);
    run(&mut cpu, &mut bus, 1);
    assert!(cpu.reg.flag_c());

    // With carry set → clear
    let (mut cpu, mut bus) = setup(&[0x3F]);
    cpu.reg.set_flag_c(true);
    run(&mut cpu, &mut bus, 1);
    assert!(!cpu.reg.flag_c());
}

// ---------------------------------------------------------------------------
// DAA
// ---------------------------------------------------------------------------

#[test]
fn daa_after_addition() {
    // LD A, 0x09 ; ADD A, 0x01 → 0x0A; DAA → 0x10 (BCD)
    let (mut cpu, mut bus) = setup(&[0x3E, 0x09, 0xC6, 0x01, 0x27]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.reg.a, 0x10);
    assert!(!cpu.reg.flag_c());
}

#[test]
fn daa_after_subtraction() {
    // LD A, 0x20 ; SUB 0x01 → 0x1F; DAA → 0x19 (BCD result of 20 - 01)
    let (mut cpu, mut bus) = setup(&[0x3E, 0x20, 0xD6, 0x01, 0x27]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.reg.a, 0x19);
}

// ---------------------------------------------------------------------------
// RLCA / RLA / RRCA / RRA
// ---------------------------------------------------------------------------

#[test]
fn rlca_rotates_left_through_bit7() {
    // LD A, 0x85 ; RLCA  → 0x0B, C=1
    let (mut cpu, mut bus) = setup(&[0x3E, 0x85, 0x07]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x0B);
    assert!(cpu.reg.flag_c());
    assert!(!cpu.reg.flag_z()); // RLCA always clears Z
}

#[test]
fn rrca_rotates_right_through_bit0() {
    // LD A, 0x01 ; RRCA  → 0x80, C=1
    let (mut cpu, mut bus) = setup(&[0x3E, 0x01, 0x0F]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x80);
    assert!(cpu.reg.flag_c());
}

#[test]
fn rla_rotates_through_carry() {
    // LD A, 0x80 ; SCF ; RLA  → 0x01, C=1 (old bit7 → new carry, old carry → bit0)
    let (mut cpu, mut bus) = setup(&[0x3E, 0x80, 0x37, 0x17]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.reg.a, 0x01);
    assert!(cpu.reg.flag_c());
}

#[test]
fn rra_rotates_through_carry() {
    // LD A, 0x01 ; SCF ; RRA  → 0x80, C=1
    let (mut cpu, mut bus) = setup(&[0x3E, 0x01, 0x37, 0x1F]);
    run(&mut cpu, &mut bus, 3);
    assert_eq!(cpu.reg.a, 0x80);
    assert!(cpu.reg.flag_c());
}

// ---------------------------------------------------------------------------
// Jumps — JR / JP
// ---------------------------------------------------------------------------

#[test]
fn jr_unconditional() {
    // JR +2 — skips the 0x00 (NOP) byte and lands on 0x3E (LD A, 0x55)
    // Layout: 0x0100=JR, 0x0101=+2, 0x0102=NOP(skipped), 0x0103=LD A,0x55, 0x0104=0x55
    let (mut cpu, mut bus) = setup(&[0x18, 0x01, 0x00, 0x3E, 0x55]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x55);
}

#[test]
fn jr_nz_taken_and_not_taken() {
    // JR NZ, +2  — taken when Z=0
    let (mut cpu, mut bus) = setup(&[0x20, 0x01, 0x00, 0x3E, 0x55]);
    cpu.reg.set_flag_z(false);
    run(&mut cpu, &mut bus, 2); // JR NZ (taken), then LD A,0x55
    assert_eq!(cpu.reg.a, 0x55);

    // JR NZ, +2  — NOT taken when Z=1, so next instruction is NOP(0x00)
    let (mut cpu, mut bus) = setup(&[0x20, 0x01, 0x00, 0x3E, 0x55]);
    cpu.reg.set_flag_z(true);
    run(&mut cpu, &mut bus, 2); // JR NZ (not taken), then NOP
    assert_eq!(cpu.reg.a, 0x00); // A untouched
}

#[test]
fn jp_absolute() {
    // JP 0x0104 (skipping one NOP at 0x0103), then LD A, 0xBB at 0x0104
    // 0x0100: C3 04 01   JP 0x0104
    // 0x0103: 00         NOP (skipped)
    // 0x0104: 3E BB      LD A, 0xBB
    let (mut cpu, mut bus) = setup(&[0xC3, 0x04, 0x01, 0x00, 0x3E, 0xBB]);
    run(&mut cpu, &mut bus, 2); // JP, then LD A
    assert_eq!(cpu.reg.a, 0xBB);
}

// ---------------------------------------------------------------------------
// CALL / RET
// ---------------------------------------------------------------------------

#[test]
fn call_and_ret() {
    // CALL 0x0105 ; NOP ; NOP ; NOP (pad); RET
    // 0x0100: CD 05 01   CALL 0x0105
    // 0x0103: 3E 77      LD A, 0x77  (executed after RET)
    // 0x0105: 3E AA      LD A, 0xAA  (subroutine sets A)
    // 0x0107: C9         RET
    let (mut cpu, mut bus) = setup(&[
        0xCD, 0x05, 0x01, // CALL 0x0105
        0x3E, 0x77,       // LD A, 0x77  (after return)
        0x3E, 0xAA,       // LD A, 0xAA  (subroutine)
        0xC9,             // RET
    ]);
    run(&mut cpu, &mut bus, 3); // CALL, LD A 0xAA, RET
    assert_eq!(cpu.reg.a, 0xAA);
    // PC should now point to 0x0103 (LD A, 0x77)
    assert_eq!(cpu.reg.pc, 0x0103);
    assert_eq!(cpu.reg.sp, 0xFFFE); // SP restored
}

// ---------------------------------------------------------------------------
// LDH (high page)
// ---------------------------------------------------------------------------

#[test]
fn ldh_store_and_load() {
    // LD A, 0x55 ; LDH (0x80), A ; LDH A, (0x80) via another reg path
    // 0xFF80 is HRAM — writable
    let (mut cpu, mut bus) = setup(&[
        0x3E, 0x55,       // LD A, 0x55
        0xE0, 0x80,       // LDH (0xFF80), A
        0x3E, 0x00,       // LD A, 0x00  — clear A
        0xF0, 0x80,       // LDH A, (0xFF80)
    ]);
    run(&mut cpu, &mut bus, 4);
    assert_eq!(cpu.reg.a, 0x55);
}

// ---------------------------------------------------------------------------
// CB prefix — BIT, SET, RES, rotates, shifts, SWAP
// ---------------------------------------------------------------------------

#[test]
fn cb_bit_sets_z_when_zero() {
    // LD B, 0b10101010 ; BIT 0, B  → bit 0 is 0, Z set
    let (mut cpu, mut bus) = setup(&[0x06, 0xAA, 0xCB, 0x40]);
    run(&mut cpu, &mut bus, 2);
    assert!(cpu.reg.flag_z());
    assert!(cpu.reg.flag_h());
    assert!(!cpu.reg.flag_n());
}

#[test]
fn cb_bit_clears_z_when_set() {
    // LD B, 0b10101010 ; BIT 1, B  → bit 1 is 1, Z clear
    let (mut cpu, mut bus) = setup(&[0x06, 0xAA, 0xCB, 0x48]);
    run(&mut cpu, &mut bus, 2);
    assert!(!cpu.reg.flag_z());
}

#[test]
fn cb_set_bit() {
    // LD C, 0x00 ; SET 3, C  → 0x08
    let (mut cpu, mut bus) = setup(&[0x0E, 0x00, 0xCB, 0xD9]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.c, 0x08);
}

#[test]
fn cb_res_bit() {
    // LD C, 0xFF ; RES 3, C  → 0xF7
    let (mut cpu, mut bus) = setup(&[0x0E, 0xFF, 0xCB, 0x99]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.c, 0xF7);
}

#[test]
fn cb_rlc() {
    // LD B, 0x80 ; RLC B  → 0x01, C=1
    let (mut cpu, mut bus) = setup(&[0x06, 0x80, 0xCB, 0x00]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.b, 0x01);
    assert!(cpu.reg.flag_c());
    assert!(!cpu.reg.flag_z());
}

#[test]
fn cb_rrc() {
    // LD D, 0x01 ; RRC D  → 0x80, C=1
    let (mut cpu, mut bus) = setup(&[0x16, 0x01, 0xCB, 0x0A]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.d, 0x80);
    assert!(cpu.reg.flag_c());
}

#[test]
fn cb_sla() {
    // LD E, 0x81 ; SLA E  → 0x02, C=1
    let (mut cpu, mut bus) = setup(&[0x1E, 0x81, 0xCB, 0x23]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.e, 0x02);
    assert!(cpu.reg.flag_c());
}

#[test]
fn cb_sra_preserves_sign() {
    // LD H, 0x80 ; SRA H  → 0xC0 (arithmetic right shift, sign extended)
    let (mut cpu, mut bus) = setup(&[0x26, 0x80, 0xCB, 0x2C]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.h, 0xC0);
    assert!(!cpu.reg.flag_c());
}

#[test]
fn cb_srl() {
    // LD L, 0x81 ; SRL L  → 0x40, C=1
    let (mut cpu, mut bus) = setup(&[0x2E, 0x81, 0xCB, 0x3D]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.l, 0x40);
    assert!(cpu.reg.flag_c());
}

#[test]
fn cb_swap() {
    // LD A, 0xAB ; SWAP A  → 0xBA
    let (mut cpu, mut bus) = setup(&[0x3E, 0xAB, 0xCB, 0x37]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0xBA);
    assert!(!cpu.reg.flag_z());
}

#[test]
fn cb_swap_zero() {
    // LD A, 0x00 ; SWAP A  → 0x00, Z set
    let (mut cpu, mut bus) = setup(&[0x3E, 0x00, 0xCB, 0x37]);
    run(&mut cpu, &mut bus, 2);
    assert_eq!(cpu.reg.a, 0x00);
    assert!(cpu.reg.flag_z());
}

// ---------------------------------------------------------------------------
// Interrupt dispatch
// ---------------------------------------------------------------------------

#[test]
fn interrupt_dispatch_jumps_to_vector() {
    // EI ; NOP  — after NOP the IME becomes true (deferred), then an interrupt
    // fires and the CPU should push PC and jump to 0x0040 (VBlank vector).
    let (mut cpu, mut bus) = setup(&[
        0xFB, // EI
        0x00, // NOP  (IME becomes true after this)
        0x00, // NOP  (this NOP should be interrupted)
    ]);
    bus.interrupts.enable = 0x01; // enable VBlank
    bus.interrupts.flags = 0x01;  // request VBlank

    // Step 1: EI executes — schedules IME for next step.
    // Step 2: IME becomes true, interrupt is pending → dispatch fires instead
    //         of executing the NOP. PC is pushed and jumps to 0x0040.
    run(&mut cpu, &mut bus, 2);

    assert_eq!(cpu.reg.pc, 0x0040); // jumped to VBlank vector
    assert!(!cpu.ime);              // IME cleared during ISR
    assert_eq!(bus.interrupts.flags, 0x00); // flag acknowledged
}
