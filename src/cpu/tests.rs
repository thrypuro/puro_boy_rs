//! CPU targets. Every test runs a tiny program through `CPU::step()`, so they check
//! decoding, the MMU and cycle counts as well as the instruction itself.
//!
//! Suggested order:
//! 1. `boot_state`, `loads`, `alu`, `alu16`, `rotates`, `control_flow`
//! 2. `cycles` - every opcode returns the right T-cycle count (one big checklist)
//! 3. `cb_prefixed`
//! 4. `interrupts`, `halt`
//! 5. `blargg` - real test ROMs, run with
//!    `cargo test --release blargg -- --ignored --nocapture`
//!
//! Model assumed here: `step()` first services a pending interrupt (if IME is set),
//! otherwise fetches and executes one instruction. It returns T-cycles (NOP = 4).
//!
//! References: https://gbdev.io/pandocs/CPU_Instruction_Set.html,
//! https://rgbds.gbdev.io/docs/gbz80.7 (per-instruction flags and cycles)

use super::shared::{get_opcodes, RegisterNames, RegisterNames::*};
use super::CPU;
use crate::MMU;

const IF: u16 = 0xFF0F;
const IE: u16 = 0xFFFF;

/// A 32KB ROM with `program` placed at the entry point 0x0100.
fn rom_with(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x100..0x100 + program.len()].copy_from_slice(program);
    rom
}

/// Builds a CPU (PC = 0x0100) running `program` and hands it to `test`.
fn with_cpu(program: &[u8], test: impl FnOnce(&mut CPU)) {
    let mut mmu = MMU::new(rom_with(program));
    let mut cpu = CPU::new(&mut mmu);
    test(&mut cpu);
}

fn get(cpu: &CPU, reg: RegisterNames) -> u8 {
    cpu.registers.get_register_value_8(reg)
}
fn set(cpu: &mut CPU, reg: RegisterNames, value: u8) {
    cpu.registers.set_register_value_8(reg, value)
}
fn get16(cpu: &CPU, reg: RegisterNames) -> u16 {
    cpu.registers.get_register_value_16(reg)
}
fn set16(cpu: &mut CPU, reg: RegisterNames, value: u16) {
    cpu.registers.set_register_value_16(reg, value)
}
// Memory helpers: a failed access is a bug in the MMU, so they panic with the address.
fn read(cpu: &CPU, addr: u16) -> u8 {
    cpu.memory
        .read(addr)
        .unwrap_or_else(|e| panic!("read {addr:#06X}: {e:?}"))
}
fn write(cpu: &mut CPU, addr: u16, value: u8) {
    cpu.memory
        .write(addr, value)
        .unwrap_or_else(|e| panic!("write {addr:#06X}: {e:?}"))
}
fn read_word(cpu: &CPU, addr: u16) -> u16 {
    cpu.memory
        .read_word(addr)
        .unwrap_or_else(|e| panic!("read_word {addr:#06X}: {e:?}"))
}
fn write_word(cpu: &mut CPU, addr: u16, value: u16) {
    cpu.memory
        .write_word(addr, value)
        .unwrap_or_else(|e| panic!("write_word {addr:#06X}: {e:?}"))
}
/// (Z, N, H, C)
fn flags(cpu: &CPU) -> (bool, bool, bool, bool) {
    cpu.registers.get_flags()
}

mod boot_state {
    use super::*;

    #[test]
    fn registers_match_dmg_after_boot_rom() {
        with_cpu(&[], |cpu| {
            assert_eq!(get16(cpu, AF), 0x01B0, "AF (low byte holds the flags)");
            assert_eq!(get16(cpu, BC), 0x0013);
            assert_eq!(get16(cpu, DE), 0x00D8);
            assert_eq!(get16(cpu, HL), 0x014D);
            assert_eq!(cpu.registers.sp, 0xFFFE);
            assert_eq!(cpu.registers.pc, 0x0100);
        });
    }

    #[test]
    fn f_register_reflects_flags() {
        // XOR A -> Z set, everything else clear -> F = 0x80
        with_cpu(&[0xAF], |cpu| {
            cpu.step();
            assert_eq!(get16(cpu, AF) & 0xFF, 0x80);
        });
    }

    #[test]
    fn nop_advances_pc() {
        with_cpu(&[0x00], |cpu| {
            assert_eq!(cpu.step(), 4);
            assert_eq!(cpu.registers.pc, 0x0101);
        });
    }
}

mod loads {
    use super::*;

    #[test]
    fn ld_r_n8() {
        with_cpu(&[0x06, 0x42], |cpu| {
            assert_eq!(cpu.step(), 8);
            assert_eq!(get(cpu, B), 0x42);
            assert_eq!(cpu.registers.pc, 0x0102);
        });
    }

    #[test]
    fn ld_r_r() {
        with_cpu(&[0x41], |cpu| {
            // LD B, C
            set(cpu, C, 0x99);
            assert_eq!(cpu.step(), 4);
            assert_eq!(get(cpu, B), 0x99);
        });
    }

    #[test]
    fn ld_rr_n16() {
        with_cpu(&[0x01, 0x34, 0x12], |cpu| {
            assert_eq!(cpu.step(), 12);
            assert_eq!(get16(cpu, BC), 0x1234);
            assert_eq!(cpu.registers.pc, 0x0103);
        });
    }

