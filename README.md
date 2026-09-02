# puro_boy — a Game Boy emulator in Rust

A Game Boy (DMG) emulator written to learn Rust and to understand the hardware from the instruction set up. CPU is complete and passes <!-- FILL: e.g. Blargg's cpu_instrs tests, or "the opcode table" -->; memory mapping and graphics are in progress.

## Status

| Component | State |
|---|---|
| CPU (SM83): all unprefixed and CB-prefixed opcodes, flags, timing | Done |
| Registers, interrupts | Done |
| MMU: bus, I/O register file | Partial |
| Cartridge loading, MBC ROM banking | In progress |
| PPU: tile fetch, scanline rendering | Initial version reads correct bus addresses; no output yet |
| Audio, input | Not started |

Next milestone: boot the DMG boot ROM and render the Nintendo logo scroll.

## Layout

```
src/cpu/            instruction decode & execute, opcodes.json drives the table
src/mmu/            bus, bank switching, ppu
src/main.rs         load ROM, run loop
```

## Running

```bash
cargo run --release -- path/to/rom.gb
```

