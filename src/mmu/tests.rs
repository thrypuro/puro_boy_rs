//! MMU targets. Start with `memory_map` - the CPU can't do anything useful until
//! reads and writes land in the right place.
//!
//! Tests return `EmuRes<()>`, so an unexpected `Err` from the MMU fails the test
//! with the error variant shown.
//!
//! Run one group with e.g. `cargo test mmu::tests::memory_map`.
//!
//! Reference: https://gbdev.io/pandocs/Memory_Map.html

use super::*;
use crate::error::{EmuError, EmuRes};

const IF: u16 = 0xFF0F;
const IE: u16 = 0xFFFF;

fn blank_mmu() -> MMU {
    MMU::new(vec![0; 0x8000])
}

fn run(mmu: &mut MMU, cycles: u32) {
    assert!(cycles % 4 == 0);
    for _ in 0..cycles / 4 {
        mmu.tick(4);
    }
}

mod memory_map {
    use super::*;

    #[test]
    fn reads_rom() -> EmuRes<()> {
        let mut rom = vec![0; 0x8000];
        rom[0x0000] = 0x11;
        rom[0x0150] = 0x22;
        rom[0x7FFF] = 0x33;
        let mmu = MMU::new(rom);
        assert_eq!(mmu.read(0x0000)?, 0x11);
        assert_eq!(mmu.read(0x0150)?, 0x22);
        assert_eq!(mmu.read(0x7FFF)?, 0x33);
        Ok(())
    }

    #[test]
    fn small_rom_is_padded() -> EmuRes<()> {
        let mmu = MMU::new(vec![0xAB, 0xCD]);
        assert_eq!(mmu.read(0x0000)?, 0xAB);
        assert_eq!(mmu.read(0x0001)?, 0xCD);
        assert_eq!(mmu.read(0x7FFF)?, 0x00);
        Ok(())
    }

    #[test]
    fn writes_to_rom_succeed_but_are_ignored() -> EmuRes<()> {
        // Without an MBC, ROM is read-only. (With MBC1 these writes select banks,
        // so they must not be errors.)
        let mut mmu = MMU::new(vec![0x42; 0x8000]);
        mmu.write(0x0000, 0x00)?;
        mmu.write(0x4000, 0x00)?;
        assert_eq!(mmu.read(0x0000)?, 0x42);
        assert_eq!(mmu.read(0x4000)?, 0x42);
        Ok(())
    }

    #[test]
    fn vram_is_routed_to_ppu() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0x8000, 0x12)?;
        mmu.write(0x9FFF, 0x34)?;
        assert_eq!(mmu.read(0x8000)?, 0x12);
        assert_eq!(mmu.read(0x9FFF)?, 0x34);
        assert_eq!(mmu.ppu.read(0x8000), 0x12, "VRAM should live in the PPU");
        Ok(())
    }

    #[test]
    fn external_ram() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0xA000, 0x56)?;
        mmu.write(0xBFFF, 0x78)?;
        assert_eq!(mmu.read(0xA000)?, 0x56);
        assert_eq!(mmu.read(0xBFFF)?, 0x78);
        Ok(())
    }

    #[test]
    fn work_ram() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0xC000, 0x9A)?;
        mmu.write(0xDFFF, 0xBC)?;
        assert_eq!(mmu.read(0xC000)?, 0x9A);
        assert_eq!(mmu.read(0xDFFF)?, 0xBC);
        Ok(())
    }

    #[test]
    fn external_and_work_ram_are_separate() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0xA000, 0x01)?;
        mmu.write(0xC000, 0x02)?;
        assert_eq!(mmu.read(0xA000)?, 0x01);
        assert_eq!(mmu.read(0xC000)?, 0x02);
        Ok(())
    }

    #[test]
    fn oam_is_routed_to_ppu() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0xFE00, 0x44)?;
        mmu.write(0xFE9F, 0x55)?;
        assert_eq!(mmu.read(0xFE00)?, 0x44);
        assert_eq!(mmu.read(0xFE9F)?, 0x55);
        assert_eq!(mmu.ppu.read(0xFE00), 0x44, "OAM should live in the PPU");
        Ok(())
    }

    #[test]
    fn lcd_registers_are_routed_to_ppu() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0xFF42, 0x66)?; // SCY
        assert_eq!(mmu.read(0xFF42)?, 0x66);
        assert_eq!(mmu.ppu.read(0xFF42), 0x66);
        Ok(())
    }

    #[test]
    fn every_io_register_is_accessible() -> EmuRes<()> {
        // Games poke sound/joypad/etc. registers you haven't implemented yet.
        // Those should be Ok (ignored or stored), never an error.
        let mut mmu = blank_mmu();
        for addr in 0xFF00..=0xFF7Fu16 {
            if addr == 0xFF46 {
                continue; // DMA, tested separately
            }
            mmu.write(addr, 0)?;
            mmu.read(addr)?;
        }
        Ok(())
    }

    #[test]
    fn high_ram() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0xFF80, 0x77)?;
        mmu.write(0xFFFE, 0x88)?;
        assert_eq!(mmu.read(0xFF80)?, 0x77);
        assert_eq!(mmu.read(0xFFFE)?, 0x88);
        Ok(())
    }

    #[test]
    fn interrupt_enable_register() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(IE, 0x1F)?;
        assert_eq!(mmu.read(IE)?, 0x1F);
        Ok(())
    }

    #[test]
    fn interrupt_flag_register() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(IF, 0x05)?;
        assert_eq!(mmu.read(IF)? & 0x1F, 0x05);
        Ok(())
    }

    #[test]
    fn words_are_little_endian() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write_word(0xC000, 0xBEEF)?;
        assert_eq!(mmu.read(0xC000)?, 0xEF);
        assert_eq!(mmu.read(0xC001)?, 0xBE);
        assert_eq!(mmu.read_word(0xC000)?, 0xBEEF);
        Ok(())
    }
}