    #[test]
    fn ld_sp_n16() {
        with_cpu(&[0x31, 0x00, 0xD0], |cpu| {
            cpu.step();
            assert_eq!(cpu.registers.sp, 0xD000);
        });
    }

    #[test]
    fn ld_hl_indirect_a_writes_memory() {
        with_cpu(&[0x77], |cpu| {
            set16(cpu, HL, 0xC000);
            set(cpu, A, 0x12);
            assert_eq!(cpu.step(), 8);
            assert_eq!(read(cpu, 0xC000), 0x12);
        });
    }

    #[test]
    fn ld_a_hl_indirect_reads_memory() {
        with_cpu(&[0x7E], |cpu| {
            write(cpu, 0xC000, 0x34);
            set16(cpu, HL, 0xC000);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get(cpu, A), 0x34);
        });
    }

    #[test]
    fn ld_hl_indirect_n8() {
        with_cpu(&[0x36, 0x56], |cpu| {
            set16(cpu, HL, 0xC000);
            assert_eq!(cpu.step(), 12);
            assert_eq!(read(cpu, 0xC000), 0x56);
        });
    }

    #[test]
    fn ld_bc_indirect_and_de_indirect() {
        // LD A, (BC) ; LD (DE), A
        with_cpu(&[0x0A, 0x12], |cpu| {
            write(cpu, 0xC010, 0x77);
            set16(cpu, BC, 0xC010);
            set16(cpu, DE, 0xC020);
            cpu.step();
            cpu.step();
            assert_eq!(read(cpu, 0xC020), 0x77);
        });
    }

    #[test]
    fn ld_hl_increment() {
        // LD (HL+), A
        with_cpu(&[0x22], |cpu| {
            set16(cpu, HL, 0xC000);
            set(cpu, A, 0xAB);
            assert_eq!(cpu.step(), 8);
            assert_eq!(read(cpu, 0xC000), 0xAB);
            assert_eq!(get16(cpu, HL), 0xC001);
        });
    }

    #[test]
    fn ld_hl_decrement() {
        // LD A, (HL-)
        with_cpu(&[0x3A], |cpu| {
            write(cpu, 0xC005, 0xCD);
            set16(cpu, HL, 0xC005);
            cpu.step();
            assert_eq!(get(cpu, A), 0xCD);
            assert_eq!(get16(cpu, HL), 0xC004);
        });
    }

    #[test]
    fn ld_a16_a_and_back() {
        // LD (0xC123), A ; LD A, 0 ; LD A, (0xC123)
        with_cpu(&[0xEA, 0x23, 0xC1, 0x3E, 0x00, 0xFA, 0x23, 0xC1], |cpu| {
            set(cpu, A, 0x5A);
            assert_eq!(cpu.step(), 16);
            assert_eq!(read(cpu, 0xC123), 0x5A);
            cpu.step();
            assert_eq!(cpu.step(), 16);
            assert_eq!(get(cpu, A), 0x5A);
        });
    }

    #[test]
    fn ld_a16_sp_writes_both_bytes() {
        with_cpu(&[0x08, 0x00, 0xC0], |cpu| {
            cpu.registers.sp = 0xBEEF;
            assert_eq!(cpu.step(), 20);
            assert_eq!(read(cpu, 0xC000), 0xEF);
            assert_eq!(read(cpu, 0xC001), 0xBE);
        });
    }

    #[test]
    fn ldh_a8() {
        // LDH (0x80), A ; LDH A, (0x81)
        with_cpu(&[0xE0, 0x80, 0xF0, 0x81], |cpu| {
            write(cpu, 0xFF81, 0x22);
            set(cpu, A, 0x11);
            assert_eq!(cpu.step(), 12);
            assert_eq!(read(cpu, 0xFF80), 0x11);
            assert_eq!(cpu.step(), 12);
            assert_eq!(get(cpu, A), 0x22);
        });
    }

    #[test]
    fn ldh_c() {
        // LD (C), A ; LD A, (C)
        with_cpu(&[0xE2, 0xF2], |cpu| {
            set(cpu, C, 0x85);
            set(cpu, A, 0x33);
            assert_eq!(cpu.step(), 8);
            assert_eq!(read(cpu, 0xFF85), 0x33);
            set(cpu, A, 0);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get(cpu, A), 0x33);
        });
    }

    #[test]
    fn ld_sp_hl() {
        with_cpu(&[0xF9], |cpu| {
            set16(cpu, HL, 0xD123);
            assert_eq!(cpu.step(), 8);
            assert_eq!(cpu.registers.sp, 0xD123);
        });
    }

    #[test]
    fn ld_hl_sp_plus_e8() {
        with_cpu(&[0xF8, 0x02], |cpu| {
            cpu.registers.sp = 0xFFF8;
            assert_eq!(cpu.step(), 12);
            assert_eq!(get16(cpu, HL), 0xFFFA);
            assert_eq!(flags(cpu), (false, false, false, false));
        });
    }

    #[test]
    fn push_pop_round_trip() {
        // PUSH BC ; POP DE
        with_cpu(&[0xC5, 0xD1], |cpu| {
            cpu.registers.sp = 0xD000;
            set16(cpu, BC, 0x1234);
            assert_eq!(cpu.step(), 16);
            assert_eq!(cpu.registers.sp, 0xCFFE);
            assert_eq!(read_word(cpu, 0xCFFE), 0x1234);
            assert_eq!(cpu.step(), 12);
            assert_eq!(get16(cpu, DE), 0x1234);
            assert_eq!(cpu.registers.sp, 0xD000);
        });
    }

    #[test]
    fn pop_af_masks_low_nibble_and_sets_flags() {
        with_cpu(&[0xF1], |cpu| {
            cpu.registers.sp = 0xC000;
            write_word(cpu, 0xC000, 0x12FF);
            cpu.step();
            assert_eq!(get16(cpu, AF), 0x12F0);
            assert_eq!(flags(cpu), (true, true, true, true));
        });
    }

    #[test]
    fn push_af_includes_flags() {
        // XOR A (F = 0x80) ; PUSH AF
        with_cpu(&[0xAF, 0xF5], |cpu| {
            cpu.registers.sp = 0xD000;
            cpu.step();
            cpu.step();
            assert_eq!(read_word(cpu, 0xCFFE), 0x0080);
        });
    }
}

