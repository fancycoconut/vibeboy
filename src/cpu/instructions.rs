/// Complete Sharp LR35902 instruction set.
///
/// Returns the number of machine cycles consumed.
use super::Cpu;
use crate::bus::Bus;

pub fn execute(cpu: &mut Cpu, bus: &mut Bus) -> u8 {
    let op = cpu.fetch8(bus);
    match op {
        // ====================================================================
        // Miscellaneous / Control
        // ====================================================================
        0x00 => 1, // NOP
        0x10 => { cpu.fetch8(bus); 1 } // STOP — treat as NOP for now
        0x76 => { cpu.halt(); 1 } // HALT
        0xF3 => { cpu.ime = false; 1 } // DI
        0xFB => { cpu.schedule_ei(); 1 } // EI
        0x27 => { op_daa(cpu); 1 } // DAA
        0x2F => { // CPL
            cpu.reg.a = !cpu.reg.a;
            cpu.reg.set_flag_n(true);
            cpu.reg.set_flag_h(true);
            1
        }
        0x37 => { // SCF
            cpu.reg.set_flag_n(false);
            cpu.reg.set_flag_h(false);
            cpu.reg.set_flag_c(true);
            1
        }
        0x3F => { // CCF
            let c = cpu.reg.flag_c();
            cpu.reg.set_flag_n(false);
            cpu.reg.set_flag_h(false);
            cpu.reg.set_flag_c(!c);
            1
        }

        // ====================================================================
        // 8-bit loads — LD r, r'
        // ====================================================================
        0x40 => 1, // LD B,B
        0x41 => { cpu.reg.b = cpu.reg.c; 1 }
        0x42 => { cpu.reg.b = cpu.reg.d; 1 }
        0x43 => { cpu.reg.b = cpu.reg.e; 1 }
        0x44 => { cpu.reg.b = cpu.reg.h; 1 }
        0x45 => { cpu.reg.b = cpu.reg.l; 1 }
        0x46 => { cpu.reg.b = bus.read(cpu.reg.hl()); 2 }
        0x47 => { cpu.reg.b = cpu.reg.a; 1 }
        0x48 => { cpu.reg.c = cpu.reg.b; 1 }
        0x49 => 1, // LD C,C
        0x4A => { cpu.reg.c = cpu.reg.d; 1 }
        0x4B => { cpu.reg.c = cpu.reg.e; 1 }
        0x4C => { cpu.reg.c = cpu.reg.h; 1 }
        0x4D => { cpu.reg.c = cpu.reg.l; 1 }
        0x4E => { cpu.reg.c = bus.read(cpu.reg.hl()); 2 }
        0x4F => { cpu.reg.c = cpu.reg.a; 1 }
        0x50 => { cpu.reg.d = cpu.reg.b; 1 }
        0x51 => { cpu.reg.d = cpu.reg.c; 1 }
        0x52 => 1, // LD D,D
        0x53 => { cpu.reg.d = cpu.reg.e; 1 }
        0x54 => { cpu.reg.d = cpu.reg.h; 1 }
        0x55 => { cpu.reg.d = cpu.reg.l; 1 }
        0x56 => { cpu.reg.d = bus.read(cpu.reg.hl()); 2 }
        0x57 => { cpu.reg.d = cpu.reg.a; 1 }
        0x58 => { cpu.reg.e = cpu.reg.b; 1 }
        0x59 => { cpu.reg.e = cpu.reg.c; 1 }
        0x5A => { cpu.reg.e = cpu.reg.d; 1 }
        0x5B => 1, // LD E,E
        0x5C => { cpu.reg.e = cpu.reg.h; 1 }
        0x5D => { cpu.reg.e = cpu.reg.l; 1 }
        0x5E => { cpu.reg.e = bus.read(cpu.reg.hl()); 2 }
        0x5F => { cpu.reg.e = cpu.reg.a; 1 }
        0x60 => { cpu.reg.h = cpu.reg.b; 1 }
        0x61 => { cpu.reg.h = cpu.reg.c; 1 }
        0x62 => { cpu.reg.h = cpu.reg.d; 1 }
        0x63 => { cpu.reg.h = cpu.reg.e; 1 }
        0x64 => 1, // LD H,H
        0x65 => { cpu.reg.h = cpu.reg.l; 1 }
        0x66 => { cpu.reg.h = bus.read(cpu.reg.hl()); 2 }
        0x67 => { cpu.reg.h = cpu.reg.a; 1 }
        0x68 => { cpu.reg.l = cpu.reg.b; 1 }
        0x69 => { cpu.reg.l = cpu.reg.c; 1 }
        0x6A => { cpu.reg.l = cpu.reg.d; 1 }
        0x6B => { cpu.reg.l = cpu.reg.e; 1 }
        0x6C => { cpu.reg.l = cpu.reg.h; 1 }
        0x6D => 1, // LD L,L
        0x6E => { cpu.reg.l = bus.read(cpu.reg.hl()); 2 }
        0x6F => { cpu.reg.l = cpu.reg.a; 1 }
        0x70 => { bus.write(cpu.reg.hl(), cpu.reg.b); 2 }
        0x71 => { bus.write(cpu.reg.hl(), cpu.reg.c); 2 }
        0x72 => { bus.write(cpu.reg.hl(), cpu.reg.d); 2 }
        0x73 => { bus.write(cpu.reg.hl(), cpu.reg.e); 2 }
        0x74 => { bus.write(cpu.reg.hl(), cpu.reg.h); 2 }
        0x75 => { bus.write(cpu.reg.hl(), cpu.reg.l); 2 }
        0x77 => { bus.write(cpu.reg.hl(), cpu.reg.a); 2 }
        0x78 => { cpu.reg.a = cpu.reg.b; 1 }
        0x79 => { cpu.reg.a = cpu.reg.c; 1 }
        0x7A => { cpu.reg.a = cpu.reg.d; 1 }
        0x7B => { cpu.reg.a = cpu.reg.e; 1 }
        0x7C => { cpu.reg.a = cpu.reg.h; 1 }
        0x7D => { cpu.reg.a = cpu.reg.l; 1 }
        0x7E => { cpu.reg.a = bus.read(cpu.reg.hl()); 2 }
        0x7F => 1, // LD A,A

        // ====================================================================
        // 8-bit loads — LD r, n (immediate)
        // ====================================================================
        0x06 => { cpu.reg.b = cpu.fetch8(bus); 2 }
        0x0E => { cpu.reg.c = cpu.fetch8(bus); 2 }
        0x16 => { cpu.reg.d = cpu.fetch8(bus); 2 }
        0x1E => { cpu.reg.e = cpu.fetch8(bus); 2 }
        0x26 => { cpu.reg.h = cpu.fetch8(bus); 2 }
        0x2E => { cpu.reg.l = cpu.fetch8(bus); 2 }
        0x36 => { let n = cpu.fetch8(bus); bus.write(cpu.reg.hl(), n); 3 }
        0x3E => { cpu.reg.a = cpu.fetch8(bus); 2 }

        // ====================================================================
        // 8-bit loads — indirect
        // ====================================================================
        0x02 => { bus.write(cpu.reg.bc(), cpu.reg.a); 2 }
        0x12 => { bus.write(cpu.reg.de(), cpu.reg.a); 2 }
        0x0A => { cpu.reg.a = bus.read(cpu.reg.bc()); 2 }
        0x1A => { cpu.reg.a = bus.read(cpu.reg.de()); 2 }
        0x22 => { // LD (HL+), A
            bus.write(cpu.reg.hl(), cpu.reg.a);
            let hl = cpu.reg.hl().wrapping_add(1);
            cpu.reg.set_hl(hl);
            2
        }
        0x2A => { // LD A, (HL+)
            cpu.reg.a = bus.read(cpu.reg.hl());
            let hl = cpu.reg.hl().wrapping_add(1);
            cpu.reg.set_hl(hl);
            2
        }
        0x32 => { // LD (HL-), A
            bus.write(cpu.reg.hl(), cpu.reg.a);
            let hl = cpu.reg.hl().wrapping_sub(1);
            cpu.reg.set_hl(hl);
            2
        }
        0x3A => { // LD A, (HL-)
            cpu.reg.a = bus.read(cpu.reg.hl());
            let hl = cpu.reg.hl().wrapping_sub(1);
            cpu.reg.set_hl(hl);
            2
        }
        0xE0 => { // LDH (n), A  — store A into 0xFF00+n
            let n = cpu.fetch8(bus) as u16;
            bus.write(0xFF00 | n, cpu.reg.a);
            3
        }
        0xF0 => { // LDH A, (n)
            let n = cpu.fetch8(bus) as u16;
            cpu.reg.a = bus.read(0xFF00 | n);
            3
        }
        0xE2 => { bus.write(0xFF00 | cpu.reg.c as u16, cpu.reg.a); 2 }
        0xF2 => { cpu.reg.a = bus.read(0xFF00 | cpu.reg.c as u16); 2 }
        0xEA => { let a = cpu.fetch16(bus); bus.write(a, cpu.reg.a); 4 }
        0xFA => { let a = cpu.fetch16(bus); cpu.reg.a = bus.read(a); 4 }

        // ====================================================================
        // 16-bit loads
        // ====================================================================
        0x01 => { let n = cpu.fetch16(bus); cpu.reg.set_bc(n); 3 }
        0x11 => { let n = cpu.fetch16(bus); cpu.reg.set_de(n); 3 }
        0x21 => { let n = cpu.fetch16(bus); cpu.reg.set_hl(n); 3 }
        0x31 => { cpu.reg.sp = cpu.fetch16(bus); 3 }
        0x08 => { // LD (nn), SP
            let a = cpu.fetch16(bus);
            bus.write16(a, cpu.reg.sp);
            5
        }
        0xF9 => { cpu.reg.sp = cpu.reg.hl(); 2 } // LD SP, HL
        0xF8 => { // LD HL, SP+e
            let e = cpu.fetch8(bus) as i8 as i32;
            let sp = cpu.reg.sp as i32;
            let result = sp.wrapping_add(e);
            cpu.reg.set_hl(result as u16);
            cpu.reg.set_flags(false, false,
                (sp ^ e ^ result) & 0x10 != 0,
                (sp ^ e ^ result) & 0x100 != 0);
            3
        }

        // PUSH / POP
        0xC5 => { cpu.push16(bus, cpu.reg.bc()); 4 }
        0xD5 => { cpu.push16(bus, cpu.reg.de()); 4 }
        0xE5 => { cpu.push16(bus, cpu.reg.hl()); 4 }
        0xF5 => { cpu.push16(bus, cpu.reg.af()); 4 }
        0xC1 => { let v = cpu.pop16(bus); cpu.reg.set_bc(v); 3 }
        0xD1 => { let v = cpu.pop16(bus); cpu.reg.set_de(v); 3 }
        0xE1 => { let v = cpu.pop16(bus); cpu.reg.set_hl(v); 3 }
        0xF1 => { let v = cpu.pop16(bus); cpu.reg.set_af(v); 3 }

        // ====================================================================
        // 8-bit ALU
        // ====================================================================

        // ADD A, r
        0x80 => { op_add(cpu, cpu.reg.b); 1 }
        0x81 => { op_add(cpu, cpu.reg.c); 1 }
        0x82 => { op_add(cpu, cpu.reg.d); 1 }
        0x83 => { op_add(cpu, cpu.reg.e); 1 }
        0x84 => { op_add(cpu, cpu.reg.h); 1 }
        0x85 => { op_add(cpu, cpu.reg.l); 1 }
        0x86 => { let v = bus.read(cpu.reg.hl()); op_add(cpu, v); 2 }
        0x87 => { op_add(cpu, cpu.reg.a); 1 }
        0xC6 => { let n = cpu.fetch8(bus); op_add(cpu, n); 2 }

        // ADC A, r
        0x88 => { op_adc(cpu, cpu.reg.b); 1 }
        0x89 => { op_adc(cpu, cpu.reg.c); 1 }
        0x8A => { op_adc(cpu, cpu.reg.d); 1 }
        0x8B => { op_adc(cpu, cpu.reg.e); 1 }
        0x8C => { op_adc(cpu, cpu.reg.h); 1 }
        0x8D => { op_adc(cpu, cpu.reg.l); 1 }
        0x8E => { let v = bus.read(cpu.reg.hl()); op_adc(cpu, v); 2 }
        0x8F => { op_adc(cpu, cpu.reg.a); 1 }
        0xCE => { let n = cpu.fetch8(bus); op_adc(cpu, n); 2 }

        // SUB r
        0x90 => { op_sub(cpu, cpu.reg.b); 1 }
        0x91 => { op_sub(cpu, cpu.reg.c); 1 }
        0x92 => { op_sub(cpu, cpu.reg.d); 1 }
        0x93 => { op_sub(cpu, cpu.reg.e); 1 }
        0x94 => { op_sub(cpu, cpu.reg.h); 1 }
        0x95 => { op_sub(cpu, cpu.reg.l); 1 }
        0x96 => { let v = bus.read(cpu.reg.hl()); op_sub(cpu, v); 2 }
        0x97 => { op_sub(cpu, cpu.reg.a); 1 }
        0xD6 => { let n = cpu.fetch8(bus); op_sub(cpu, n); 2 }

        // SBC A, r
        0x98 => { op_sbc(cpu, cpu.reg.b); 1 }
        0x99 => { op_sbc(cpu, cpu.reg.c); 1 }
        0x9A => { op_sbc(cpu, cpu.reg.d); 1 }
        0x9B => { op_sbc(cpu, cpu.reg.e); 1 }
        0x9C => { op_sbc(cpu, cpu.reg.h); 1 }
        0x9D => { op_sbc(cpu, cpu.reg.l); 1 }
        0x9E => { let v = bus.read(cpu.reg.hl()); op_sbc(cpu, v); 2 }
        0x9F => { op_sbc(cpu, cpu.reg.a); 1 }
        0xDE => { let n = cpu.fetch8(bus); op_sbc(cpu, n); 2 }

        // AND r
        0xA0 => { op_and(cpu, cpu.reg.b); 1 }
        0xA1 => { op_and(cpu, cpu.reg.c); 1 }
        0xA2 => { op_and(cpu, cpu.reg.d); 1 }
        0xA3 => { op_and(cpu, cpu.reg.e); 1 }
        0xA4 => { op_and(cpu, cpu.reg.h); 1 }
        0xA5 => { op_and(cpu, cpu.reg.l); 1 }
        0xA6 => { let v = bus.read(cpu.reg.hl()); op_and(cpu, v); 2 }
        0xA7 => { op_and(cpu, cpu.reg.a); 1 }
        0xE6 => { let n = cpu.fetch8(bus); op_and(cpu, n); 2 }

        // XOR r
        0xA8 => { op_xor(cpu, cpu.reg.b); 1 }
        0xA9 => { op_xor(cpu, cpu.reg.c); 1 }
        0xAA => { op_xor(cpu, cpu.reg.d); 1 }
        0xAB => { op_xor(cpu, cpu.reg.e); 1 }
        0xAC => { op_xor(cpu, cpu.reg.h); 1 }
        0xAD => { op_xor(cpu, cpu.reg.l); 1 }
        0xAE => { let v = bus.read(cpu.reg.hl()); op_xor(cpu, v); 2 }
        0xAF => { op_xor(cpu, cpu.reg.a); 1 }
        0xEE => { let n = cpu.fetch8(bus); op_xor(cpu, n); 2 }

        // OR r
        0xB0 => { op_or(cpu, cpu.reg.b); 1 }
        0xB1 => { op_or(cpu, cpu.reg.c); 1 }
        0xB2 => { op_or(cpu, cpu.reg.d); 1 }
        0xB3 => { op_or(cpu, cpu.reg.e); 1 }
        0xB4 => { op_or(cpu, cpu.reg.h); 1 }
        0xB5 => { op_or(cpu, cpu.reg.l); 1 }
        0xB6 => { let v = bus.read(cpu.reg.hl()); op_or(cpu, v); 2 }
        0xB7 => { op_or(cpu, cpu.reg.a); 1 }
        0xF6 => { let n = cpu.fetch8(bus); op_or(cpu, n); 2 }

        // CP r  (compare — SUB without storing result)
        0xB8 => { op_cp(cpu, cpu.reg.b); 1 }
        0xB9 => { op_cp(cpu, cpu.reg.c); 1 }
        0xBA => { op_cp(cpu, cpu.reg.d); 1 }
        0xBB => { op_cp(cpu, cpu.reg.e); 1 }
        0xBC => { op_cp(cpu, cpu.reg.h); 1 }
        0xBD => { op_cp(cpu, cpu.reg.l); 1 }
        0xBE => { let v = bus.read(cpu.reg.hl()); op_cp(cpu, v); 2 }
        0xBF => { op_cp(cpu, cpu.reg.a); 1 }
        0xFE => { let n = cpu.fetch8(bus); op_cp(cpu, n); 2 }

        // INC r
        0x04 => { cpu.reg.b = op_inc(cpu, cpu.reg.b); 1 }
        0x0C => { cpu.reg.c = op_inc(cpu, cpu.reg.c); 1 }
        0x14 => { cpu.reg.d = op_inc(cpu, cpu.reg.d); 1 }
        0x1C => { cpu.reg.e = op_inc(cpu, cpu.reg.e); 1 }
        0x24 => { cpu.reg.h = op_inc(cpu, cpu.reg.h); 1 }
        0x2C => { cpu.reg.l = op_inc(cpu, cpu.reg.l); 1 }
        0x34 => {
            let v = bus.read(cpu.reg.hl());
            let r = op_inc(cpu, v);
            bus.write(cpu.reg.hl(), r);
            3
        }
        0x3C => { cpu.reg.a = op_inc(cpu, cpu.reg.a); 1 }

        // DEC r
        0x05 => { cpu.reg.b = op_dec(cpu, cpu.reg.b); 1 }
        0x0D => { cpu.reg.c = op_dec(cpu, cpu.reg.c); 1 }
        0x15 => { cpu.reg.d = op_dec(cpu, cpu.reg.d); 1 }
        0x1D => { cpu.reg.e = op_dec(cpu, cpu.reg.e); 1 }
        0x25 => { cpu.reg.h = op_dec(cpu, cpu.reg.h); 1 }
        0x2D => { cpu.reg.l = op_dec(cpu, cpu.reg.l); 1 }
        0x35 => {
            let v = bus.read(cpu.reg.hl());
            let r = op_dec(cpu, v);
            bus.write(cpu.reg.hl(), r);
            3
        }
        0x3D => { cpu.reg.a = op_dec(cpu, cpu.reg.a); 1 }

        // ====================================================================
        // 16-bit ALU
        // ====================================================================
        0x03 => { cpu.reg.set_bc(cpu.reg.bc().wrapping_add(1)); 2 }
        0x13 => { cpu.reg.set_de(cpu.reg.de().wrapping_add(1)); 2 }
        0x23 => { cpu.reg.set_hl(cpu.reg.hl().wrapping_add(1)); 2 }
        0x33 => { cpu.reg.sp = cpu.reg.sp.wrapping_add(1); 2 }
        0x0B => { cpu.reg.set_bc(cpu.reg.bc().wrapping_sub(1)); 2 }
        0x1B => { cpu.reg.set_de(cpu.reg.de().wrapping_sub(1)); 2 }
        0x2B => { cpu.reg.set_hl(cpu.reg.hl().wrapping_sub(1)); 2 }
        0x3B => { cpu.reg.sp = cpu.reg.sp.wrapping_sub(1); 2 }

        // ADD HL, rr
        0x09 => { op_add_hl(cpu, cpu.reg.bc()); 2 }
        0x19 => { op_add_hl(cpu, cpu.reg.de()); 2 }
        0x29 => { op_add_hl(cpu, cpu.reg.hl()); 2 }
        0x39 => { op_add_hl(cpu, cpu.reg.sp); 2 }

        // ADD SP, e
        0xE8 => {
            let e = cpu.fetch8(bus) as i8 as i32;
            let sp = cpu.reg.sp as i32;
            let result = sp.wrapping_add(e);
            cpu.reg.sp = result as u16;
            cpu.reg.set_flags(false, false,
                (sp ^ e ^ result) & 0x10 != 0,
                (sp ^ e ^ result) & 0x100 != 0);
            4
        }

        // ====================================================================
        // Rotates / Shifts (A register, fast versions)
        // ====================================================================
        0x07 => { // RLCA
            let c = cpu.reg.a >> 7;
            cpu.reg.a = (cpu.reg.a << 1) | c;
            cpu.reg.set_flags(false, false, false, c != 0);
            1
        }
        0x17 => { // RLA
            let old_c = cpu.reg.flag_c() as u8;
            let c = cpu.reg.a >> 7;
            cpu.reg.a = (cpu.reg.a << 1) | old_c;
            cpu.reg.set_flags(false, false, false, c != 0);
            1
        }
        0x0F => { // RRCA
            let c = cpu.reg.a & 1;
            cpu.reg.a = (cpu.reg.a >> 1) | (c << 7);
            cpu.reg.set_flags(false, false, false, c != 0);
            1
        }
        0x1F => { // RRA
            let old_c = cpu.reg.flag_c() as u8;
            let c = cpu.reg.a & 1;
            cpu.reg.a = (cpu.reg.a >> 1) | (old_c << 7);
            cpu.reg.set_flags(false, false, false, c != 0);
            1
        }

        // ====================================================================
        // Jumps
        // ====================================================================
        0xC3 => { cpu.reg.pc = cpu.fetch16(bus); 4 } // JP nn
        0xE9 => { cpu.reg.pc = cpu.reg.hl(); 1 }      // JP HL

        0xC2 => { // JP NZ, nn
            let a = cpu.fetch16(bus);
            if !cpu.reg.flag_z() { cpu.reg.pc = a; 4 } else { 3 }
        }
        0xCA => { // JP Z, nn
            let a = cpu.fetch16(bus);
            if cpu.reg.flag_z() { cpu.reg.pc = a; 4 } else { 3 }
        }
        0xD2 => { // JP NC, nn
            let a = cpu.fetch16(bus);
            if !cpu.reg.flag_c() { cpu.reg.pc = a; 4 } else { 3 }
        }
        0xDA => { // JP C, nn
            let a = cpu.fetch16(bus);
            if cpu.reg.flag_c() { cpu.reg.pc = a; 4 } else { 3 }
        }

        0x18 => { // JR e
            let e = cpu.fetch8(bus) as i8 as i16;
            cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(e);
            3
        }
        0x20 => { // JR NZ, e
            let e = cpu.fetch8(bus) as i8 as i16;
            if !cpu.reg.flag_z() { cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(e); 3 } else { 2 }
        }
        0x28 => { // JR Z, e
            let e = cpu.fetch8(bus) as i8 as i16;
            if cpu.reg.flag_z() { cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(e); 3 } else { 2 }
        }
        0x30 => { // JR NC, e
            let e = cpu.fetch8(bus) as i8 as i16;
            if !cpu.reg.flag_c() { cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(e); 3 } else { 2 }
        }
        0x38 => { // JR C, e
            let e = cpu.fetch8(bus) as i8 as i16;
            if cpu.reg.flag_c() { cpu.reg.pc = cpu.reg.pc.wrapping_add_signed(e); 3 } else { 2 }
        }

        // ====================================================================
        // Calls / Returns
        // ====================================================================
        0xCD => { // CALL nn
            let a = cpu.fetch16(bus);
            cpu.push16(bus, cpu.reg.pc);
            cpu.reg.pc = a;
            6
        }
        0xC4 => { // CALL NZ, nn
            let a = cpu.fetch16(bus);
            if !cpu.reg.flag_z() { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = a; 6 } else { 3 }
        }
        0xCC => { // CALL Z, nn
            let a = cpu.fetch16(bus);
            if cpu.reg.flag_z() { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = a; 6 } else { 3 }
        }
        0xD4 => { // CALL NC, nn
            let a = cpu.fetch16(bus);
            if !cpu.reg.flag_c() { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = a; 6 } else { 3 }
        }
        0xDC => { // CALL C, nn
            let a = cpu.fetch16(bus);
            if cpu.reg.flag_c() { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = a; 6 } else { 3 }
        }

        0xC9 => { // RET
            cpu.reg.pc = cpu.pop16(bus);
            4
        }
        0xD9 => { // RETI
            cpu.reg.pc = cpu.pop16(bus);
            cpu.ime = true;
            4
        }
        0xC0 => { // RET NZ
            if !cpu.reg.flag_z() { cpu.reg.pc = cpu.pop16(bus); 5 } else { 2 }
        }
        0xC8 => { // RET Z
            if cpu.reg.flag_z() { cpu.reg.pc = cpu.pop16(bus); 5 } else { 2 }
        }
        0xD0 => { // RET NC
            if !cpu.reg.flag_c() { cpu.reg.pc = cpu.pop16(bus); 5 } else { 2 }
        }
        0xD8 => { // RET C
            if cpu.reg.flag_c() { cpu.reg.pc = cpu.pop16(bus); 5 } else { 2 }
        }

        // RST
        0xC7 => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x00; 4 }
        0xCF => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x08; 4 }
        0xD7 => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x10; 4 }
        0xDF => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x18; 4 }
        0xE7 => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x20; 4 }
        0xEF => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x28; 4 }
        0xF7 => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x30; 4 }
        0xFF => { cpu.push16(bus, cpu.reg.pc); cpu.reg.pc = 0x38; 4 }

        // ====================================================================
        // CB prefix — extended bit operations
        // ====================================================================
        0xCB => execute_cb(cpu, bus),

        op => {
            eprintln!("[CPU] Unimplemented opcode: 0x{op:02X} at PC=0x{:04X}", cpu.reg.pc.wrapping_sub(1));
            1
        }
    }
}

