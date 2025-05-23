# Yet another gameboy emulator written in Rust

## Introduction
This is a toy emulator for me experiment with as I learn rust.

## Running

Specify the path to the rom as input to the program.

```bash
cargo run --release
```

## Todo
- [x] Complete CPU
	- [x] Prefixed Operation
	- [x] Unprefixed Operation

- [ ] MMU
	- [ ] Cart File
	- [ ] PPU
	- [ ] IoRegister File
	- [x] Initial Version of PPU (read correct address on bus)

- [ ] Audio

- [ ] GUI for selecting ROM?

## Screenshots

## References
- `opcodes.json` is taken from [here](https://gbdev.io/gb-opcodes/Opcodes.json)