mod alu {
    use super::*;

    #[test]
    fn add() {
        with_cpu(&[0x80], |cpu| {
            set(cpu, A, 0x3A);
            set(cpu, B, 0xC6);
            assert_eq!(cpu.step(), 4);
            assert_eq!(get(cpu, A), 0x00);
            assert_eq!(flags(cpu), (true, false, true, true));
        });
    }

    #[test]
    fn add_n8() {
        with_cpu(&[0xC6, 0x01], |cpu| {
            set(cpu, A, 0x0F);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get(cpu, A), 0x10);
            assert_eq!(flags(cpu), (false, false, true, false));
        });
    }

    #[test]
    fn add_hl_indirect() {
        with_cpu(&[0x86], |cpu| {
            write(cpu, 0xC000, 0x02);
            set16(cpu, HL, 0xC000);
            set(cpu, A, 0x03);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get(cpu, A), 0x05);
        });
    }

    #[test]
    fn adc_uses_carry() {
        with_cpu(&[0x8B], |cpu| {
            // ADC A, E
            set(cpu, A, 0xE1);
            set(cpu, E, 0x0F);
            cpu.registers.set_flags(false, false, false, true);
            cpu.step();
            assert_eq!(get(cpu, A), 0xF1);
            assert_eq!(flags(cpu), (false, false, true, false));
        });
    }

    #[test]
    fn sub_to_zero() {
        with_cpu(&[0x93], |cpu| {
            // SUB A, E
            set(cpu, A, 0x3E);
            set(cpu, E, 0x3E);
            cpu.step();
            assert_eq!(get(cpu, A), 0x00);
            assert_eq!(flags(cpu), (true, true, false, false));
        });
    }

    #[test]
    fn sub_with_borrow() {
        with_cpu(&[0x93], |cpu| {
            set(cpu, A, 0x3E);
            set(cpu, E, 0x40);
            cpu.step();
            assert_eq!(get(cpu, A), 0xFE);
            assert_eq!(flags(cpu), (false, true, false, true));
        });
    }

    #[test]
    fn sbc_uses_carry() {
        with_cpu(&[0x9C], |cpu| {
            // SBC A, H
            set(cpu, A, 0x3B);
            set(cpu, H, 0x2A);
            cpu.registers.set_flags(false, false, false, true);
            cpu.step();
            assert_eq!(get(cpu, A), 0x10);
            assert_eq!(flags(cpu), (false, true, false, false));
        });
    }

    #[test]
    fn and() {
        with_cpu(&[0xA5], |cpu| {
            // AND A, L
            set(cpu, A, 0x5A);
            set(cpu, L, 0x3F);
            cpu.step();
            assert_eq!(get(cpu, A), 0x1A);
            assert_eq!(flags(cpu), (false, false, true, false));
        });
    }

    #[test]
    fn or() {
        with_cpu(&[0xB0], |cpu| {
            // OR A, B
            set(cpu, A, 0x50);
            set(cpu, B, 0x0A);
            cpu.step();
            assert_eq!(get(cpu, A), 0x5A);
            assert_eq!(flags(cpu), (false, false, false, false));
        });
    }

    #[test]
    fn xor_a_clears_a() {
        with_cpu(&[0xAF], |cpu| {
            set(cpu, A, 0x77);
            cpu.step();
            assert_eq!(get(cpu, A), 0x00);
            assert_eq!(flags(cpu), (true, false, false, false));
        });
    }

    #[test]
    fn cp_sets_flags_without_changing_a() {
        with_cpu(&[0xB8], |cpu| {
            // CP A, B
            set(cpu, A, 0x3C);
            set(cpu, B, 0x2F);
            cpu.step();
            assert_eq!(get(cpu, A), 0x3C);
            assert_eq!(flags(cpu), (false, true, true, false));
        });
    }

    #[test]
    fn cp_n8_equal() {
        with_cpu(&[0xFE, 0x3C], |cpu| {
            set(cpu, A, 0x3C);
            assert_eq!(cpu.step(), 8);
            assert_eq!(flags(cpu), (true, true, false, false));
        });
    }

    #[test]
    fn inc_half_carry_and_keeps_carry() {
        with_cpu(&[0x04], |cpu| {
            set(cpu, B, 0x0F);
            cpu.registers.set_flags(false, true, false, true);
            assert_eq!(cpu.step(), 4);
            assert_eq!(get(cpu, B), 0x10);
            assert_eq!(flags(cpu), (false, false, true, true));
        });
    }

    #[test]
    fn inc_wraps_to_zero() {
        with_cpu(&[0x04], |cpu| {
            set(cpu, B, 0xFF);
            cpu.registers.set_flags(false, false, false, false);
            cpu.step();
            assert_eq!(get(cpu, B), 0x00);
            assert_eq!(flags(cpu), (true, false, true, false));
        });
    }

    #[test]
    fn dec_to_zero() {
        with_cpu(&[0x05], |cpu| {
            set(cpu, B, 0x01);
            cpu.registers.set_flags(false, false, false, false);
            cpu.step();
            assert_eq!(get(cpu, B), 0x00);
            assert_eq!(flags(cpu), (true, true, false, false));
        });
    }

    #[test]
    fn dec_half_borrow() {
        with_cpu(&[0x05], |cpu| {
            set(cpu, B, 0x10);
            cpu.registers.set_flags(false, false, false, false);
            cpu.step();
            assert_eq!(get(cpu, B), 0x0F);
            assert_eq!(flags(cpu), (false, true, true, false));
        });
    }

    #[test]
    fn inc_hl_indirect() {
        with_cpu(&[0x34], |cpu| {
            write(cpu, 0xC000, 0x41);
            set16(cpu, HL, 0xC000);
            assert_eq!(cpu.step(), 12);
            assert_eq!(read(cpu, 0xC000), 0x42);
        });
    }

    #[test]
    fn daa_after_add() {
        // ADD A, B ; DAA   (45 + 38 = 83 in BCD)
        with_cpu(&[0x80, 0x27], |cpu| {
            set(cpu, A, 0x45);
            set(cpu, B, 0x38);
            cpu.step();
            assert_eq!(cpu.step(), 4);
            assert_eq!(get(cpu, A), 0x83);
            assert_eq!(flags(cpu), (false, false, false, false));
        });
    }

    #[test]
    fn daa_after_add_overflow() {
        // 99 + 01 = 00 carry 1
        with_cpu(&[0x80, 0x27], |cpu| {
            set(cpu, A, 0x99);
            set(cpu, B, 0x01);
            cpu.step();
            cpu.step();
            assert_eq!(get(cpu, A), 0x00);
            assert_eq!(flags(cpu), (true, false, false, true));
        });
    }

    #[test]
    fn daa_after_sub() {
        // 10 - 01 = 09
        with_cpu(&[0xD6, 0x01, 0x27], |cpu| {
            set(cpu, A, 0x10);
            cpu.step();
            cpu.step();
            assert_eq!(get(cpu, A), 0x09);
            assert_eq!(flags(cpu), (false, true, false, false));
        });
    }

    #[test]
    fn cpl() {
        with_cpu(&[0x2F], |cpu| {
            set(cpu, A, 0x35);
            cpu.registers.set_flags(false, false, false, false);
            cpu.step();
            assert_eq!(get(cpu, A), 0xCA);
            assert_eq!(flags(cpu), (false, true, true, false));
        });
    }

    #[test]
    fn scf_and_ccf() {
        with_cpu(&[0x37, 0x3F], |cpu| {
            cpu.registers.set_flags(true, true, true, false);
            cpu.step();
            assert_eq!(flags(cpu), (true, false, false, true), "SCF");
            cpu.step();
            assert_eq!(flags(cpu), (true, false, false, false), "CCF");
        });
    }
}