// ============================================================================
// CB-prefixed instructions (bit manipulation, rotates, shifts)
// ============================================================================

fn execute_cb(cpu: &mut Cpu, bus: &mut Bus) -> u8 {
    let op = cpu.fetch8(bus);

    // Decode operand index: 0=B 1=C 2=D 3=E 4=H 5=L 6=(HL) 7=A
    let reg_idx = op & 0x07;
    let is_hl = reg_idx == 6;

    let val = if is_hl {
        bus.read(cpu.reg.hl())
    } else {
        get_reg(cpu, reg_idx)
    };

    let (result, extra_cycles) = match op >> 3 {
        0x00 => (cb_rlc(cpu, val), 0),   // RLC
        0x01 => (cb_rrc(cpu, val), 0),   // RRC
        0x02 => (cb_rl(cpu, val), 0),    // RL
        0x03 => (cb_rr(cpu, val), 0),    // RR
        0x04 => (cb_sla(cpu, val), 0),   // SLA
        0x05 => (cb_sra(cpu, val), 0),   // SRA
        0x06 => (cb_swap(cpu, val), 0),  // SWAP
        0x07 => (cb_srl(cpu, val), 0),   // SRL
        0x08..=0x0F => { // BIT b, r — test bit
            let bit = (op >> 3) - 0x08;
            cb_bit(cpu, val, bit);
            (val, 0) // BIT doesn't write back
        }
        0x10..=0x17 => { // RES b, r
            let bit = (op >> 3) - 0x10;
            (val & !(1 << bit), 0)
        }
        0x18..=0x1F => { // SET b, r
            let bit = (op >> 3) - 0x18;
            (val | (1 << bit), 0)
        }
        _ => unreachable!(),
    };

    // BIT instructions don't write back
    let is_bit = (op >> 6) == 0x01;
    if !is_bit {
        if is_hl {
            bus.write(cpu.reg.hl(), result);
        } else {
            set_reg(cpu, reg_idx, result);
        }
    }

    let base_cycles = if is_hl { 4 } else { 2 };
    // BIT (HL) is 3, others with (HL) are 4
    let cycles = if is_hl && is_bit { 3 } else { base_cycles + extra_cycles };
    cycles
}