mod prohibited {
    use super::*;

    // Echo RAM (0xE000-0xFDFF) and the unusable region (0xFEA0-0xFEFF) are
    // treated as prohibited. (On hardware, echo RAM mirrors 0xC000-0xDDFF.
    // If you ever decide to emulate that, swap these for mirror tests.)

    #[test]
    fn echo_ram_read_is_prohibited() {
        let mmu = blank_mmu();
        assert_eq!(mmu.read(0xE000), Err(EmuError::ProhibitedRead));
        assert_eq!(mmu.read(0xFDFF), Err(EmuError::ProhibitedRead));
    }

    #[test]
    fn echo_ram_write_is_prohibited() {
        let mut mmu = blank_mmu();
        assert_eq!(mmu.write(0xE000, 1), Err(EmuError::ProhibitedWrite));
        assert_eq!(mmu.write(0xFDFF, 1), Err(EmuError::ProhibitedWrite));
    }

    #[test]
    fn unusable_region_read_is_prohibited() {
        let mmu = blank_mmu();
        assert_eq!(mmu.read(0xFEA0), Err(EmuError::ProhibitedRead));
        assert_eq!(mmu.read(0xFEFF), Err(EmuError::ProhibitedRead));
    }

    #[test]
    fn unusable_region_write_is_prohibited() {
        let mut mmu = blank_mmu();
        assert_eq!(mmu.write(0xFEA0, 1), Err(EmuError::ProhibitedWrite));
        assert_eq!(mmu.write(0xFEFF, 1), Err(EmuError::ProhibitedWrite));
    }

    #[test]
    fn regions_either_side_are_allowed() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.read(0xDFFF)?; // last WRAM byte
        mmu.read(0xFE9F)?; // last OAM byte
        mmu.read(0xFF00)?; // first IO register
        mmu.write(0xDFFF, 0)?;
        mmu.write(0xFE9F, 0)?;
        mmu.write(0xFF00, 0)?;
        Ok(())
    }

    #[test]
    fn read_word_reports_word_error() {
        let mmu = blank_mmu();
        assert_eq!(mmu.read_word(0xFEA0), Err(EmuError::ProhibitedReadWord));
        // straddling: low byte fine, high byte prohibited
        assert_eq!(mmu.read_word(0xDFFF), Err(EmuError::ProhibitedReadWord));
    }

    #[test]
    fn write_word_reports_word_error() {
        let mut mmu = blank_mmu();
        assert_eq!(mmu.write_word(0xFEA0, 1), Err(EmuError::ProhibitedWriteWord));
        assert_eq!(mmu.write_word(0xDFFF, 1), Err(EmuError::ProhibitedWriteWord));
    }

    #[test]
    fn word_at_top_of_memory_does_not_overflow() {
        // 0xFFFF + 1 overflows a u16: should be an error, not a panic.
        let mmu = blank_mmu();
        assert_eq!(mmu.read_word(0xFFFF), Err(EmuError::ProhibitedReadWord));
    }
}