mod alu16 {
    use super::*;

    #[test]
    fn add_hl_bc() {
        with_cpu(&[0x09], |cpu| {
            set16(cpu, HL, 0x8A23);
            set16(cpu, BC, 0x0605);
            cpu.registers.set_flags(true, true, false, false);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get16(cpu, HL), 0x9028);
            assert_eq!(flags(cpu), (true, false, true, false), "Z is untouched");
        });
    }

    #[test]
    fn add_hl_hl_carry() {
        with_cpu(&[0x29], |cpu| {
            set16(cpu, HL, 0x8A23);
            cpu.registers.set_flags(false, false, false, false);
            cpu.step();
            assert_eq!(get16(cpu, HL), 0x1446);
            assert_eq!(flags(cpu), (false, false, true, true));
        });
    }

    #[test]
    fn inc_dec_rr_leave_flags_alone() {
        // INC BC ; DEC DE
        with_cpu(&[0x03, 0x1B], |cpu| {
            set16(cpu, BC, 0xFFFF);
            set16(cpu, DE, 0x0000);
            cpu.registers.set_flags(false, true, false, true);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get16(cpu, BC), 0x0000);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get16(cpu, DE), 0xFFFF);
            assert_eq!(flags(cpu), (false, true, false, true));
        });
    }

    #[test]
    fn inc_sp() {
        with_cpu(&[0x33], |cpu| {
            cpu.registers.sp = 0xC000;
            cpu.step();
            assert_eq!(cpu.registers.sp, 0xC001);
        });
    }

    #[test]
    fn add_sp_e8_positive() {
        with_cpu(&[0xE8, 0x02], |cpu| {
            cpu.registers.sp = 0xFFF8;
            assert_eq!(cpu.step(), 16);
            assert_eq!(cpu.registers.sp, 0xFFFA);
            assert_eq!(flags(cpu), (false, false, false, false));
        });
    }

    #[test]
    fn add_sp_e8_negative() {
        // Flags come from the unsigned low-byte add: 0x01 + 0xFF
        with_cpu(&[0xE8, 0xFF], |cpu| {
            cpu.registers.sp = 0x0001;
            cpu.step();
            assert_eq!(cpu.registers.sp, 0x0000);
            assert_eq!(flags(cpu), (false, false, true, true), "Z is always 0");
        });
    }
}