// ============================================================================
// Register index helpers (0=B 1=C 2=D 3=E 4=H 5=L 6=(HL) 7=A)
// ============================================================================

fn get_reg(cpu: &Cpu, idx: u8) -> u8 {
    match idx {
        0 => cpu.reg.b,
        1 => cpu.reg.c,
        2 => cpu.reg.d,
        3 => cpu.reg.e,
        4 => cpu.reg.h,
        5 => cpu.reg.l,
        7 => cpu.reg.a,
        _ => 0,
    }
}

fn set_reg(cpu: &mut Cpu, idx: u8, val: u8) {
    match idx {
        0 => cpu.reg.b = val,
        1 => cpu.reg.c = val,
        2 => cpu.reg.d = val,
        3 => cpu.reg.e = val,
        4 => cpu.reg.h = val,
        5 => cpu.reg.l = val,
        7 => cpu.reg.a = val,
        _ => {}
    }
}

// ============================================================================
// ALU helpers
// ============================================================================

fn op_add(cpu: &mut Cpu, val: u8) {
    let a = cpu.reg.a;
    let result = a.wrapping_add(val);
    cpu.reg.set_flags(
        result == 0,
        false,
        (a & 0xF) + (val & 0xF) > 0xF,
        (a as u16) + (val as u16) > 0xFF,
    );
    cpu.reg.a = result;
}

