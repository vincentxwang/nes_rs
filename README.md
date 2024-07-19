# nes_rs

A (to-be performant) Rust-based emulation of the [Nintendo Entertainment System](https://en.wikipedia.org/wiki/Nintendo_Entertainment_System).

![Pac-man image](assets/pacman.png)

# Features

- [NES 6502](https://www.nesdev.org/wiki/CPU) implementation 

# Runs...

| ROM                 | Status                                                 |
|---------------------|--------------------------------------------------------|
| Pac-man             | :white_check_mark:                                     |
| Balloon Fight       | kind of, something's funky with controls               |
| Donkey Kong         | garbled title screen, but seems to work fine otherwise |
| nestest             | :white_check_mark:                                     |
| The Legend of Zelda | :x:                                                    |
| Final Fantasy       | :x:                                                    |

# Roadmap

- CPU
    - [X] Official instructions
    - [X] BUS
    - [X] Unofficial instructions
    - [X] Cycle accuracy
- Cartridges
    - [X] iNES format
    - [ ] Mapper 1
- PPU
    - [X] PPU registers
    - [X] NMI interrupt
    - [X] Background rendering
    - [X] Foreground rendering
    - [ ] Scrolling
    - [ ] Correct DMA behavior
- UI
    - [X] Controller mappings
- APU
    - [ ] APU
- Testing/docs
    - [X] 6502 test suite
    - [ ] Blargg CPU/PPU tests
    - [ ] Docs

