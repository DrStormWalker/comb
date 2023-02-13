# CoMB (Corroded Macro Bindings)

An application that allows you to map gamepad inputs, keyboard events,
mouse inputs and more to keyboard inputs and other actions

## About

CoMB is implemented through the use of evdev a generic input device interface
that generalizes inputs from different drivers. 

CoMB uses Rust's [mio](https://github.com/tokio-rs/mio) library to allow for non-blocking
access to the evdev devices.

## Features

- [X] Watch config file for changes
- [X] Watch for added/removed devices
- [X] Button/Keyboard key inputs
  - [ ] Map to virtual device input
  - [X] Run script
  - [ ] Modifier keys
- [ ] Joystick inputs
  - [ ] Map to different joystick
  - [ ] Interpret as keypress
  - [ ] Run script
- [ ] Human readable errors and warnings
- [ ] Grab input device
- [ ] Multiple Virtual input devices
- [ ] Midi input?

## Installing

### Cargo

```sh
git clone https://github.com/DrStormWalker/comb
cd comb
cargo install --locked
```

### NixOS

Install via flakes:

```sh
nix build "github:DrStormWalker/comb#comb"
./result/bin/comb
```

### Requirements

- A Rust nightly toolchain

## Configuration

TODO

## Platforms

Currently supported platforms:

- Linux

There are potentially other platforms that CoMB could run on. If you find
another platform CoMB runs on, please submit a PR to update the list!