fn op_adc(cpu: &mut Cpu, val: u8) {
    let a = cpu.reg.a;
    let c = cpu.reg.flag_c() as u8;
    let result = a.wrapping_add(val).wrapping_add(c);
    cpu.reg.set_flags(
        result == 0,
        false,
        (a & 0xF) + (val & 0xF) + c > 0xF,
        (a as u16) + (val as u16) + (c as u16) > 0xFF,
    );
    cpu.reg.a = result;
}

fn op_sub(cpu: &mut Cpu, val: u8) {
    let a = cpu.reg.a;
    let result = a.wrapping_sub(val);
    cpu.reg.set_flags(
        result == 0,
        true,
        (a & 0xF) < (val & 0xF),
        (a as u16) < (val as u16),
    );
    cpu.reg.a = result;
}

fn op_sbc(cpu: &mut Cpu, val: u8) {
    let a = cpu.reg.a;
    let c = cpu.reg.flag_c() as u8;
    let result = a.wrapping_sub(val).wrapping_sub(c);
    cpu.reg.set_flags(
        result == 0,
        true,
        (a & 0xF) < (val & 0xF) + c,
        (a as u16) < (val as u16) + (c as u16),
    );
    cpu.reg.a = result;
}

fn op_and(cpu: &mut Cpu, val: u8) {
    cpu.reg.a &= val;
    cpu.reg.set_flags(cpu.reg.a == 0, false, true, false);
}

