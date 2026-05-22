# Binary Cursor
> Linux only (uses `evdev` / `uinput`)

## Table of Contents

- [Intro](#intro)
  - [What's this](#whats-this)
  - [Demo (Copy & Paste)](#demo-copy--paste)
  - [Features](#features)
- [Tested on](#tested-on)
- [Getting Started](#getting-started)
  - [Build](#build)
  - [Setup](#setup)
- [Configuration & Examples](#configuration--examples)
  - [Virtual Mouse Settings](#virtual-mouse-settings)
  - [Keybinds](#keybinds)
  - [Rewire](#rewire)
  - [Per-device configuration](#per-device-configuration)
- [References](#references)
- [License](#license)

<br>

## Intro

### What's this

Built to fully control the mouse using only the keyboard.

Inspired by the grid mode of existing solutions like warpd and mouseless, this tool blends that idea with a natural mode in a simplified form, aiming to give you the feeling of **owning the cursor in real time**.

<br>

### Demo (Copy & Paste)
**Full mouse control — movement, scrolling, clicks, and drag — keyboard only.**

https://github.com/user-attachments/assets/de68d664-9fa7-41c9-8a45-5c69f5ace63e

<br>

### Features

#### Three trigger modes

| Mode | Description |
| --- | --- |
| **Linear** | The cursor moves by a fixed step at a constant rate. |
| **Logarithmic** | Each input moves the cursor by half the previous distance (`128 → 64 → 32 ...`). Used for fine-tuning the cursor via **binary search** after a rough move with Linear mode. |
| **Scroll** | Combined with the configured directional keys, this mode performs up/down/left/right wheel scrolling. |

> [!IMPORTANT]
> - For uniform cursor movement, disable **mouse acceleration** in your system settings.
> - Left and right mouse clicks work in any mode.
> - When keybinds overlap, trigger priority is **scroll(highest)** → **logarithmic** → **linear**.

#### Other characteristics

- **Composable primitives** — Bind only a few basic mouse actions to keys, and their combinations work as-is. For instance, moving the cursor while holding click produces a drag, and pressing two adjacent direction keys together produces diagonal movement.
- **No overlay** — Nothing extra is drawn on the screen. You press keys, and the cursor moves. That's it.
- **Hold-to-trigger** — To switch quickly between regular typing and momentary mouse control without finger gymnastics, every mode is triggered by **holding** the key, not by toggling.
- **Per-key distinction** — Built on the [evdev](https://github.com/emberian/evdev) crate, using its raw scancodes(Follows the Linux kernel scancodes) for key identification. Every key is uniquely distinguishable — for example, **left and right Control** can be bound separately.
- **Per-device configuration** — For keymap and rewire settings, you can create separate configurations that apply only to a specific keyboard device by referencing the device name (see [Per-device configuration](#per-device-configuration)).
- **Hot-plug support** — Detects keyboard connect/disconnect events at runtime and reconfigures automatically. No need to restart after plugging in a new keyboard via USB or Bluetooth.
- **Selective grab** — You can decide whether each mode-trigger key is grabbed (see [Virtual Mouse Settings](#virtual-mouse-settings)). For instance, if Shift is set as a mode trigger, leaving it ungrabbed lets it serve double duty as both a trigger and the normal case-shift key.
- **Key rewiring** — A simple config file (`rewire.conf`) lets you translate physical key inputs into different key inputs (see [Rewire](#rewire)).

<br>

## Tested on
  - **OS**: EndeavourOS (Arch-based), x86_64
  - **Kernel**: 6.19.11-arch1-1
  - **Compositor**: Hyprland 0.54.3 (Wayland)


<br>

## Getting Started

### Build

#### Dependencies
  - Rust 1.85+ (edition 2024)
  - `pkg-config`
  - `libudev` development headers
    - Debian/Ubuntu: `sudo apt install libudev-dev pkg-config`
    - Fedora: `sudo dnf install systemd-devel pkgconf`
    - Arch: already included with `systemd` (default)

```bash
git clone https://github.com/the-legend-0f-zelda/bincur
cd bincur
cargo build --release
```
Binary will be at `target/release/bincur`.

<br>

### Setup

A helper script handles permission setup, binary installation, default config file creation, and systemd user service registration in one shot.

```bash
./scripts/setup.sh
```
> [!IMPORTANT]
> **Don't run as root.** The script invokes `sudo` internally only for steps that actually need root. (It caches credentials with `sudo -v` at start, so you'll only be prompted for your password once.)

What the script does:

1. **Add user to the `input` group** — for `/dev/input/event*` access (requires re-login)
2. **Write the udev rule** (`/etc/udev/rules.d/99-bincur-uinput.rules`) — exposes `/dev/uinput` to the `input` group with mode 0660
3. **Install the binary** — `target/release/bincur` → `~/.local/bin/bincur` (run `cargo build --release` first)
4. **Write default config files** — `~/.config/bincur/vmouse.conf`, `~/.config/bincur/keymap.conf` (skipped if they already exist)
5. **Register the systemd user service** — `~/.config/systemd/user/bincur.service`
---
After setup, **start** it with:

```bash
systemctl --user start bincur
```

To **revert**:

```bash
./scripts/setup.sh --uninstall
```

This stops/removes the service, deletes the binary, and removes the udev rule. User config files (`~/.config/bincur/*.conf`) and `input` group membership are preserved (other tools may depend on them).

For usage info: `./scripts/setup.sh --help`

<br>

## Configuration & Examples

Place the config files under the directory pointed to by the `BINCUR_CONF_HOME` environment variable (takes precedence) or `~/.config/bincur` (default).

- `vmouse.conf` — properties of the emulated virtual mouse
- `keymap.conf` — keybind configuration
- `rewire.conf` — settings that translate physical key inputs into different key input events.

The examples below are tuned to minimize finger movement from the typing home position (fingers resting on `a`, `s`, `d`, `f`, `j`, `k`, `l`, `;`).

### Virtual Mouse Settings

`~/.config/bincur/vmouse.conf:`
```conf
# Grab mode trigger keys
# When set to TRUE, key-combo input events bound to the corresponding mode are consumed and not propagated to other applications.
grab_linear : FALSE
grab_logarithmic : TRUE
grab_scroll : TRUE

# Cursor step size
step_size_x : 256
step_size_y : 256

# Wheel scroll distance
scroll_dist_x : 2
scroll_dist_y : 2
```

### Keybinds

`~/.config/bincur/keymap.conf:`
```conf
# 0. Terminate process
leftctrl+q : EXIT

# 1. Trigger virtual mouse mode
leftalt : LINEAR_MODE
leftalt+leftshift : LOGARITHMIC_MODE
leftalt+c : SCROLL_MODE

# 2. Move cursor or scroll
i : MOVE_UP
k : MOVE_DOWN
j : MOVE_LEFT
l : MOVE_RIGHT

# 3. Mouse click
space : CLICK_LEFT
semicolon : CLICK_RIGHT
```

### Rewire

The example below swaps the inputs of the keyboard's Left Alt and Left Meta keys.

`~/.config/bincur/rewire.conf`
```conf
leftalt -> leftmeta
leftmeta -> leftalt
```

#### Tips

- In the example, `LOGARITHMIC_MODE` and `SCROLL_MODE` are bound to combinations with `leftalt` to prevent `leftshift` or `c` alone from being grabbed by `vmouse.conf`.
- Although keys are written in lowercase and values in uppercase in the example, the parser is fully case-insensitive — write either side however you want, and you can even mix cases (e.g., camelCase) within a single token.
- To find the name of a key you want to bind, see **Key names** under [References](#references). Strip the **`KEY_`** prefix from each scancode identifier and the remainder is the bindable key name (e.g., `KEY_LEFTCTRL` → `LEFTCTRL`).
- If background apps reacting to modifiers like Meta or Alt — when you use them as mode triggers — bothers you, there are three main solutions.
  - First, set the corresponding mode's `grab` option to `TRUE` in `vmouse.conf`. The side effect is that no other application, including the compositor, can use that key.
  - Second, use `rewire.conf`. For example, if you want to use the left Meta key as a bincur mode trigger but a browser reacting to the `Super_L` symbol is annoying, you can add ```leftmeta -> f18``` in `rewire.conf`. Since most apps don't react to F18, only bincur gets triggered. If you want a compositor or certain apps to still use the left Meta selectively, replace the left Meta key with F18 in each of those apps' keybind settings. However, if an app reacts to every key input except specific modifier symbols, you'll need to use the first or third solution.
  - Third, change the key's keysym through xkb configuration. For example, on Hyprland, setting the mode trigger key's keysym to `Hyper_L` makes background apps that react to the `Super_L` symbol stop responding, while still letting the key serve as the main mod key for bincur and Hyprland triggers.

### Per-device configuration
When you switch between keyboards with different layouts, the keybinds that feel comfortable on each can differ. To save you from manually editing config files every time you swap between a gaming keyboard, a work keyboard, or a laptop's built-in keyboard, bincur provides per-device keybind and rewire configuration.

By default, `keymap.conf` and `rewire.conf` apply to all keyboards. If you want a different configuration for a specific keyboard only, name the files `keymap.<device-name>.conf` and `rewire.<device-name>.conf` respectively and write your settings there. You can easily look up your keyboard device's name through inspect mode.

Run inspect mode with the following command:
```bash
bincur -i
```

After it starts, press any key on the keyboard whose name you want to find. The terminal will print the device name, internal index, and key event info for the keyboard that fired the event.

---


## References

- [Key names (evdev scancodes)](https://docs.rs/evdev/0.13.2/src/evdev/scancodes.rs.html#26-579) — valid key identifiers for the left side of `:`

<br>

## License

Copyright (C) 2026 master@scamsite.biz

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License, version 3, as published by the
Free Software Foundation. See the [LICENSE](LICENSE) file for the full text.

> **AI disclosure:** This README was originally written in Korean by the author;
> its English text was translated with the assistance of AI tools. The shell
> script under `scripts/` was written with AI assistance.
