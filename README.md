# gb-debug

This is a Game Boy Debugger and Emulator.

## Features

- [x] Disassembler
- [x] Tile Map Viewer
- [x] Tile Data Viewer
- [x] Memory Dump Viewer
- [x] CPU Breakpoints
- [x] DMG Emulator
- [ ] CGB Emulator (It's mostly broken still)
- [ ] Sound Emulation
- [ ] Serial I/O
- [ ] Save States
- [ ] Game Genie Codes
- [ ] Game Shark Codes
- [ ] Memory Breakpoints
- [ ] VRAM Viewer
- [ ] OAM Viewer
- [ ] I/O Viewer
- [ ] Timer Viewer
- [ ] Interrupt Viewer
- [ ] RTC Viewer

## Building from Source

You can build this project from source by using cargo.

```bash
git clone https://github.com/sten-code/gb-debug.git
cd gb-debug
cargo run --release
```

## Sources

These are the sources I used to help me build this project.

- https://gbdev.io/pandocs
- https://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM
- https://gbdev.io/gb-opcodes/optables/
- https://rgbds.gbdev.io/docs/v0.8.0/gbz80.7
- https://rylev.github.io/DMG-01/public/book/introduction.html
- https://github.com/rylev/DMG-01/tree/master/lib-dmg-01/src
- https://github.com/mvdnes/rboy
- https://github.com/Gekkio/mooneye-gb