fn op_or(cpu: &mut Cpu, val: u8) {
    cpu.reg.a |= val;
    cpu.reg.set_flags(cpu.reg.a == 0, false, false, false);
}

fn op_xor(cpu: &mut Cpu, val: u8) {
    cpu.reg.a ^= val;
    cpu.reg.set_flags(cpu.reg.a == 0, false, false, false);
}

fn op_cp(cpu: &mut Cpu, val: u8) {
    let a = cpu.reg.a;
    cpu.reg.set_flags(
        a == val,
        true,
        (a & 0xF) < (val & 0xF),
        (a as u16) < (val as u16),
    );
}

fn op_inc(cpu: &mut Cpu, val: u8) -> u8 {
    let result = val.wrapping_add(1);
    let h = (val & 0xF) == 0xF;
    cpu.reg.set_flag_z(result == 0);
    cpu.reg.set_flag_n(false);
    cpu.reg.set_flag_h(h);
    result
}

fn op_dec(cpu: &mut Cpu, val: u8) -> u8 {
    let result = val.wrapping_sub(1);
    let h = (val & 0xF) == 0;
    cpu.reg.set_flag_z(result == 0);
    cpu.reg.set_flag_n(true);
    cpu.reg.set_flag_h(h);
    result
}

fn op_add_hl(cpu: &mut Cpu, val: u16) {
    let hl = cpu.reg.hl();
    let result = hl.wrapping_add(val);
    cpu.reg.set_flag_n(false);
    cpu.reg.set_flag_h((hl & 0xFFF) + (val & 0xFFF) > 0xFFF);
    cpu.reg.set_flag_c((hl as u32) + (val as u32) > 0xFFFF);
    cpu.reg.set_hl(result);
}