mod rotates {
    use super::*;

    #[test]
    fn rlca() {
        with_cpu(&[0x07], |cpu| {
            set(cpu, A, 0x85);
            assert_eq!(cpu.step(), 4);
            assert_eq!(get(cpu, A), 0x0B);
            assert_eq!(flags(cpu), (false, false, false, true));
        });
    }

    #[test]
    fn rla() {
        with_cpu(&[0x17], |cpu| {
            set(cpu, A, 0x95);
            cpu.registers.set_flags(false, false, false, true);
            cpu.step();
            assert_eq!(get(cpu, A), 0x2B);
            assert_eq!(flags(cpu), (false, false, false, true));
        });
    }

    #[test]
    fn rrca() {
        with_cpu(&[0x0F], |cpu| {
            set(cpu, A, 0x3B);
            cpu.step();
            assert_eq!(get(cpu, A), 0x9D);
            assert_eq!(flags(cpu), (false, false, false, true));
        });
    }

    #[test]
    fn rra() {
        with_cpu(&[0x1F], |cpu| {
            set(cpu, A, 0x81);
            cpu.registers.set_flags(false, false, false, false);
            cpu.step();
            assert_eq!(get(cpu, A), 0x40);
            assert_eq!(flags(cpu), (false, false, false, true));
        });
    }

    #[test]
    fn rlca_never_sets_zero() {
        with_cpu(&[0x07], |cpu| {
            set(cpu, A, 0x00);
            cpu.step();
            assert_eq!(flags(cpu).0, false);
        });
    }
}

mod control_flow {
    use super::*;

    #[test]
    fn jp_a16() {
        with_cpu(&[0xC3, 0x50, 0x01], |cpu| {
            assert_eq!(cpu.step(), 16);
            assert_eq!(cpu.registers.pc, 0x0150);
        });
    }

    #[test]
    fn jp_nz_taken() {
        with_cpu(&[0xC2, 0x50, 0x01], |cpu| {
            cpu.registers.set_flags(false, false, false, false);
            assert_eq!(cpu.step(), 16);
            assert_eq!(cpu.registers.pc, 0x0150);
        });
    }

    #[test]
    fn jp_nz_not_taken() {
        with_cpu(&[0xC2, 0x50, 0x01], |cpu| {
            cpu.registers.set_flags(true, false, false, false);
            assert_eq!(cpu.step(), 12);
            assert_eq!(cpu.registers.pc, 0x0103);
        });
    }

    #[test]
    fn jp_hl() {
        with_cpu(&[0xE9], |cpu| {
            set16(cpu, HL, 0x1234);
            assert_eq!(cpu.step(), 4);
            assert_eq!(cpu.registers.pc, 0x1234);
        });
    }

    #[test]
    fn jr_forward() {
        with_cpu(&[0x18, 0x05], |cpu| {
            assert_eq!(cpu.step(), 12);
            assert_eq!(
                cpu.registers.pc, 0x0107,
                "relative to the *next* instruction"
            );
        });
    }

    #[test]
    fn jr_backward() {
        with_cpu(&[0x18, 0xFE], |cpu| {
            cpu.step();
            assert_eq!(cpu.registers.pc, 0x0100, "JR -2 is an infinite loop");
        });
    }

    #[test]
    fn jr_z_taken_and_not_taken() {
        with_cpu(&[0x28, 0x10], |cpu| {
            cpu.registers.set_flags(false, false, false, false);
            assert_eq!(cpu.step(), 8);
            assert_eq!(cpu.registers.pc, 0x0102);
        });
        with_cpu(&[0x28, 0x10], |cpu| {
            cpu.registers.set_flags(true, false, false, false);
            assert_eq!(cpu.step(), 12);
            assert_eq!(cpu.registers.pc, 0x0112);
        });
    }

    #[test]
    fn jr_nc_taken() {
        with_cpu(&[0x30, 0x10], |cpu| {
            cpu.registers.set_flags(false, false, false, false);
            assert_eq!(cpu.step(), 12);
            assert_eq!(cpu.registers.pc, 0x0112);
        });
    }

    #[test]
    fn call_pushes_return_address() {
        with_cpu(&[0xCD, 0x34, 0x12], |cpu| {
            cpu.registers.sp = 0xD000;
            assert_eq!(cpu.step(), 24);
            assert_eq!(cpu.registers.pc, 0x1234);
            assert_eq!(cpu.registers.sp, 0xCFFE);
            assert_eq!(read_word(cpu, 0xCFFE), 0x0103);
        });
    }

