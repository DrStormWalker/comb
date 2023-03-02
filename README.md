# CoMB (Corroded Macro Bindings)

An application that allows you to map gamepad inputs, keyboard events,
mouse inputs and more to keyboard inputs and other actions

## About

CoMB is implemented through the use of evdev a generic input device interface
that generalizes inputs from different drivers.

CoMB uses Rust's [tokio](https://github.com/tokio-rs/tokio) library to allow for non-blocking
access to the evdev devices. One can opt to use Rust's [mio](https://github.com/tokio-rs/mio)
library instead by disabling default features, however this will limit the amount features that
are enabled.

## Features

- [X] Watch config file for changes
- [X] Watch for added/removed devices
- [X] Button/Keyboard key inputs
  - [X] Map to virtual device input
  - [X] Run script
  - [ ] Modifier keys
- [X] Joystick inputs
  - [X] Map to different joystick
  - [X] Interpret as keypress
  - [X] Run script
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

CoMB can be configured through the configuration file at `~/.config/comb/config.toml`.

Alternatively if `$XDG_CONFIG_HOME` is defined then CoMB can be configured throught the
file `$XDG_CONFIG_HOME/comb/config.toml`. If the config file cannot be found at any of
the places above `$XDG_CONFIG_DIRS` will be searched. If the file is not
found in any of these locations a new config file will be created at
`$XDG_CONFIG_HOME/comb/config.toml` or `~/.config/comb/config.toml`.

### Sample Configuration file

```toml
[[devices]]
name = "8BitDo Zero 2 gamepad"

[[devices]]
path = "/dev/input/event25"

[[devices.actions]]
bind = "btn:z"
to = "key:leftmeta"

[[devices.actions]]
bind = "key:z"
to = "key:leftmeta"

[[devices.actions]]
bind = "key:p"
cmd = "swaylock"

[[devices.actions]]
bind = "key:r"
cmd = "swaylock"
when = "released"

[[devices.actions]]
bind = "abs_axis:y"
to = "key:down"
when = ">32768"

[[devices.actions]]
bind = "abs_axis:y"
to = "key:up"
when = "<32768"
```

## Platforms

Currently supported platforms:

- Linux

There are potentially other platforms that CoMB could run on. If you find
another platform CoMB runs on, please submit a PR to update the list!