fn op_daa(cpu: &mut Cpu) {
    let mut a = cpu.reg.a as i32;
    if !cpu.reg.flag_n() {
        // After addition: correct lower and upper BCD digits
        if cpu.reg.flag_h() || (a & 0x0F) > 0x09 { a += 0x06; }
        let carry = cpu.reg.flag_c() || a > 0x9F;
        if carry { a += 0x60; }
        // In the addition path carry is explicitly set or cleared
        cpu.reg.set_flag_c(carry);
    } else {
        // After subtraction: carry is preserved as-is, only adjust value
        if cpu.reg.flag_h() { a -= 0x06; }
        if cpu.reg.flag_c() { a -= 0x60; }
    }
    cpu.reg.a = a as u8;
    cpu.reg.set_flag_h(false);
    cpu.reg.set_flag_z(cpu.reg.a == 0);
}

// ============================================================================
// CB rotate/shift helpers
// ============================================================================

fn cb_rlc(cpu: &mut Cpu, val: u8) -> u8 {
    let c = val >> 7;
    let result = (val << 1) | c;
    cpu.reg.set_flags(result == 0, false, false, c != 0);
    result
}

fn cb_rrc(cpu: &mut Cpu, val: u8) -> u8 {
    let c = val & 1;
    let result = (val >> 1) | (c << 7);
    cpu.reg.set_flags(result == 0, false, false, c != 0);
    result
}