    #[test]
    fn call_nz_not_taken() {
        with_cpu(&[0xC4, 0x34, 0x12], |cpu| {
            cpu.registers.sp = 0xD000;
            cpu.registers.set_flags(true, false, false, false);
            assert_eq!(cpu.step(), 12);
            assert_eq!(cpu.registers.pc, 0x0103);
            assert_eq!(cpu.registers.sp, 0xD000);
        });
    }

    #[test]
    fn ret_pops_pc() {
        with_cpu(&[0xC9], |cpu| {
            cpu.registers.sp = 0xC000;
            write_word(cpu, 0xC000, 0x1234);
            assert_eq!(cpu.step(), 16);
            assert_eq!(cpu.registers.pc, 0x1234);
            assert_eq!(cpu.registers.sp, 0xC002);
        });
    }

    #[test]
    fn ret_z_taken_and_not_taken() {
        with_cpu(&[0xC8], |cpu| {
            cpu.registers.sp = 0xC000;
            write_word(cpu, 0xC000, 0x1234);
            cpu.registers.set_flags(true, false, false, false);
            assert_eq!(cpu.step(), 20);
            assert_eq!(cpu.registers.pc, 0x1234);
        });
        with_cpu(&[0xC8], |cpu| {
            cpu.registers.sp = 0xC000;
            cpu.registers.set_flags(false, false, false, false);
            assert_eq!(cpu.step(), 8);
            assert_eq!(cpu.registers.pc, 0x0101);
            assert_eq!(cpu.registers.sp, 0xC000);
        });
    }

    #[test]
    fn call_then_ret() {
        // 0x0100: CALL 0x0110 ; 0x0103: NOP ... 0x0110: RET
        let mut program = vec![0u8; 0x11];
        program[0..3].copy_from_slice(&[0xCD, 0x10, 0x01]);
        program[0x10] = 0xC9;
        with_cpu(&program, |cpu| {
            cpu.registers.sp = 0xD000;
            cpu.step();
            assert_eq!(cpu.registers.pc, 0x0110);
            cpu.step();
            assert_eq!(cpu.registers.pc, 0x0103);
            assert_eq!(cpu.registers.sp, 0xD000);
        });
    }

    #[test]
    fn rst() {
        with_cpu(&[0xFF], |cpu| {
            // RST 38h
            cpu.registers.sp = 0xD000;
            assert_eq!(cpu.step(), 16);
            assert_eq!(cpu.registers.pc, 0x0038);
            assert_eq!(read_word(cpu, 0xCFFE), 0x0101);
        });
    }
}

mod cb_prefixed {
    use super::*;

    #[test]
    fn rlc_r() {
        with_cpu(&[0xCB, 0x00], |cpu| {
            set(cpu, B, 0x85);
            assert_eq!(cpu.step(), 8);
            assert_eq!(get(cpu, B), 0x0B);
            assert_eq!(flags(cpu), (false, false, false, true));
            assert_eq!(cpu.registers.pc, 0x0102);
        });
    }

    #[test]
    fn rl_sets_zero_unlike_rla() {
        with_cpu(&[0xCB, 0x11], |cpu| {
            set(cpu, C, 0x80);
            cpu.registers.set_flags(false, false, false, false);
            cpu.step();
            assert_eq!(get(cpu, C), 0x00);
            assert_eq!(flags(cpu), (true, false, false, true));
        });
    }

    #[test]
    fn sla() {
        with_cpu(&[0xCB, 0x22], |cpu| {
            set(cpu, D, 0x80);
            cpu.step();
            assert_eq!(get(cpu, D), 0x00);
            assert_eq!(flags(cpu), (true, false, false, true));
        });
    }

    #[test]
    fn sra_keeps_sign_bit() {
        with_cpu(&[0xCB, 0x2F], |cpu| {
            set(cpu, A, 0x8A);
            cpu.step();
            assert_eq!(get(cpu, A), 0xC5);
            assert_eq!(flags(cpu), (false, false, false, false));
        });
    }

    #[test]
    fn srl() {
        with_cpu(&[0xCB, 0x38], |cpu| {
            set(cpu, B, 0x01);
            cpu.step();
            assert_eq!(get(cpu, B), 0x00);
            assert_eq!(flags(cpu), (true, false, false, true));
        });
    }

    #[test]
    fn swap() {
        with_cpu(&[0xCB, 0x37], |cpu| {
            set(cpu, A, 0xF1);
            cpu.registers.set_flags(false, true, true, true);
            cpu.step();
            assert_eq!(get(cpu, A), 0x1F);
            assert_eq!(flags(cpu), (false, false, false, false));
        });
    }

    #[test]
    fn bit() {
        with_cpu(&[0xCB, 0x7C], |cpu| {
            // BIT 7, H
            set(cpu, H, 0x80);
            cpu.registers.set_flags(true, true, false, true);
            assert_eq!(cpu.step(), 8);
            assert_eq!(flags(cpu), (false, false, true, true), "C untouched");
        });
        with_cpu(&[0xCB, 0x7C], |cpu| {
            set(cpu, H, 0x7F);
            cpu.step();
            assert_eq!(flags(cpu).0, true, "bit clear -> Z set");
        });
    }

    #[test]
    fn set_and_res() {
        // SET 3, A ; RES 7, A
        with_cpu(&[0xCB, 0xDF, 0xCB, 0xBF], |cpu| {
            set(cpu, A, 0x80);
            cpu.step();
            assert_eq!(get(cpu, A), 0x88);
            cpu.step();
            assert_eq!(get(cpu, A), 0x08);
        });
    }

