# Yet another gameboy emulator written in Rust

## Introduction
This is a toy emulator for me experiment with as I learn rust.

## Running

Specify the path to the rom as input to the program.

```bash
cargo run --release
```

## Tests

The tests in `src/mmu/tests.rs`, `src/mmu/ppu/tests.rs` and `src/cpu/tests.rs` are targets to work through.
Each file lists a suggested order at the top. Run one group at a time:

```bash
cargo test mmu::tests::memory_map
cargo test ppu::tests::timing
cargo test cpu::tests::cycles -- --nocapture   # lists every opcode with wrong cycles
cargo test --release blargg -- --ignored --nocapture   # real test ROMs
```

## Todo
- [ ] Complete CPU
	- [ ] Prefixed Operation
	- [x] Unprefixed Operation

- [ ] MMU
	- [ ] Cart File
	- [ ] PPU
	- [ ] IoRegister File

- [ ] Audio (Not started)

- [ ] GUI for selecting ROM?

## Screenshots

## References
- `opcodes.json` is taken from [here](https://gbdev.io/gb-opcodes/Opcodes.json)