mod dma {
    use super::*;

    #[test]
    fn writing_ff46_copies_160_bytes_to_oam() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        for i in 0..0xA0u16 {
            mmu.write(0xC100 + i, i as u8)?;
        }
        mmu.write(0xFF46, 0xC1)?; // source = 0xC100
        // Real DMA takes 640 cycles; doing it instantly is fine to start with.
        run(&mut mmu, 640);
        for i in 0..0xA0u16 {
            assert_eq!(mmu.read(0xFE00 + i)?, i as u8, "OAM byte {i}");
        }
        Ok(())
    }
}

mod serial {
    use super::*;

    #[test]
    fn starting_a_transfer_appends_sb_to_output() -> EmuRes<()> {
        // Blargg's test ROMs print their results this way.
        let mut mmu = blank_mmu();
        for c in "Passed".bytes() {
            mmu.write(0xFF01, c)?; // SB
            mmu.write(0xFF02, 0x81)?; // SC: start, internal clock
        }
        assert_eq!(mmu.serial_output(), "Passed");
        Ok(())
    }
}

mod timer {
    use super::*;

    const DIV: u16 = 0xFF04;
    const TIMA: u16 = 0xFF05;
    const TMA: u16 = 0xFF06;
    const TAC: u16 = 0xFF07;

    #[test]
    fn div_increments_every_256_cycles() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(DIV, 0)?;
        run(&mut mmu, 252);
        assert_eq!(mmu.read(DIV)?, 0);
        run(&mut mmu, 4);
        assert_eq!(mmu.read(DIV)?, 1);
        run(&mut mmu, 256 * 9);
        assert_eq!(mmu.read(DIV)?, 10);
        Ok(())
    }

    #[test]
    fn writing_div_resets_it() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        run(&mut mmu, 256 * 5);
        mmu.write(DIV, 0x99)?;
        assert_eq!(mmu.read(DIV)?, 0);
        Ok(())
    }

    #[test]
    fn tima_does_not_count_when_disabled() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(TAC, 0b001)?; // fastest speed, but enable bit clear
        mmu.write(TIMA, 0)?;
        run(&mut mmu, 1024);
        assert_eq!(mmu.read(TIMA)?, 0);
        Ok(())
    }

    #[test]
    fn tima_counts_at_selected_speed() -> EmuRes<()> {
        // TAC 0b01 = every 16 cycles, 0b10 = 64, 0b11 = 256, 0b00 = 1024
        for (tac, period) in [(0b01u8, 16u32), (0b10, 64), (0b11, 256), (0b00, 1024)] {
            let mut mmu = blank_mmu();
            mmu.write(DIV, 0)?; // also resets the internal counter
            mmu.write(TIMA, 0)?;
            mmu.write(TAC, 0b100 | tac)?;
            run(&mut mmu, period * 3);
            assert_eq!(mmu.read(TIMA)?, 3, "TAC = {tac:#04b}");
        }
        Ok(())
    }

    #[test]
    fn tima_overflow_reloads_tma_and_requests_interrupt() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(DIV, 0)?;
        mmu.write(IF, 0)?;
        mmu.write(TMA, 0xF0)?;
        mmu.write(TIMA, 0xFF)?;
        mmu.write(TAC, 0b101)?;
        run(&mut mmu, 16 + 8); // overflow, plus a few cycles for the delayed reload
        assert_eq!(mmu.read(TIMA)?, 0xF0);
        assert_ne!(mmu.read(IF)? & (1 << 2), 0, "timer interrupt not requested");
        Ok(())
    }
}

mod interrupts {
    use super::*;

    #[test]
    fn ppu_vblank_sets_if_bit_0() -> EmuRes<()> {
        let mut mmu = blank_mmu();
        mmu.write(0xFF40, 0x80)?; // LCD on
        mmu.write(IF, 0)?;
        run(&mut mmu, 456 * 144 + 4);
        assert_ne!(mmu.read(IF)? & 1, 0);
        Ok(())
    }
}