    #[test]
    fn set_hl_indirect() {
        with_cpu(&[0xCB, 0xC6], |cpu| {
            set16(cpu, HL, 0xC000);
            assert_eq!(cpu.step(), 16);
            assert_eq!(read(cpu, 0xC000), 0x01);
        });
    }

    #[test]
    fn bit_hl_indirect() {
        with_cpu(&[0xCB, 0x46], |cpu| {
            write(cpu, 0xC000, 0x01);
            set16(cpu, HL, 0xC000);
            assert_eq!(cpu.step(), 12);
            assert_eq!(flags(cpu).0, false);
        });
    }
}

mod cycles {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    /// Runs `program` once and returns the cycles `step()` reported, or why it failed.
    fn cycles_for(program: &[u8]) -> Result<u8, String> {
        catch_unwind(AssertUnwindSafe(|| {
            let mut mmu = MMU::new(rom_with(program));
            let mut cpu = CPU::new(&mut mmu);
            set16(&mut cpu, AF, 0x01B0); // Z = 1, C = 1: NZ/NC not taken, Z/C taken
            set16(&mut cpu, HL, 0xC000);
            cpu.registers.sp = 0xD000;
            cpu.step()
        }))
        .map_err(|e| {
            e.downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "panicked".into())
        })
    }

    fn expected_cycles(op: &json::JsonValue) -> u8 {
        let cycles = &op["cycles"];
        if cycles.len() == 1 {
            return cycles[0].as_u8().unwrap();
        }
        let condition = op["operands"][0]["name"].as_str().unwrap();
        let taken = matches!(condition, "Z" | "C");
        cycles[if taken { 0 } else { 1 }].as_u8().unwrap()
    }

    /// Checks every opcode in a table and lists all the ones that are wrong.
    fn check_table(table: &str, prefix: &[u8]) {
        let opcodes = get_opcodes();
        let mut failures = Vec::new();
        for code in 0..=0xFFu8 {
            let op = &opcodes[table][format!("0x{code:02X}")];
            let mnemonic = op["mnemonic"].as_str().unwrap();
            if mnemonic.starts_with("ILLEGAL") || matches!(mnemonic, "PREFIX" | "STOP" | "HALT") {
                continue;
            }
            let mut program = prefix.to_vec();
            program.extend([code, 0x00, 0x00]);
            let expected = expected_cycles(op);
            match cycles_for(&program) {
                Ok(c) if c == expected => {}
                Ok(c) => {
                    failures.push(format!("0x{code:02X} {mnemonic}: got {c}, want {expected}"))
                }
                Err(e) => failures.push(format!("0x{code:02X} {mnemonic}: panicked: {e}")),
            }
        }
        assert!(
            failures.is_empty(),
            "{} {table} opcodes wrong:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn every_unprefixed_opcode() {
        check_table("unprefixed", &[]);
    }

    #[test]
    fn every_cb_prefixed_opcode() {
        check_table("cbprefixed", &[0xCB]);
    }
}

mod interrupts {
    use super::*;

    #[test]
    fn dispatch_jumps_to_vector() {
        with_cpu(&[0x00], |cpu| {
            cpu.ime = true;
            cpu.registers.sp = 0xD000;
            write(cpu, IE, 0x01);
            write(cpu, IF, 0x01);
            assert_eq!(cpu.step(), 20);
            assert_eq!(cpu.registers.pc, 0x0040);
            assert_eq!(read_word(cpu, 0xCFFE), 0x0100, "old PC pushed");
            assert_eq!(read(cpu, IF) & 0x01, 0, "IF bit acknowledged");
            assert!(!cpu.ime, "IME cleared on dispatch");
        });
    }

    #[test]
    fn each_source_has_its_own_vector() {
        for (bit, vector) in [(0, 0x40), (1, 0x48), (2, 0x50), (3, 0x58), (4, 0x60)] {
            with_cpu(&[0x00], |cpu| {
                cpu.ime = true;
                cpu.registers.sp = 0xD000;
                write(cpu, IE, 0x1F);
                write(cpu, IF, 1 << bit);
                cpu.step();
                assert_eq!(cpu.registers.pc, vector, "interrupt bit {bit}");
            });
        }
    }

    #[test]
    fn lowest_bit_has_priority() {
        with_cpu(&[0x00], |cpu| {
            cpu.ime = true;
            cpu.registers.sp = 0xD000;
            write(cpu, IE, 0x1F);
            write(cpu, IF, 0b10100); // timer + joypad
            cpu.step();
            assert_eq!(cpu.registers.pc, 0x0050, "timer first");
            assert_eq!(read(cpu, IF) & 0x1F, 0b10000, "joypad still pending");
        });
    }

    #[test]
    fn not_dispatched_when_ime_clear() {
        with_cpu(&[0x00], |cpu| {
            cpu.ime = false;
            write(cpu, IE, 0x01);
            write(cpu, IF, 0x01);
            cpu.step();
            assert_eq!(cpu.registers.pc, 0x0101);
        });
    }