fn cb_rl(cpu: &mut Cpu, val: u8) -> u8 {
    let old_c = cpu.reg.flag_c() as u8;
    let c = val >> 7;
    let result = (val << 1) | old_c;
    cpu.reg.set_flags(result == 0, false, false, c != 0);
    result
}

fn cb_rr(cpu: &mut Cpu, val: u8) -> u8 {
    let old_c = cpu.reg.flag_c() as u8;
    let c = val & 1;
    let result = (val >> 1) | (old_c << 7);
    cpu.reg.set_flags(result == 0, false, false, c != 0);
    result
}

fn cb_sla(cpu: &mut Cpu, val: u8) -> u8 {
    let c = val >> 7;
    let result = val << 1;
    cpu.reg.set_flags(result == 0, false, false, c != 0);
    result
}

fn cb_sra(cpu: &mut Cpu, val: u8) -> u8 {
    let c = val & 1;
    let result = ((val as i8) >> 1) as u8; // arithmetic (sign-extends)
    cpu.reg.set_flags(result == 0, false, false, c != 0);
    result
}

fn cb_srl(cpu: &mut Cpu, val: u8) -> u8 {
    let c = val & 1;
    let result = val >> 1;
    cpu.reg.set_flags(result == 0, false, false, c != 0);
    result
}

fn cb_swap(cpu: &mut Cpu, val: u8) -> u8 {
    let result = (val >> 4) | (val << 4);
    cpu.reg.set_flags(result == 0, false, false, false);
    result
}

fn cb_bit(cpu: &mut Cpu, val: u8, bit: u8) {
    let z = (val >> bit) & 1 == 0;
    cpu.reg.set_flag_z(z);
    cpu.reg.set_flag_n(false);
    cpu.reg.set_flag_h(true);
}