    #[test]
    fn not_dispatched_when_masked_by_ie() {
        with_cpu(&[0x00], |cpu| {
            cpu.ime = true;
            write(cpu, IE, 0x00);
            write(cpu, IF, 0x01);
            cpu.step();
            assert_eq!(cpu.registers.pc, 0x0101);
        });
    }

    #[test]
    fn di_clears_ime() {
        with_cpu(&[0xF3], |cpu| {
            cpu.ime = true;
            assert_eq!(cpu.step(), 4);
            assert!(!cpu.ime);
        });
    }

    #[test]
    fn ei_takes_effect_after_the_next_instruction() {
        // EI ; NOP ; NOP
        with_cpu(&[0xFB, 0x00, 0x00], |cpu| {
            cpu.registers.sp = 0xD000;
            write(cpu, IE, 0x01);
            write(cpu, IF, 0x01);
            assert_eq!(cpu.step(), 4); // EI
            cpu.step(); // NOP still runs
            assert_eq!(cpu.registers.pc, 0x0102);
            cpu.step(); // now the interrupt is serviced
            assert_eq!(cpu.registers.pc, 0x0040);
            assert_eq!(read_word(cpu, 0xCFFE), 0x0102);
        });
    }

    #[test]
    fn reti_returns_and_enables_ime() {
        with_cpu(&[0xD9], |cpu| {
            cpu.ime = false;
            cpu.registers.sp = 0xC000;
            write_word(cpu, 0xC000, 0x1234);
            assert_eq!(cpu.step(), 16);
            assert_eq!(cpu.registers.pc, 0x1234);
            assert!(cpu.ime);
        });
    }
}

mod halt {
    use super::*;

    #[test]
    fn halt_stops_execution() {
        with_cpu(&[0x76, 0x00], |cpu| {
            write(cpu, IE, 0x00);
            assert_eq!(cpu.step(), 4);
            assert!(cpu.halted);
            assert_eq!(cpu.step(), 4, "time still passes while halted");
            assert_eq!(cpu.registers.pc, 0x0101);
        });
    }

    #[test]
    fn halt_wakes_without_dispatch_when_ime_clear() {
        with_cpu(&[0x76, 0x00], |cpu| {
            cpu.ime = false;
            cpu.step();
            write(cpu, IE, 0x01);
            write(cpu, IF, 0x01);
            cpu.step();
            assert!(!cpu.halted);
            assert_eq!(
                cpu.registers.pc, 0x0102,
                "continued with the NOP after HALT"
            );
        });
    }

    #[test]
    fn halt_wakes_and_dispatches_when_ime_set() {
        with_cpu(&[0x76, 0x00], |cpu| {
            cpu.ime = true;
            cpu.registers.sp = 0xD000;
            cpu.step();
            write(cpu, IE, 0x01);
            write(cpu, IF, 0x01);
            cpu.step();
            assert!(!cpu.halted);
            assert_eq!(cpu.registers.pc, 0x0040);
            assert_eq!(read_word(cpu, 0xCFFE), 0x0101);
        });
    }
}

/// Blargg's ROMs print "Passed" or "Failed" over the serial port.
/// These need the CPU, MMU, timer and interrupts working, so they're the final boss.
mod blargg {
    use super::*;

    const ROM_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_roms/game-boy-test-roms-v7.0/blargg"
    );

    fn run_rom(path: &str) {
        let full = format!("{ROM_DIR}/{path}");
        let rom = std::fs::read(&full).unwrap_or_else(|e| panic!("can't read {full}: {e}"));
        let mut mmu = MMU::new(rom);
        let mut cpu = CPU::new(&mut mmu);
        // ~60s of emulated time is plenty for any single ROM.
        let mut elapsed: u64 = 0;
        for _ in 0..100_000_000u64 {
            let cycles = cpu.step();
            cpu.memory.tick(cycles);
            elapsed += cycles as u64;
            let out = cpu.memory.serial_output();
            if out.contains("Passed") {
                return;
            }
            if out.contains("Failed") || elapsed > 60 * 4_194_304 {
                break;
            }
        }
        panic!("{path}:\n{}", cpu.memory.serial_output());
    }

    #[test]
    #[ignore]
    fn cpu_instrs_01_special() {
        run_rom("cpu_instrs/individual/01-special.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_02_interrupts() {
        run_rom("cpu_instrs/individual/02-interrupts.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_03_op_sp_hl() {
        run_rom("cpu_instrs/individual/03-op sp,hl.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_04_op_r_imm() {
        run_rom("cpu_instrs/individual/04-op r,imm.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_05_op_rp() {
        run_rom("cpu_instrs/individual/05-op rp.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_06_ld_r_r() {
        run_rom("cpu_instrs/individual/06-ld r,r.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_07_jr_jp_call_ret_rst() {
        run_rom("cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_08_misc_instrs() {
        run_rom("cpu_instrs/individual/08-misc instrs.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_09_op_r_r() {
        run_rom("cpu_instrs/individual/09-op r,r.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_10_bit_ops() {
        run_rom("cpu_instrs/individual/10-bit ops.gb");
    }
    #[test]
    #[ignore]
    fn cpu_instrs_11_op_a_hl() {
        run_rom("cpu_instrs/individual/11-op a,(hl).gb");
    }
    #[test]
    #[ignore]
    fn instr_timing() {
        run_rom("instr_timing/instr_timing.gb");
    }
}
